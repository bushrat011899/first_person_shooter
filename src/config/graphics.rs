use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Msaa {
    Off,
    X2,
    X4,
    X8
}

impl From<bevy::prelude::Msaa> for Msaa {
    fn from(value: bevy::prelude::Msaa) -> Self {
        match value {
            bevy::prelude::Msaa::Off => Self::Off,
            bevy::prelude::Msaa::Sample2 => Self::X2,
            bevy::prelude::Msaa::Sample4 => Self::X4,
            bevy::prelude::Msaa::Sample8 => Self::X8,
        }
    }
}

impl Into<bevy::prelude::Msaa> for Msaa {
    fn into(self) -> bevy::prelude::Msaa {
        match self {
            Msaa::Off => bevy::prelude::Msaa::Off,
            Msaa::X2 => bevy::prelude::Msaa::Sample2,
            Msaa::X4 => bevy::prelude::Msaa::Sample4,
            Msaa::X8 => bevy::prelude::Msaa::Sample8,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub msaa: Msaa,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            msaa: Msaa::X4
        }
    }
}
