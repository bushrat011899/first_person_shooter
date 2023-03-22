use bevy::prelude::*;

use super::{FpsController, FpsControllerInput, MoveMode};

/// A standard FPS controller with mouse and keyboard controls.
#[derive(Bundle, Default)]
pub struct FpsControllerBundle {
    pub input: FpsControllerInput,
    pub controller: FpsController,
    pub move_mode: MoveMode,
}
