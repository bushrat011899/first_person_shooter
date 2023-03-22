use bevy::{input::mouse::MouseMotion, prelude::*};
use bytemuck::{Pod, Zeroable};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};
use enum_iterator::all;

pub use buttons::*;
pub use pointer::*;

use crate::config::UserAction;

mod buttons;
mod pointer;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Default, Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub buttons: ButtonInput,
    pub pointer: PointerInput,
}

pub fn capture_and_encode_user_input(
    _handle: In<PlayerHandle>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_events: EventReader<MouseMotion>,
    config: Res<crate::config::Config>,
) -> PlayerInput {
    let mut input = PlayerInput::default();

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
        .map(|event| event.delta)
        .sum::<Vec2>()
        .into();

    input
}
