use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};
use half::bf16;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default, Serialize, Deserialize, Debug)]
pub struct PointerInput {
    delta_x: [u8; 2],
    delta_y: [u8; 2],
}

impl Eq for PointerInput {}

impl From<Vec2> for PointerInput {
    fn from(delta: Vec2) -> Self {
        Self {
            delta_x: bf16::from_f32(delta.x).to_be_bytes(),
            delta_y: bf16::from_f32(delta.y).to_be_bytes(),
        }
    }
}

impl Into<Vec2> for PointerInput {
    fn into(self) -> Vec2 {
        Vec2 {
            x: bf16::from_be_bytes(self.delta_x).into(),
            y: bf16::from_be_bytes(self.delta_y).into(),
        }
    }
}
