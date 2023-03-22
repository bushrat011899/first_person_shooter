use bevy::prelude::*;

/// Represents if the player is using the free-look feature.
#[derive(Clone, Copy)]
pub enum FreeLookState {
    /// The player is not using free-look (default)
    Not,
    /// The player has just started free-looking
    Starting,
    /// The player has been free-looking for an indefinite amount of time
    Looking,
    /// The player has just stopped free-looking
    Stopping,
}

/// Component describing desired player inputs in a device-agnostic way.
#[derive(Component, Default)]
pub struct FpsControllerInput {
    pub fly: bool,
    pub sprint: bool,
    pub jump: bool,
    pub crouch: bool,
    pub free_look: FreeLookState,
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
}

impl Default for FreeLookState {
    fn default() -> Self {
        Self::Not
    }
}

impl FreeLookState {
    pub fn evolve(self, free_looking: bool) -> Self {
        match (self, free_looking) {
            (Self::Stopping, true) | (Self::Not, true) => Self::Starting,
            (Self::Stopping, false) | (Self::Not, false) => Self::Not,
            (Self::Starting, true) | (Self::Looking, true) => Self::Looking,
            (Self::Starting, false) | (Self::Looking, false) => Self::Stopping,
        }
    }
}
