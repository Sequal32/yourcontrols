use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{states, AIRCRAFT_DEFINITIONS_PATH};

#[derive(Serialize, Deserialize, specta::Type, Debug)]
pub struct AircraftConfig {
    name: String,
    path: String,
}

#[tauri::command]
#[specta::specta]
pub fn get_aircraft_configs() -> Result<HashMap<String, Vec<AircraftConfig>>, String> {
    let mut aircraft_configs: HashMap<String, Vec<AircraftConfig>> = HashMap::new();

    let files = fs::read_dir(AIRCRAFT_DEFINITIONS_PATH).map_err(|e| e.to_string())?;

    for file in files {
        let file_path = file.map_err(|e| e.to_string())?.path();

        // Skip non-YAML files
        if file_path.extension().is_none_or(|v| v != "yaml") {
            continue;
        }

        let file_stem = file_path.file_stem().unwrap().to_string_lossy();
        let config: Vec<&str> = file_stem.split(" - ").collect();

        let mut new_category = "Other";
        let mut new_aircraft_config = AircraftConfig {
            name: file_stem.to_string(),
            path: file_path.to_string_lossy().to_string(),
        };

        if config.len() == 2 {
            new_category = config[0];
            new_aircraft_config.name = config[1].to_string();
        }

        if aircraft_configs.contains_key(new_category) {
            aircraft_configs
                .get_mut(new_category)
                .unwrap()
                .push(new_aircraft_config);
        } else {
            aircraft_configs.insert(new_category.to_string(), vec![new_aircraft_config]);
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
    log::info!(
        "{} {} {} {}",
        username,
        aircraft,
        instructor_mode,
        streamer_mode
    );

    log::error!("save_settings: Not yet implemented!");

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMethod {
    Direct,
    Relay,
    CloudServer,
}

#[tauri::command]
#[specta::specta]
pub fn start_server(
    sim_connect_state: State<'_, states::SimConnectorState>,
    definitions_state: State<'_, states::DefinitionsState>,
    settings_state: State<'_, states::SettingsState>,
    method: ConnectionMethod,
) {
    let connected = {
        let mut sim_connect_mutex = sim_connect_state.lock().unwrap();
        *definitions_state.lock().unwrap() = states::DefinitionsWrapper::new();

        let conn =
            cfg!(feature = "skip_sim_connect") || sim_connect_mutex.0.connect("YourControls");

        if conn {
            log::info!("[SIM] Connected to SimConnect.");
        } else {
            // TODO
            // app_interface.error("Could not connect to SimConnect! Is the sim running?");
        };

        conn
    };

    let settings = { settings_state.lock().unwrap().clone() };

    // if config_to_load.is_empty() {
    //     app_interface.server_fail("Select an aircraft config first!");
    // } else if !load_definitions(&mut definitions, &mut config_to_load) {
    //     app_interface.error(
    //         "Error loading definition files. Check the log for more information.",
    //     );
    // } else if connected {
    //     definitions.on_connected(&conn).ok();
    //     control.on_connected(&conn);
    //     // Display attempting to start server
    //     app_interface.attempt();

    if connected {
        log::info!("[NETWORK] Starting server...");

        definitions_state
            .lock()
            .unwrap()
            .0
            .on_connected(&sim_connect_state.lock().unwrap().0)
            .ok();
    }

    match method {
        ConnectionMethod::Relay => {
            // let mut client = Box::new(Client::new(
            //     username.clone(),
            //     updater.get_version().to_string(),
            //     config.conn_timeout,
            // ));

            // match client.start_with_relay(is_ipv6) {
            //     Ok(_) => {
            //         transfer_client = Some(client);
            //         log::info!("[NETWORK] Hosting started");
            //     }
            //     Err(e) => {
            //         log::info!("[NETWORK] Hosting could not start! Reason: {}", e);
            //         app_interface.server_fail(&e.to_string());
            //     }
            // }

            log::error!("Relay not yet implemented!");
        }
        ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
            // let mut server = Box::new(Server::new(
            //     username.clone(),
            //     updater.get_version().to_string(),
            //     config.conn_timeout,
            // ));

            // let result = match method {
            //     ConnectionMethod::Direct => server.start(is_ipv6, port, use_upnp),
            //     ConnectionMethod::CloudServer => server.start_with_hole_punching(is_ipv6),
            //     _ => panic!("Not implemented!"),
            // };

            // match result {
            //     Ok(_) => {
            //         // Assign server as transfer client
            //         transfer_client = Some(server);
            //         log::info!("[NETWORK] Server started");
            //     }
            //     Err(e) => {
            //         app_interface.server_fail(&e.to_string());
            //         log::info!("[NETWORK] Could not start server! Reason: {}", e);
            //     }
            // }

            log::error!("Relay not yet implemented!");
        }
    };

    //     config.port = port;
    //     config.name = username;
    //     write_configuration(&config);
    // }

    log::error!("start_server: Not yet implemented!");
}
