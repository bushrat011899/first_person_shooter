use std::{f32::consts::TAU, time::Duration};

use bevy::{
    gltf::Gltf,
    gltf::{GltfMesh, GltfNode},
    math::Vec3Swizzles,
    prelude::*,
    window::{CursorGrabMode, WindowMode},
};
use bevy_rapier3d::prelude::*;

use bevy_fps_controller::controller::*;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 0.0);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.5,
        })
        .insert_resource(ClearColor(Color::hex("D4F5F5").unwrap()))
        .insert_resource(RapierConfiguration::default())
        .insert_resource(SimplePerformance {
            frames: 0.0,
            delta_time: 0.0,
            frame_time: f32::INFINITY
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FpsControllerPlugin)
        .add_startup_system(setup)
        .add_systems((manage_cursor, scene_colliders, display_text, respawn, fire_gun))
        .run();
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let mut window = window.single_mut();
    window.title = String::from("Minimal FPS Controller Example");
    window.mode = WindowMode::Fullscreen;

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 6000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Note that we have two entities for the player
    // One is a "logical" player that handles the physics computation and collision
    // The other is a "render" player that is what is displayed to the user
    // This distinction is useful for later on if you want to add multiplayer,
    // where often time these two ideas are not exactly synced up
    commands
        .spawn((
            Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            FpsControllerBundle {
                controller: FpsController {
                    air_acceleration: 80.0,
                    ..default()
                },
                ..default()
            },
            TransformBundle::from_transform(Transform::from_translation(SPAWN_POINT)),
            VisibilityBundle::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                        Camera3dBundle {
                        projection: Projection::Perspective(PerspectiveProjection {
                            fov: TAU / 5.0,
                            ..default()
                        }),
                        ..default()
                    },
                    VisibilityBundle::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SceneBundle {
                            scene: assets.load("L1A1_aussie.glb#Scene0"),
                            transform: Transform::from_translation(Vec3 { x: 0.2, y: -0.2, z: -0.5 }),
                            ..Default::default()
                        },
                        Gun {
                            fire: assets.load("L1A1_aussie.glb#Animation0"),
                        },
                    ));
                });
        });

    commands.insert_resource(MainScene {
        handle: assets.load("playground.glb"),
        is_loaded: false,
    });

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: assets.load("fira_mono.ttf"),
                font_size: 24.0,
                color: Color::BLACK,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );
}

fn respawn(mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        if transform.translation.y > -50.0 {
            continue;
        }

        velocity.linvel = Vec3::ZERO;
        transform.translation = SPAWN_POINT;
    }
}

#[derive(Resource)]
struct MainScene {
    handle: Handle<Gltf>,
    is_loaded: bool,
}

#[derive(Resource)]
struct SimplePerformance {
    frames: f32,
    delta_time: f32,
    frame_time: f32
}

impl SimplePerformance {
    fn update(&mut self, delta_time: f32) {
        self.frames += 1.0;
        self.delta_time += delta_time;

        if self.delta_time > 1.0 && self.frames > 0.0 {
            self.frame_time = self.delta_time / self.frames;
            self.frames = 0.0;
            self.delta_time = 0.0;
        }
    }
}

#[derive(Component)]
struct Gun {
    fire: Handle<AnimationClip>
}

fn fire_gun(
    input: Res<Input<MouseButton>>,
    gun_query: Query<(Entity, &Gun), With<Gun>>,
    children: Query<&Children>,
    mut query: Query<&mut AnimationPlayer, Without<Gun>>,
) {
    for (gun_entity, gun) in gun_query.iter() {
        for child in children.iter_descendants(gun_entity) {
            let Ok(mut player) = query.get_mut(child) else {
                continue;
            };

            if input.just_pressed(MouseButton::Left) {
                player
                    .set_speed(2.0)
                    .play_with_transition(gun.fire.clone_weak(), Duration::from_millis(10))
                    .set_elapsed(0.0);
            }
        }
    }
}

fn scene_colliders(
    mut commands: Commands,
    mut main_scene: ResMut<MainScene>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<GltfMesh>>,
    gltf_node_assets: Res<Assets<GltfNode>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    if main_scene.is_loaded {
        return;
    }

    let Some(gltf) = gltf_assets.get(&main_scene.handle) else {
        return;
    };

    let scene = gltf.scenes.first().unwrap().clone();
    commands.spawn(SceneBundle { scene, ..default() });

    for node in &gltf.nodes {
        let node = gltf_node_assets.get(&node).unwrap();
        let Some(gltf_mesh) = node.mesh.clone() else {
            continue;
        };

        let gltf_mesh = gltf_mesh_assets.get(&gltf_mesh).unwrap();
        for mesh_primitive in &gltf_mesh.primitives {
            let mesh = mesh_assets.get(&mesh_primitive.mesh).unwrap();
            commands.spawn((
                Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap(),
                RigidBody::Fixed,
                TransformBundle::from_transform(node.transform),
            ));
        }
    }

    main_scene.is_loaded = true;
}

fn manage_cursor(
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut window_query: Query<&mut Window>,
    mut controller_query: Query<&mut KeyboardAndMouseInputBindings>,
) {
    let mut window = window_query.single_mut();
    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        for mut controller in &mut controller_query {
            controller.enabled = true;
        }
    }
    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        for mut controller in &mut controller_query {
            controller.enabled = false;
        }
    }
}

fn display_text(
    time: Res<Time>,
    mut perf: ResMut<SimplePerformance>,
    mut controller_query: Query<(&Transform, &Velocity)>,
    mut text_query: Query<&mut Text>,
) {
    let dt = time.delta_seconds();

    perf.update(dt);

    let frame_time = perf.frame_time;
    let fps = 1.0 / frame_time;

    for (transform, velocity) in &mut controller_query {
        for mut text in &mut text_query {
            text.sections[0].value = format!(
                "vel: {:.2}, {:.2}, {:.2}\npos: {:.2}, {:.2}, {:.2}\nspd: {:.2}\nspf: {:.3}\nfps: {:3.0}",
                velocity.linvel.x,
                velocity.linvel.y,
                velocity.linvel.z,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                velocity.linvel.xz().length(),
                frame_time,
                fps
            );
        }
    }
}
