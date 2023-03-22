use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier3d::prelude::*;

use super::{FpsController, FpsControllerInput, FreeLookState};

/// Component describing the current movement mode of a player.
#[derive(Component, PartialEq)]
pub enum MoveMode {
    /// No-Clip flying mode.
    Noclip,
    /// Standard ground-based movement.
    Ground,
}

impl Default for MoveMode {
    fn default() -> Self {
        Self::Ground
    }
}

/// System responsible for choosing the movement mode of the players.
pub fn choose_movement_mode(mut query: Query<(&FpsControllerInput, &mut MoveMode)>) {
    for (input, mut move_mode) in query.iter_mut() {
        if input.fly {
            *move_mode = match *move_mode {
                MoveMode::Noclip => MoveMode::Ground,
                MoveMode::Ground => MoveMode::Noclip,
            }
        }
    }
}

/// System responsible for mapping abstract look inputs into concrete player orientation.
pub fn map_input_orientation(mut query: Query<(&mut FpsController, &mut FpsControllerInput)>) {
    for (mut controller, mut input) in query.iter_mut() {
        controller.pitch = input.pitch;

        match input.free_look {
            FreeLookState::Not => controller.yaw = input.yaw,
            FreeLookState::Stopping => input.yaw = controller.yaw,
            _ => {}
        };
    }
}

/// System responsible for mapping abstract input movements into concreate player translation.
pub fn map_input_movement(
    time: Res<Time>,
    physics_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &FpsControllerInput,
        &MoveMode,
        &mut FpsController,
        &mut Collider,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    for (entity, input, move_mode, mut controller, mut collider, mut transform, mut velocity) in
        query.iter_mut()
    {
        match *move_mode {
            MoveMode::Noclip => {
                noclip_movement(input, &mut controller, &mut velocity);
            }
            MoveMode::Ground => {
                ground_movement(
                    &time,
                    &physics_context,
                    entity,
                    input,
                    &mut controller,
                    &mut collider,
                    &mut transform,
                    &mut velocity,
                );
            }
        }
    }
}

/// Subsystem responsible for controlling the player when in no-clip mode.
fn noclip_movement(
    input: &FpsControllerInput,
    controller: &mut FpsController,
    velocity: &mut Velocity,
) {
    if input.movement == Vec3::ZERO {
        let friction = controller.fly_friction.clamp(0.0, 1.0);
        velocity.linvel *= 1.0 - friction;
        if velocity.linvel.length_squared() < f32::EPSILON {
            velocity.linvel = Vec3::ZERO;
        }
    } else {
        let fly_speed = if input.sprint {
            controller.fast_fly_speed
        } else {
            controller.fly_speed
        };
        let mut move_to_world =
            Mat3::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.0);
        move_to_world.z_axis *= -1.0; // Forward is -Z
        move_to_world.y_axis = Vec3::Y; // Vertical movement aligned with world up
        velocity.linvel = move_to_world * input.movement * fly_speed;
    }
}

