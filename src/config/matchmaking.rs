use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchMakingSettings {
    pub server: String,
    pub room: String,
    pub players: NonZeroUsize,
}

impl Default for MatchMakingSettings {
    fn default() -> Self {
        Self {
            server: "wss://matchbox-muskrats.fly.dev:443".to_owned(),
            room: "default_room".to_owned(),
            players: NonZeroUsize::new(4).unwrap(),
        }
    }
}

impl MatchMakingSettings {
    pub const TICK_RATE: u16 = 60;

    pub const fn tick_rate(&self) -> u16 {
        Self::TICK_RATE
    }
}
