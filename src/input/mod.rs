use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::Velocity;
use bytemuck::{Pod, Zeroable};
use enum_iterator::all;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};

pub use buttons::*;
pub use pointer::*;
pub use resync::*;

use crate::{config::UserAction, player::OwningPlayer, controller::FpsControllerInput};

mod buttons;
mod pointer;
mod resync;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Default, Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub buttons: ButtonInput,
    pub pointer: PointerInput,
    pub resync: ResyncInputEncoded,
}

#[derive(Resource)]
pub struct LocalPlayerHandle(pub PlayerHandle);

pub fn capture_and_encode_user_input(
    handle: In<PlayerHandle>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut local_player: ResMut<LocalPlayerHandle>,
    config: Res<crate::config::Config>,
    torsos: Query<(&Transform, &Velocity, &FpsControllerInput, &OwningPlayer), With<crate::player::Torso>>,
    mut sync_target: Local<u8>,
) -> PlayerInput {
    local_player.0 = handle.0;

    let mut input = PlayerInput::default();

    for (transform, velocity, controller, OwningPlayer(player)) in torsos.iter() {
        if *player != local_player.0 {
            continue;
        }

        // TODO: Rotate between sending translation, rotation, etc.
        let resync = match *sync_target {
            0 => {
                let Vec3 { x, y, z } = transform.translation;
                ResyncInput::Translation { x, y, z }
            },
            1 => {
                ResyncInput::Rotation { yaw: controller.yaw, pitch: controller.pitch, roll: 0.0 }
            },
            2 => {
                let Vec3 { x, y, z } = velocity.linvel;
                ResyncInput::Velocity { x, y, z }
            },
            3 => {
                let Vec3 { x, y, z } = velocity.angvel;
                ResyncInput::AngularVelocity { yaw: x, pitch: y, roll: z }
            },
            _ => continue
        };

        input.resync = resync.into();
    }

    *sync_target = (*sync_target + 1) % 4;

    for action in all::<UserAction>() {
        let pressed = match config.controls.input_for(action) {
            crate::config::UserInput::Keyboard(key) => keyboard_input.pressed(*key),
            crate::config::UserInput::Mouse(button) => mouse_input.pressed(*button),
        };

        if pressed {
            input.buttons.set(action, true);
        }
    }

    input.pointer = mouse_events
        .iter()
        .map(|event| event.delta * config.controls.pointer_sensitivity / 40.0)
        .sum::<Vec2>()
        .into();

    input
}
