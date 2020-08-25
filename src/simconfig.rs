use serde_json;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Write, BufReader};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub update_rate: u16,
    pub conn_timeout: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 7777,
            update_rate: 10,
            conn_timeout: 10.0
        }
    }
}

impl Config {
    pub fn write_to_file(&self, filename: &str) -> Result<(), &str> {
        let mut file = match File::create(filename) {
            Ok(file) => file,
            Err(_) => {return Err("Could not open configuration file.");}
        };

        match serde_json::to_string_pretty(self) {
            Ok(data_string) => match file.write(data_string.as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err("Could not write to configuration file.")
            },
            Err(_) => Err("Could not serialize structure!")
        }
    }

    pub fn read_from_file(filename: &str) -> Result<Self, &str> {
        let file = match File::open(filename) {
            Ok(file) => file,
            Err(_) => {return Err("Could not open configuration file.");}
        };
        let reader = BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(data) => Ok(data),
            Err(_) => Err("Configuration file corrupted.")
        }
    }
}