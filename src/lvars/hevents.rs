use bimap::BiHashMap;
use serde_yaml;
use std::{fmt::Display, fs::File};

use super::LVars;

pub enum LoadError {
    FileError(std::io::Error),
    ParseError(serde_yaml::Error)
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileError(e) => write!(f, "Could not read file: {}", e),
            LoadError::ParseError(e) => write!(f, "Could not parse file: {}", e)
        }
    }
}

pub struct HEvents {
    mapping: BiHashMap<String, u32>,
    pub group_id: u32
}

impl HEvents {
    pub fn new(group_id: u32) -> Self {
        Self {
            mapping: BiHashMap::new(),
            group_id: group_id,
        }
    }

    pub fn load_from_config(&mut self, path: &str) -> Result<(), LoadError> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(LoadError::FileError(e))
        };

        let data: BiHashMap<String, u32> = match serde_yaml::from_reader(file) {
            Ok(d) => d,
            Err(e) => return Err(LoadError::ParseError(e))
        };

        self.mapping = data;

        Ok(())
    }

    pub fn process_event_data(&self, data: &simconnect::SIMCONNECT_RECV_EVENT) -> Option<&String> {
        unsafe {
            return self.mapping.get_by_right(&data.dwData)
        }
    }

    pub fn on_connected(&self, conn: &simconnect::SimConnector) {
        conn.map_client_event_to_sim_event(10000, "Custom.Event7777");
        conn.add_client_event_to_notification_group(self.group_id, 10000, false);
    }

    pub fn get_number_defined(&self) -> usize {
        return self.mapping.len()
    }
}