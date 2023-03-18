use bevy::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier3d::prelude::*;
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
pub struct OwningPlayer(pub usize);

#[derive(Component)]
pub struct Torso;

#[derive(Component)]
pub struct LeftHand;

#[derive(Component)]
pub struct RightHand;

#[derive(Component)]
pub struct Feet;

#[derive(Component)]
pub struct Head;

pub struct PlayerEntity {
    pub head: Entity,
    pub torso: Entity,
    pub feet: Entity,
    pub left_hand: Entity,
    pub right_hand: Entity,
}

pub fn spawn_player(commands: &mut Commands, player_id: usize) -> PlayerEntity {
    let player = PlayerEntity {
        head: commands.spawn(Head).id(),
        torso: commands.spawn(Torso).id(),
        feet: commands.spawn(Feet).id(),
        left_hand: commands.spawn(LeftHand).id(),
        right_hand: commands.spawn(RightHand).id(),
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
                transform: Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: 1.5,
                    z: 0.0,
                }),
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
            TransformBundle::from_transform(Transform::from_translation(Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            })),
            VisibilityBundle::default(),
        ));

    commands.entity(player.feet).insert((
        OwningPlayer(player_id),
        TransformBundle::from_transform(Transform::from_translation(Vec3::ZERO)),
        VisibilityBundle::default(),
    ));

    commands.entity(player.left_hand).insert((
        OwningPlayer(player_id),
        TransformBundle::from_transform(Transform::from_translation(Vec3 {
            x: -0.2,
            y: -0.2,
            z: -0.5,
        })),
        VisibilityBundle::default(),
    ));

    commands.entity(player.right_hand).insert((
        OwningPlayer(player_id),
        TransformBundle::from_transform(Transform::from_translation(Vec3 {
            x: 0.2,
            y: -0.2,
            z: -0.5,
        })),
        VisibilityBundle::default(),
    ));

    player
}

pub fn head_bobbing(
    time: Res<Time>,
    torsos: Query<&Velocity, (With<Torso>, Without<Head>)>,
    mut heads: Query<(&mut Transform, &Parent), (Without<Torso>, With<Head>)>,
) {
    let dt = time.elapsed_seconds();

    for (mut transform, torso) in heads.iter_mut() {
        let Ok(velocity) = torsos.get(torso.get()) else {
            continue;
        };

        let base_translation = Vec3 {
            x: 0.0,
            y: 1.5,
            z: 0.0,
        };

        let head_bob = Vec3 {
            x: 0.0,
            y: 0.02 * velocity.linvel.length(),
            z: 0.0,
        } * f32::sin(dt * 10.0);

        transform.translation = base_translation + head_bob;
    }
}

pub fn right_hand_bobbing(
    time: Res<Time>,
    torsos: Query<&Velocity, (With<Torso>, Without<RightHand>, Without<Head>)>,
    heads: Query<&Parent, (Without<Torso>, Without<RightHand>, With<Head>)>,
    mut hands: Query<(&mut Transform, &Parent), (Without<Torso>, With<RightHand>, Without<Head>)>,
) {
    let dt = time.elapsed_seconds();

    for (mut transform, head) in hands.iter_mut() {
        let Ok(torso) = heads.get(head.get()) else {
            continue;
        };

        let Ok(velocity) = torsos.get(torso.get()) else {
            continue;
        };

        let base_translation = Vec3 {
            x: 0.2,
            y: -0.2,
            z: -0.5,
        };

        let hand_bob = Vec3 {
            x: 0.0,
            y: 0.002 * velocity.linvel.length(),
            z: 0.0,
        } * f32::sin(dt * 10.0 + TAU / 4.0);

        transform.translation = base_translation + hand_bob;
    }
}

pub fn left_hand_bobbing(
    time: Res<Time>,
    torsos: Query<&Velocity, (With<Torso>, Without<LeftHand>, Without<Head>)>,
    heads: Query<&Parent, (Without<Torso>, Without<LeftHand>, With<Head>)>,
    mut hands: Query<(&mut Transform, &Parent), (Without<Torso>, With<LeftHand>, Without<Head>)>,
) {
    let dt = time.elapsed_seconds();

    for (mut transform, head) in hands.iter_mut() {
        let Ok(torso) = heads.get(head.get()) else {
            continue;
        };

        let Ok(velocity) = torsos.get(torso.get()) else {
            continue;
        };

        let base_translation = Vec3 {
            x: -0.2,
            y: -0.2,
            z: -0.5,
        };

        let hand_bob = Vec3 {
            x: 0.0,
            y: 0.002 * velocity.linvel.length(),
            z: 0.0,
        } * f32::sin(dt * 10.0 + 3.0 * TAU / 4.0);

        transform.translation = base_translation + hand_bob;
    }
}
