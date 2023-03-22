use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

mod controls;
mod graphics;
mod logging;
mod matchmaking;

pub use controls::*;
pub use graphics::*;
pub use logging::*;
pub use matchmaking::*;

#[derive(Serialize, Deserialize, Default, Resource)]
pub struct Config {
    pub matchmaking: MatchMakingSettings,
    pub controls: ControlBindings,
    pub graphics: GraphicsSettings,
    pub logging: LoggingSettings,
}

impl Config {
    const FILE_NAME: &str = "settings.json";

    pub fn try_load() -> Result<Self, &'static str> {
        log::trace!("Loading Settings from '{}'", Self::FILE_NAME);

        let file = File::open(Self::FILE_NAME).map_err(|_| "Cannot Open Settings File")?;

        let buf_reader = BufReader::new(file);

        let config = serde_json::from_reader(buf_reader).map_err(|_| "Cannot Parse Settings")?;

        Ok(config)
    }

    pub fn try_save(&self) -> Result<(), &'static str> {
        log::trace!("Saving Settings to '{}'", Self::FILE_NAME);

        let file = File::create(Self::FILE_NAME).map_err(|_| "Cannot Create Settings File")?;

        let buf_writer = BufWriter::new(file);

        serde_json::to_writer(buf_writer, self).map_err(|_| "Cannot Write Settings File")?;

        Ok(())
    }
}
