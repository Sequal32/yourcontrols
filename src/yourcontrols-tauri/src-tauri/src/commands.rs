use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::{states, Result, AIRCRAFT_DEFINITIONS_PATH};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct AircraftConfig {
    name: String,
    path: String,
}

#[tauri::command]
#[specta::specta]
pub fn get_aircraft_configs() -> Result<HashMap<String, Vec<AircraftConfig>>> {
    let mut aircraft_configs: HashMap<String, Vec<AircraftConfig>> = HashMap::new();

    let files = fs::read_dir(AIRCRAFT_DEFINITIONS_PATH)?;

    for file in files {
        let file_path = file?.path();

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

        if let Some(v) = aircraft_configs.get_mut(new_category) {
            v.push(new_aircraft_config);
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
) -> Result<()> {
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

#[derive(Serialize, Deserialize, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMethod {
    Direct,
    Relay,
    CloudServer,
}

// AppMessage::StartServer {
//     username,
//     port,
//     is_ipv6,
//     method,
//     use_upnp,
// } => {
// let connect_to_sim = |conn: &mut SimConnector, definitions: &mut Definitions| {
//     // Connect to simconnect
//     *definitions = Definitions::new();
//     #[cfg(not(feature = "skip_sim_connect"))]
//     let connected = conn.connect("YourControls");
//     #[cfg(feature = "skip_sim_connect")]
//     let connected = true;
//     if connected {
//         // Display not connected to server message
//         info!("[SIM] Connected to SimConnect.");
//     } else {
//         // Display trying to connect message
//         app_interface.error("Could not connect to SimConnect! Is the sim running?");
//     };

//     connected
// };

//     let connected = connect_to_sim(&mut conn, &mut definitions);

//     if config_to_load.is_empty() {
//         app_interface.server_fail("Select an aircraft config first!");
//     } else if !load_definitions(&mut definitions, &mut config_to_load) {
//         app_interface.error(
//             "Error loading definition files. Check the log for more information.",
//         );
//     } else if connected {
//         definitions.on_connected(&conn).ok();
//         control.on_connected(&conn);
//         // Display attempting to start server
//         app_interface.attempt();

//         match method {
//             ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
//                 let mut server = Box::new(Server::new(
//                     username.clone(),
//                     updater.get_version().to_string(),
//                     config.conn_timeout,
//                 ));

//                 let result = match method {
//                     ConnectionMethod::Direct => {
//                         server.start(is_ipv6, port, use_upnp)
//                     }
//                     ConnectionMethod::CloudServer => {
//                         server.start_with_hole_punching(is_ipv6)
//                     }
//                     _ => panic!("Not implemented!"),
//                 };

//                 match result {
//                     Ok(_) => {
//                         // Assign server as transfer client
//                         transfer_client = Some(server);
//                         info!("[NETWORK] Server started");
//                     }
//                     Err(e) => {
//                         app_interface.server_fail(&e.to_string());
//                         info!("[NETWORK] Could not start server! Reason: {}", e);
//                     }
//                 }
//             }
//             ConnectionMethod::Relay => {
//                 let mut client = Box::new(Client::new(
//                     username.clone(),
//                     updater.get_version().to_string(),
//                     config.conn_timeout,
//                 ));

//                 match client.start_with_relay(is_ipv6) {
//                     Ok(_) => {
//                         transfer_client = Some(client);
//                         info!("[NETWORK] Hosting started");
//                     }
//                     Err(e) => {
//                         info!("[NETWORK] Hosting could not start! Reason: {}", e);
//                         app_interface.server_fail(&e.to_string());
//                     }
//                 }
//             }
//         };

//         config.port = port;
//         config.name = username;
//         write_configuration(&config);
//     }
// }

#[tauri::command]
#[specta::specta]
pub fn start_server(
    sim_connect_state: State<'_, states::SimConnectorState>,
    definitions_state: State<'_, states::DefinitionsState>,
    settings_state: State<'_, states::SettingsState>,
    method: ConnectionMethod,
) -> Result<()> {
    let mut sim_connect_mutex = sim_connect_state.lock().unwrap();
    let mut definitions_mutex = definitions_state.lock().unwrap();

    *definitions_mutex = states::DefinitionsWrapper::new();

    let connected =
        cfg!(feature = "skip_sim_connect") || sim_connect_mutex.0.connect("YourControls");

    if connected {
        log::info!("[SIM] Connected to SimConnect.");
    } else {
        // TODO
        // app_interface.error("Could not connect to SimConnect! Is the sim running?");
    };

    let _settings = { settings_state.lock().unwrap().clone() };

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
            .on_connected(&sim_connect_mutex.0)
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

    Ok(())
}
