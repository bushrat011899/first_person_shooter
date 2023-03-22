use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_ggrs::PlayerInputs;
use std::f32::consts::*;

use crate::{GGRSConfig, player::OwningPlayer, config::UserAction};

use super::FpsControllerInput;

/// Minimum angle value in radians.
///
/// TODO: Number appears arbitrary, should either be simplified or explained.
const ANGLE_EPSILON: f32 = 0.001953125;

pub fn map_player_input_to_controller_input(
    inputs: Res<PlayerInputs<GGRSConfig>>,
    mut query: Query<(&OwningPlayer, &mut FpsControllerInput)>,
) {
    for (OwningPlayer(player_handle), mut controller_input) in query.iter_mut() {
        let Some((player_input, status)) = inputs.get(*player_handle) else {
            log::warn!("Player {player_handle} does not have a controller!");
            continue;
        };

        if let ggrs::InputStatus::Disconnected = status {
            log::warn!("Player {player_handle} has disconnected but the controller isn't despawned!");
            continue;
        }

        // Map pointer motion to controller orientation
        let pointer_delta: Vec2 = player_input.pointer.into();

        controller_input.pitch = (controller_input.pitch - pointer_delta.y)
            .clamp(-FRAC_PI_2 + ANGLE_EPSILON, FRAC_PI_2 - ANGLE_EPSILON);

        controller_input.yaw -= pointer_delta.x;

        if controller_input.yaw.abs() > PI {
            controller_input.yaw = controller_input.yaw.rem_euclid(TAU);
        }

        // Map button inputs to controller translation
        let x = match (player_input.buttons.get(UserAction::MoveRight), player_input.buttons.get(UserAction::MoveLeft)) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };

        let z = match (player_input.buttons.get(UserAction::MoveForward), player_input.buttons.get(UserAction::MoveBackward)) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };

        controller_input.movement = Vec3::new(
            x,
            0.0,
            z,
        );
        
        controller_input.sprint = player_input.buttons.get(UserAction::Sprint);
        controller_input.jump = player_input.buttons.get(UserAction::Jump);
        controller_input.crouch = player_input.buttons.get(UserAction::Crouch);
    }
}
