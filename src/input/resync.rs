use bytemuck::{Pod, Zeroable};
use half::bf16;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Serialize, Deserialize, Debug)]
pub struct ResyncInputEncoded {
    discriminant: u8,
    payload: [u8; 6],
}

impl Default for ResyncInputEncoded {
    fn default() -> Self {
        ResyncInput::BadData.into()
    }
}

impl Eq for ResyncInputEncoded {}

/// Represents a piece of synchronisation information.
/// Any information which could drift out of sync over time can be added here for transmission along with player input.
/// How to act on this information, and when to send it, is left up to the caller.
pub enum ResyncInput {
    Translation { x: f32, y: f32, z: f32 },

    Rotation { yaw: f32, pitch: f32, roll: f32 },

    Velocity { x: f32, y: f32, z: f32 },

    AngularVelocity { yaw: f32, pitch: f32, roll: f32 },

    /// Fallback
    BadData,
}

impl From<ResyncInput> for ResyncInputEncoded {
    fn from(value: ResyncInput) -> Self {
        let (discriminant, a, b, c) = match value {
            ResyncInput::Translation { x, y, z } => (0, x, y, z),
            ResyncInput::Rotation { yaw, pitch, roll } => (1, yaw, pitch, roll),
            ResyncInput::Velocity { x, y, z } => (2, x, y, z),
            ResyncInput::AngularVelocity { yaw, pitch, roll } => (3, yaw, pitch, roll),
            // Fallback
            ResyncInput::BadData => (u8::MAX, 0., 0., 0.),
        };

        let (a, b, c) = (
            bf16::from_f32(a).to_be_bytes(),
            bf16::from_f32(b).to_be_bytes(),
            bf16::from_f32(c).to_be_bytes(),
        );

        let payload = [a[0], a[1], b[0], b[1], c[0], c[1]];

        Self {
            discriminant,
            payload,
        }
    }
}

impl Into<ResyncInput> for ResyncInputEncoded {
    fn into(self) -> ResyncInput {
        let Self {
            discriminant,
            payload,
        } = self;

        let (a, b, c) = (
            bf16::from_be_bytes([payload[0], payload[1]]).into(),
            bf16::from_be_bytes([payload[2], payload[3]]).into(),
            bf16::from_be_bytes([payload[4], payload[5]]).into(),
        );

        match discriminant {
            0 => ResyncInput::Translation { x: a, y: b, z: c },
            1 => ResyncInput::Rotation {
                yaw: a,
                pitch: b,
                roll: c,
            },
            2 => ResyncInput::Velocity { x: a, y: b, z: c },
            3 => ResyncInput::AngularVelocity {
                yaw: a,
                pitch: b,
                roll: c,
            },
            // Fallback
            _ => ResyncInput::BadData,
        }
    }
}
