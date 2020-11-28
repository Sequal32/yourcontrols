use std::{fs::File, path::PathBuf};

use super::util::{LoadError};
use bimap::BiHashMap;

const INTERACTION_EVENT_ID: u32 = 10000;
const BUTTON_EVENT_ID: u32 = 10001;

pub fn load_mapping_config(path: &PathBuf) -> Result<BiHashMap<String, u32>, LoadError> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(LoadError::FileError(e))
    };

    let data: BiHashMap<String, u32> = match serde_yaml::from_reader(file) {
        Ok(d) => d,
        Err(e) => return Err(LoadError::ParseError(e))
    };

    Ok(data)
}

pub struct HEvents {
    key_mapping: BiHashMap<String, u32>,
    button_mapping: BiHashMap<String, u32>,
    pub group_id: u32
}

impl HEvents {
    pub fn new(group_id: u32) -> Self {
        Self {
            key_mapping: BiHashMap::new(),
            button_mapping: BiHashMap::new(),
            group_id: group_id,
        }
    }

    pub fn load_from_config(&mut self, key_path: &PathBuf, button_path: &PathBuf) -> Result<(), LoadError> {
        self.key_mapping = load_mapping_config(key_path)?;
        self.button_mapping = load_mapping_config(button_path)?;

        Ok(())
    }

    pub fn process_event_data(&self, data: &simconnect::SIMCONNECT_RECV_EVENT) -> Option<&String> {
        unsafe {
            if data.uEventID == INTERACTION_EVENT_ID {

                return self.key_mapping.get_by_right(&data.dwData)

            } else if data.uEventID == BUTTON_EVENT_ID {

                return match self.button_mapping.get_by_right(&data.dwData) {
                    Some(name) => Some(name),
                    None => None
                }

            } else {
                return None
            }
        }
    }

    pub fn on_connected(&self, conn: &simconnect::SimConnector) {
        conn.map_client_event_to_sim_event(INTERACTION_EVENT_ID, "Custom.Event7777");
        conn.add_client_event_to_notification_group(self.group_id, INTERACTION_EVENT_ID, false);

        conn.map_client_event_to_sim_event(BUTTON_EVENT_ID, "Custom.Event7778");
        conn.add_client_event_to_notification_group(self.group_id, BUTTON_EVENT_ID, false);
    }

    pub fn get_number_defined(&self) -> usize {
        return self.button_mapping.len() + self.key_mapping.len()
    }
}