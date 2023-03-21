use std::io::BufWriter;
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub matchmaking_server: SocketAddr,
    pub controls: ControlBindings,
    pub graphics: GraphicsSettings
}

impl Config {
    const FILE_NAME: &str = "settings.toml";

    pub fn try_load() -> Result<Self, ()> {
        let file = File::open(Self::FILE_NAME).map_err(|_| ())?;
    
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
    
        buf_reader.read_to_string(&mut contents).map_err(|_| ())?;
        
        let config = toml::from_str(contents.as_str()).map_err(|_| ())?;
    
        Ok(config)
    }

    pub fn try_save(&self) -> Result<(), ()> {
        let contents = toml::to_string(self).map_err(|_| ())?;
    
        let file = File::create(Self::FILE_NAME).map_err(|_| ())?;
    
        let mut buf_writer = BufWriter::new(file);
    
        buf_writer.write(contents.as_bytes()).map_err(|_| ())?;
        buf_writer.flush().map_err(|_| ())?;
    
        Ok(())
    }
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
