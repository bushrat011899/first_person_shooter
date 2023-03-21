use std::net::SocketAddr;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub fn try_load_config() -> Result<Config, ()> {
    let file = File::open("Settings.toml").map_err(|_| ())?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).map_err(|_| ())?;
    
    let config = toml::from_str(contents.as_str()).map_err(|_| ())?;

    Ok(config)
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub matchmaking_server: SocketAddr,
    pub controls: ControlBindings
}

#[derive(Serialize, Deserialize)]
pub struct ControlBindings {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub jump: String,
    pub crouch: String,
    pub sprint: String,
    pub ram: String,
    pub pour: String,
    pub load: String,
    pub fire: String,
}

#[derive(Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub msaa: u8,
}