/// Subsystem responsible for controlling the player when in standard ground-based movement mode.
fn ground_movement(
    time: &Res<Time>,
    physics_context: &Res<RapierContext>,
    entity: Entity,
    input: &FpsControllerInput,
    controller: &mut FpsController,
    collider: &mut Collider,
    transform: &mut Transform,
    velocity: &mut Velocity,
) {
    let dt = time.delta_seconds();

    let Some(capsule) = collider.as_capsule() else {
        return;
    };

    // Capsule cast downwards to find ground
    // Better than a ray cast as it handles when you are near the edge of a surface
    let capsule = capsule.raw;
    let cast_capsule = Collider::capsule(
        capsule.segment.a.into(),
        capsule.segment.b.into(),
        capsule.radius * 0.9,
    );
    // Avoid self collisions
    let filter = QueryFilter::default().exclude_rigid_body(entity);
    let ground_cast = physics_context.cast_shape(
        transform.translation,
        transform.rotation,
        -Vec3::Y,
        &cast_capsule,
        0.125,
        filter,
    );

    let speeds = Vec3::new(controller.side_speed, 0.0, controller.forward_speed);
    let mut move_to_world = Mat3::from_axis_angle(Vec3::Y, controller.yaw);
    move_to_world.z_axis *= -1.0; // Forward is -Z
    let mut wish_direction = move_to_world * (input.movement * speeds);
    let mut wish_speed = wish_direction.length();
    if wish_speed > f32::EPSILON {
        // Avoid division by zero
        wish_direction /= wish_speed; // Effectively normalize, avoid length computation twice
    }
    let max_speed = if input.crouch {
        controller.crouched_speed
    } else if input.sprint {
        controller.run_speed
    } else {
        controller.walk_speed
    };
    wish_speed = f32::min(wish_speed, max_speed);

    if let Some((_, toi)) = ground_cast {
        let has_traction = Vec3::dot(toi.normal1, Vec3::Y) > controller.traction_normal_cutoff;

        // Only apply friction after at least one tick, allows b-hopping without losing speed
        if controller.ground_tick >= 1 && has_traction {
            let lateral_speed = velocity.linvel.xz().length();
            if lateral_speed > controller.friction_speed_cutoff {
                let control = f32::max(lateral_speed, controller.stop_speed);
                let drop = control * controller.friction * dt;
                let new_speed = f32::max((lateral_speed - drop) / lateral_speed, 0.0);
                velocity.linvel.x *= new_speed;
                velocity.linvel.z *= new_speed;
            } else {
                velocity.linvel = Vec3::ZERO;
            }
            if controller.ground_tick == 1 {
                velocity.linvel.y = -toi.toi;
            }
        }

        let mut add = acceleration(
            wish_direction,
            wish_speed,
            controller.acceleration,
            velocity.linvel,
            dt,
        );
        if !has_traction {
            add.y -= controller.gravity * dt;
        }
        velocity.linvel += add;

        if has_traction {
            let linvel = velocity.linvel;
            velocity.linvel -= Vec3::dot(linvel, toi.normal1) * toi.normal1;

            if input.jump {
                velocity.linvel.y = controller.jump_speed;
            }
        }

        // Increment ground tick but cap at max value
        controller.ground_tick = controller.ground_tick.saturating_add(1);
    } else {
        controller.ground_tick = 0;
        wish_speed = f32::min(wish_speed, controller.air_speed_cap);

        let mut add = acceleration(
            wish_direction,
            wish_speed,
            controller.air_acceleration,
            velocity.linvel,
            dt,
        );
        add.y = -controller.gravity * dt;
        velocity.linvel += add;

        let air_speed = velocity.linvel.xz().length();
        if air_speed > controller.max_air_speed {
            let ratio = controller.max_air_speed / air_speed;
            velocity.linvel.x *= ratio;
            velocity.linvel.z *= ratio;
        }
    }

    /* Crouching */

    let crouch_height = controller.crouch_height;
    let upright_height = controller.upright_height;

    let crouch_speed = if input.crouch {
        -controller.crouch_speed
    } else {
        controller.uncrouch_speed
    };
    controller.height += dt * crouch_speed;
    controller.height = controller.height.clamp(crouch_height, upright_height);

    if let Some(mut capsule) = collider.as_capsule_mut() {
        capsule.set_segment(Vec3::Y * 0.5, Vec3::Y * controller.height);
    }

    // Step offset
    if controller.step_offset > f32::EPSILON && controller.ground_tick >= 1 {
        let cast_offset = velocity.linvel.normalize_or_zero() * controller.radius * 1.0625;
        let cast = physics_context.cast_ray_and_get_normal(
            transform.translation + cast_offset + Vec3::Y * controller.step_offset * 1.0625,
            -Vec3::Y,
            controller.step_offset * 0.9375,
            false,
            filter,
        );
        if let Some((_, hit)) = cast {
            transform.translation.y += controller.step_offset * 1.0625 - hit.toi;
            transform.translation += cast_offset;
        }
    }

    // Prevent falling off ledges
    if controller.ground_tick >= 1 && input.crouch {
        for _ in 0..2 {
            // Find the component of our velocity that is overhanging and subtract it off
            let overhang = overhang_component(
                entity,
                transform,
                physics_context.as_ref(),
                velocity.linvel,
                dt,
            );
            if let Some(overhang) = overhang {
                velocity.linvel -= overhang;
            }
        }
        // If we are still overhanging consider unsolvable and freeze
        if overhang_component(
            entity,
            transform,
            physics_context.as_ref(),
            velocity.linvel,
            dt,
        )
        .is_some()
        {
            velocity.linvel = Vec3::ZERO;
        }
    }
}

fn overhang_component(
    entity: Entity,
    transform: &Transform,
    physics_context: &RapierContext,
    velocity: Vec3,
    dt: f32,
) -> Option<Vec3> {
    // Cast a segment (zero radius on capsule) from our next position back towards us
    // If there is a ledge in front of us we will hit the edge of it
    // We can use the normal of the hit to subtract off the component that is overhanging
    let cast_capsule = Collider::capsule(Vec3::Y * 0.125, -Vec3::Y * 0.125, 0.0);
    let filter = QueryFilter::default().exclude_rigid_body(entity);
    let future_position = transform.translation + velocity * dt;

    let cast = physics_context.cast_shape(
        future_position,
        transform.rotation,
        -velocity,
        &cast_capsule,
        0.5,
        filter,
    );

    let Some((_, toi)) = cast else {
        return None;
    };

    let cast = physics_context.cast_ray(
        future_position + Vec3::Y * 0.125,
        -Vec3::Y,
        0.375,
        false,
        filter,
    );

    if cast.is_some() {
        return None;
    }

    // Make sure that this is actually a ledge, e.g. there is no ground in front of us
    let normal = -toi.normal1;
    let alignment = Vec3::dot(velocity, normal);
    Some(alignment * normal)
}

fn acceleration(
    wish_direction: Vec3,
    wish_speed: f32,
    acceleration: f32,
    velocity: Vec3,
    dt: f32,
) -> Vec3 {
    let velocity_projection = Vec3::dot(velocity, wish_direction);
    let add_speed = wish_speed - velocity_projection;

    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let acceleration_speed = f32::min(acceleration * wish_speed * dt, add_speed);
    wish_direction * acceleration_speed
}
