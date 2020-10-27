use serde_json;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Write, BufReader};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub update_rate: u16,
    pub conn_timeout: u64,
    pub buffer_size: usize,
    pub check_for_betas: bool,
    pub port: u16,
    pub ip: String,
    pub name: String,
    pub last_config: String
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 7777,
            update_rate: 30,
            buffer_size: 3,
            conn_timeout: 5,
            check_for_betas: false,
            ip: String::new(),
            name: String::new(),
            last_config: String::new()
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