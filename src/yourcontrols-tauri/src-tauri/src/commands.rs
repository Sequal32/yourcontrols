use std::{
    collections::HashMap,
    fs,
    sync::{self, Arc},
    thread, time,
};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use yourcontrols_net::{Client, Server};

use crate::{definitions, states, Result, AIRCRAFT_DEFINITIONS_PATH};

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

        let file_stem = match file_path.file_stem() {
            Some(f_s) => f_s.to_string_lossy(),
            None => continue,
        };

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

fn connect_to_sim(
    sim_connector: &mut simconnect::SimConnector,
    definitions: &mut definitions::Definitions,
) -> anyhow::Result<()> {
    *definitions = definitions::Definitions::new();

    let connected = cfg!(feature = "skip_sim_connect") || sim_connector.connect("YourControls");
    anyhow::ensure!(
        connected,
        "Could not connect to SimConnect! Is the sim running?"
    );

    log::info!("Successfully connected to SimConnect!");
    Ok(())
}

// AppMessage::StartServer {
//     username,
//     port,
//     is_ipv6,
//     method,
//     use_upnp,
// } => {

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
    transfer_client_state: State<'_, states::TransferClientState>,
    method: ConnectionMethod,
) -> Result<()> {
    let mut sim_connect_mutex = sim_connect_state.lock().unwrap();
    let mut definitions_mutex = definitions_state.lock().unwrap();

    connect_to_sim(&mut sim_connect_mutex.0, &mut definitions_mutex.0)?;

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

    log::info!("Starting server...");
    definitions_mutex.0.on_connected(&sim_connect_mutex.0)?;

    match method {
        ConnectionMethod::Relay => {
            let mut client = Box::new(Client::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                "0.0.0".to_string(),
                // config.conn_timeout,
                10,
            ));

            client.start_with_relay(false)?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(client));
            log::info!("Hosting started");

            // // client.start_with_relay(is_ipv6)
            // if let Err(e) = client.start_with_relay(false) {
            //     log::info!("Hosting could not start! Reason: {}", e);
            //     // TODO
            //     // app_interface.server_fail(&e.to_string());
            // } else {
            //     transfer_client = Some(client);
            //     log::info!("Hosting started");
            // }
        }
        ConnectionMethod::Direct | ConnectionMethod::CloudServer => {
            let mut server = Box::new(Server::new(
                // username.clone(),
                "username".to_string(),
                // updater.get_version().to_string(),
                "0.0.0".to_string(),
                // config.conn_timeout,
                10,
            ));

            match method {
                // ConnectionMethod::Direct => server.start(is_ipv6, port, use_upnp),
                ConnectionMethod::Direct => server.start(false, 25071, true),
                // ConnectionMethod::CloudServer => server.start_with_hole_punching(is_ipv6),
                ConnectionMethod::CloudServer => server.start_with_hole_punching(false),
                _ => panic!("Not implemented!"),
            }?;
            // .inspect_err(|e| log::info!("Could not start server! Reason: {}", e))?;

            *transfer_client_state.lock().unwrap() = Some(states::TransferClientWrapper(server));
            log::info!("Server started");

            // if let Err(e) = result {
            //     log::info!(target: "NETWORK", "Could not start server! Reason: {}", e);
            //     // TODO
            //     // app_interface.server_fail(&e.to_string());
            // } else {
            //     // Assign server as transfer client
            //     transfer_client = Some(server);
            //     log::info!(target: "NETWORK", "Server started");
            // }
        }
    };

    let transfer_client_state = Arc::clone(&transfer_client_state);
    tauri::async_runtime::spawn(async move {
        while transfer_client_state.lock().unwrap().is_some() {
            log::info!("Server running...");

            thread::sleep(time::Duration::from_secs(1));
        }
    });

    Ok(())
}
