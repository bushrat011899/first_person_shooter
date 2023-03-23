use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::prelude::Velocity;
use bytemuck::{Pod, Zeroable};
use enum_iterator::all;
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};

pub use buttons::*;
pub use pointer::*;
pub use resync::*;

use crate::{config::UserAction, player::OwningPlayer};

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
    torsos: Query<(&Transform, &Velocity, &OwningPlayer), With<crate::player::Torso>>,
) -> PlayerInput {
    local_player.0 = handle.0;

    let mut input = PlayerInput::default();

    for (transform, velocity, OwningPlayer(player)) in torsos.iter() {
        if *player != local_player.0 {
            continue;
        }

        // TODO: Rotate between sending translation, rotation, etc.
        input.resync = ResyncInput::Translation { x: transform.translation.x, y: transform.translation.y, z: transform.translation.z }.into();
    }

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
