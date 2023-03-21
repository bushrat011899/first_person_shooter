use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use ggrs::PlayerHandle;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable, Default)]
pub struct PlayerInput {
    pub inp: u8,
}

impl PlayerInput {
    const INPUT_UP: u8 = 1 << 0;
    const INPUT_DOWN: u8 = 1 << 1;
    const INPUT_LEFT: u8 = 1 << 2;
    const INPUT_RIGHT: u8 = 1 << 3;

    fn get(&self, mask: u8) -> bool {
        self.inp & mask == 1
    }

    fn set(&mut self, state: bool, mask: u8) {
        if state {
            self.inp |= mask;
        } else {
            self.inp &= !mask;
        }
    }

    pub fn get_up(&self) -> bool {
        self.get(Self::INPUT_UP)
    }

    pub fn set_up(&mut self, state: bool) {
        self.set(state, Self::INPUT_UP);
    }

    pub fn get_down(&self) -> bool {
        self.get(Self::INPUT_DOWN)
    }

    pub fn set_down(&mut self, state: bool) {
        self.set(state, Self::INPUT_DOWN);
    }

    pub fn get_left(&self) -> bool {
        self.get(Self::INPUT_LEFT)
    }

    pub fn set_left(&mut self, state: bool) {
        self.set(state, Self::INPUT_LEFT);
    }

    pub fn get_right(&self) -> bool {
        self.get(Self::INPUT_RIGHT)
    }

    pub fn set_right(&mut self, state: bool) {
        self.set(state, Self::INPUT_RIGHT);
    }
}

pub fn input(_handle: In<PlayerHandle>, keyboard_input: Res<Input<KeyCode>>) -> PlayerInput {
    let mut input = PlayerInput::default();

    if keyboard_input.pressed(KeyCode::W) {
        input.set_up(true);
    }
    if keyboard_input.pressed(KeyCode::A) {
        input.set_left(true);
    }
    if keyboard_input.pressed(KeyCode::S) {
        input.set_down(true);
    }
    if keyboard_input.pressed(KeyCode::D) {
        input.set_right(true);
    }

    input
}
