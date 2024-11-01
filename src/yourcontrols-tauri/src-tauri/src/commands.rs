use std::{collections::HashMap, fs::read_dir};

use log::{error, info};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::AIRCRAFT_DEFINITIONS_PATH;

#[derive(Serialize, Deserialize, Type)]
pub struct AircraftConfig {
    name: String,
    path: String,
}

#[tauri::command]
#[specta::specta]
// TODO: remove unwraps
pub fn get_aircraft_configs() -> Result<HashMap<String, Vec<AircraftConfig>>, String> {
    let mut aircraft_configs: HashMap<String, Vec<AircraftConfig>> = HashMap::new();

    for file in read_dir(AIRCRAFT_DEFINITIONS_PATH).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;

        let file_path = file.path();
        let config = file_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split(" - ")
            .collect::<Vec<&str>>();

        if config.len() != 2 {
            continue;
        }

        let new_aircraft_config = AircraftConfig {
            name: config[1].to_string(),
            path: file_path.to_str().unwrap().to_string(),
        };

        if aircraft_configs.contains_key(config[0]) {
            aircraft_configs
                .get_mut(config[0])
                .unwrap()
                .push(new_aircraft_config);
        } else {
            aircraft_configs.insert(config[0].to_string(), vec![new_aircraft_config]);
        }
    }

    Ok(aircraft_configs)
}

#[tauri::command]
#[specta::specta]
pub fn save_settings(
    username: String,
    aircraft: String,
    instructor_mode: bool,
    streamer_mode: bool,
) -> Result<(), String> {
    info!(
        "{} {} {} {}",
        username, aircraft, instructor_mode, streamer_mode
    );

    error!("save_settings: Not yet implemented!");

    Ok(())
}
