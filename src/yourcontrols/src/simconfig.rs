use derive_more::{Display, From};
use log::error;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{convert::AsRef, fs::File, io};

#[derive(From, Display)]
pub enum ConfigLoadError {
    FileError(io::Error),
    SerializeError(serde_json::Error),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub conn_timeout: u64,
    pub port: u16,
    pub ip: String,
    pub name: String,
    pub ui_dark_theme: bool,
    pub streamer_mode: bool,
    pub instructor_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 25071,
            conn_timeout: 5,
            ip: String::new(),
            name: String::new(),
            ui_dark_theme: true,
            streamer_mode: false,
            instructor_mode: false,
        }
    }
}

const CONFIG_FILENAME: &str = "config.json";

impl Config {
    fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), ConfigLoadError> {
        let data_string = serde_json::to_string_pretty(self)?;

        let mut file = File::create(path)?;
        file.write_all(data_string.as_bytes())?;

        Ok(())
    }

    fn read_from_file(filename: impl AsRef<std::path::Path>) -> Result<Self, ConfigLoadError> {
        let file = File::open(filename)?;

        let config = serde_json::from_reader(file)?;

        Ok(config)
    }

    pub fn read() -> Result<Self, ConfigLoadError> {
        Self::read_from_file(CONFIG_FILENAME)
    }

    pub fn write(&self) {
        if let Err(e) = self.write_to_file(CONFIG_FILENAME) {
            error!("Could not write configuration to file: {}", e);
        }
    }

    pub fn read_or_default() -> Self {
        match Self::read() {
            Ok(config) => config,
            Err(e) => {
                error!("Could not read configuration file, using default: {}", e);
                Self::default()
            }
        }
    }

    pub fn get_json_string(&self) -> String {
        serde_json::to_value(self).unwrap().to_string()
    }
}
