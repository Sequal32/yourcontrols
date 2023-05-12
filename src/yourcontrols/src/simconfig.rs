use derive_more::{Display, From};
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
    pub update_rate: u16,
    pub conn_timeout: u64,
    pub check_for_betas: bool,
    pub port: u16,
    pub ip: String,
    pub name: String,
    pub ui_dark_theme: bool,
    pub streamer_mode: bool,
    pub instructor_mode: bool,
    pub sound_muted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 25071,
            update_rate: 30,
            conn_timeout: 3,
            check_for_betas: false,
            ip: String::new(),
            name: String::new(),
            ui_dark_theme: true,
            streamer_mode: false,
            instructor_mode: false,
            sound_muted: false,
        }
    }
}

impl Config {
    pub fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), ConfigLoadError> {
        let data_string = serde_json::to_string_pretty(self)?;

        let mut file = File::create(path)?;
        file.write_all(data_string.as_bytes())?;

        Ok(())
    }

    pub fn read_from_file(filename: impl AsRef<std::path::Path>) -> Result<Self, ConfigLoadError> {
        let file = File::open(filename)?;

        let config = serde_json::from_reader(file)?;

        Ok(config)
    }

    pub fn get_json_string(&self) -> String {
        serde_json::to_value(self).unwrap().to_string()
    }
}
