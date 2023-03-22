use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Info,
    Debug,
    Warn,
    Error,
    Off,
}

impl From<log::LevelFilter> for LogLevel {
    fn from(value: log::LevelFilter) -> Self {
        match value {
            log::LevelFilter::Off => Self::Off,
            log::LevelFilter::Error => Self::Error,
            log::LevelFilter::Warn => Self::Warn,
            log::LevelFilter::Info => Self::Info,
            log::LevelFilter::Debug => Self::Debug,
            log::LevelFilter::Trace => Self::Trace,
        }
    }
}

impl Into<log::LevelFilter> for LogLevel {
    fn into(self) -> log::LevelFilter {
        match self {
            Self::Trace => log::LevelFilter::Trace,
            Self::Info => log::LevelFilter::Info,
            Self::Debug => log::LevelFilter::Debug,
            Self::Warn => log::LevelFilter::Warn,
            Self::Error => log::LevelFilter::Error,
            Self::Off => log::LevelFilter::Off,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggingSettings {
    pub level: LogLevel,
    pub overrides: HashMap<String, LogLevel>,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        let mut overrides = HashMap::new();

        overrides.insert("first_person_shooter".to_owned(), LogLevel::Info);

        Self {
            level: LogLevel::Warn,
            overrides,
        }
    }
}
