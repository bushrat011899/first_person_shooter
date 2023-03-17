use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_fps_controller::controller::*;
use std::f32::consts::*;

/* 
    A player consists of hands, legs, a torso, and a head.
    These parts are arranged in a parent-child relationship as:

     * Torso
        * Legs
        * Head
            * Left Hand
            * Right Hand

    This (anatomically incorrect) relationship allows:
     * The player camera to be placed on its head.
     * Hands to follow the head's movements (freelook).
     * The head and legs to follow the torso's transform (translation, rotation).
     * The legs to contract upwads towards the torso (crouching).
*/

#[derive(Component)]
struct OwningPlayer(usize);

#[derive(Component)]
struct Torso;

#[derive(Component)]
struct LeftHand;

#[derive(Component)]
struct RightHand;

#[derive(Component)]
struct Feet;

#[derive(Component)]
struct Head;

pub struct PlayerEntity {
    pub head: Entity,
    pub torso: Entity,
    pub feet: Entity,
    pub left_hand: Entity,
    pub right_hand: Entity,
}

pub fn spawn_player(commands: &mut Commands, player_id: usize) -> PlayerEntity {
    let player = PlayerEntity {
        head: commands
            .spawn(Head)
            .id(),
        torso: commands
            .spawn(Torso)
            .id(),
        feet: commands
            .spawn(Feet)
            .id(),
        left_hand: commands
            .spawn(LeftHand)
            .id(),
        right_hand: commands
            .spawn(RightHand)
            .id(),
    };

    commands
        .entity(player.head)
        .push_children(&[player.left_hand, player.right_hand])
        .insert((
            OwningPlayer(player_id),
            Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: TAU / 5.0,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3 { x: 0.0, y: 1.5, z: 0.0 }),
                ..default()
            },
            VisibilityBundle::default(),
            AudioReceiver,
        ));

    commands
        .entity(player.torso)
        .push_children(&[player.head, player.feet])
        .insert((
            OwningPlayer(player_id),
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
            Ccd { enabled: true },
            FpsControllerBundle::default(),
            TransformBundle::from_transform(Transform::from_translation(Vec3 { x: 0.0, y: 1.0, z: 0.0 })),
            VisibilityBundle::default(),
        ));
    
    commands
        .entity(player.feet)
        .insert((
            OwningPlayer(player_id),
            TransformBundle::from_transform(Transform::from_translation(Vec3::ZERO)),
            VisibilityBundle::default(),
        ));
    
    commands
        .entity(player.left_hand)
        .insert((
            OwningPlayer(player_id),
            TransformBundle::from_transform(Transform::from_translation(Vec3 { x: -0.2, y: -0.2, z: -0.5 })),
            VisibilityBundle::default(),
        ));
    
    commands
        .entity(player.right_hand)
        .insert((
            OwningPlayer(player_id),
            TransformBundle::from_transform(Transform::from_translation(Vec3 { x: 0.2, y: -0.2, z: -0.5 })),
            VisibilityBundle::default(),
        ));

    player

}