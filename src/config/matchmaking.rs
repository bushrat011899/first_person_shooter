use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchMakingSettings {
    pub server: String,
}

impl Default for MatchMakingSettings {
    fn default() -> Self {
        Self {
            server: "ws://localhost:3536".to_owned(),
        }
    }
}

impl MatchMakingSettings {
    pub const TICK_RATE: u16 = 60;

    pub const fn tick_rate(&self) -> u16 {
        Self::TICK_RATE
    }
}
