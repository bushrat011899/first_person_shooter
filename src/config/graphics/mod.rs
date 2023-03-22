use bevy::window::WindowMode;
use serde::{Deserialize, Serialize};

pub use msaa::*;
pub use particles::*;

mod msaa;
mod particles;

#[derive(Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub vsync: bool,
    pub mode: WindowMode,
    pub width: usize,
    pub height: usize,
    pub msaa: Msaa,
    pub particles: ParticleDetail,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            mode: WindowMode::Windowed,
            width: 1280,
            height: 720,
            msaa: Msaa::X4,
            particles: ParticleDetail::High,
        }
    }
}
