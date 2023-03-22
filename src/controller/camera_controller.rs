use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{FpsController, FpsControllerInput, FreeLookState};

/// System responsible for transforming player cameras onto their controllers.
pub fn map_camera_transform(
    mut player_query: Query<
        (
            &mut Transform,
            &Collider,
            &FpsController,
            &FpsControllerInput,
        ),
        With<FpsController>,
    >,
    mut camera_query: Query<(&mut Transform, &Parent), (With<Camera>, Without<FpsController>)>,
) {
    for (mut camera_transform, player) in camera_query.iter_mut() {
        let Ok((mut player_transform, collider, controller, input)) = player_query.get_mut(player.get()) else {
            continue;
        };

        let Some(capsule) = collider.as_capsule() else {
            continue;
        };

        // TODO: let this be more configurable
        let camera_height = capsule.segment().b().y + capsule.radius() * 0.75;
        camera_transform.translation = Vec3::Y * camera_height;

        let (_, player_pitch, player_roll) = player_transform.rotation.to_euler(EulerRot::YXZ);
        let (camera_yaw, _, camera_roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);

        let camera_yaw = match input.free_look {
            FreeLookState::Not => camera_yaw,
            FreeLookState::Starting | FreeLookState::Looking => (input.yaw - controller.yaw),
            FreeLookState::Stopping => 0.0,
        };

        player_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, controller.yaw, player_pitch, player_roll);
        camera_transform.rotation =
            Quat::from_euler(EulerRot::YXZ, camera_yaw, controller.pitch, camera_roll);
    }
}
