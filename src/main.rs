mod bytereader;
mod definitions;
mod interpolate;
mod simclient;
mod simconfig;
mod simserver;
mod syncdefs;

use bytereader::{StructDataTypes, data_type_as_bool};
use chrono;
use definitions::Definitions;
use indexmap::IndexMap;
use interpolate::{interpolate_f64, interpolate_f64_degrees};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use simclient::Client;
use simconfig::Config;
use simconnect;
use simserver::{TransferClient, ReceiveData};
use simserver::Server;
use std::{str::FromStr, net::Ipv4Addr};

fn transfer_control(conn: &simconnect::SimConnector, has_control: bool) {
    conn.transmit_client_event(1, 1000, !has_control as u32, 5, 0);
    conn.transmit_client_event(1, 1001, !has_control as u32, 5, 0);
    conn.transmit_client_event(1, 1002, !has_control as u32, 5, 0);
}

type SimValue = IndexMap<String, StructDataTypes>;
fn main() {
    // Load configuration file
    let config = match Config::read_from_file("config.json") {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.write_to_file("config.json").expect("!");
            config
        }
    };
    
    // Set up sim connect
    let mut conn = simconnect::SimConnector::new();
    conn.connect("Simple Shared Cockpit");

    let mut definitions = Definitions::new(&conn);
    let bool_defs = definitions.map_bool_sync_events(&conn, "sync_bools.dat", 1);

    conn.request_data_on_sim_object(0, 0, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME);
    conn.request_data_on_sim_object(1, 1, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SECOND);

    conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
    conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
    conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    conn.map_client_event_to_sim_event(1003, "FREEZE_LATITUDE_LONGITUDE_TOGGLE");
    conn.map_client_event_to_sim_event(1004, "FREEZE_ALTITUDE_TOGGLE");
    conn.map_client_event_to_sim_event(1005, "FREEZE_ATTITUDE_TOGGLE");

    conn.map_client_event_to_sim_event(2000, "TOGGLE_WATER_RUDDER");
    conn.add_client_event_to_notification_group(1, 2000, false);

    // Whether to start a client or a server

    let mut has_control;
    let mut can_take_control = false;
    let transfer_client: Box<dyn TransferClient>;

    // Initial connection
    println!("Enter ip to connect or type s to start server: ");
    let result: String = text_io::read!("{}");
    let (tx, rx) = match result.len() {
        1 => { 
            let mut server = Server::new();

            let transfer = match server.start(config.port) {
                Ok((tx, rx)) => {
                    println!("Server started!");
                    has_control = true;
                    (tx, rx)
                },
                Err(err) => panic!("Could not start server! {:?}", err)
            };

            transfer_client = Box::new(server);
            transfer
        },
        _ => match Ipv4Addr::from_str(&result) {
            Ok(ip) => {
                let client = Client::new();

                let transfer = match client.start(ip, config.port) {
                    Ok((tx, rx)) => {
                        println!("Client connected!");
                        has_control = false;
                        (tx, rx)
                    },
                    Err(err) => panic!("Could not start client! {:?}", err)
                };

                transfer_client = Box::new(client);
                transfer
            },
            Err(_) => panic!("Invalid ip provided!")
        }
    };
    transfer_control(&conn, has_control);

    let mut should_sync = false;
    // Interpolation Vars //

    let mut instant = std::time::Instant::now();
    let mut last_pos_update: Option<SimValue> = None;
    let mut pos_update: Option<SimValue> = None;
    let mut current_pos: Option<SimValue> = None;
    // Set data upon receipt
    let mut interpolation_time = 0.0;
    let mut add_alpha = 0.0;
    let mut last_packet: Option<Value> = None;
    let mut tick = 0;
    
    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnect::DispatchResult::SimobjectData(data)) => {
                // Send data to clients or to server
                unsafe {
                    let send_type: &str;
                    let data_string: String;

                    tick += 1;

                    let mut should_send;

                    match (*data).dwDefineID {
                        0 => {
                            let sim_data: SimValue = definitions.sim_vars.read_from_dword(std::mem::transmute_copy(&(*data).dwData));
                            send_type = "physics";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                            current_pos = Some(sim_data);
                            should_send = has_control && tick % config.update_rate == 0;
                        },
                        1 => {
                            let sim_data: SimValue = bool_defs.read_from_dword(std::mem::transmute_copy(&(*data).dwData));
                            send_type = "sync_toggle";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                            definitions.record_boolean_values(sim_data);
                            should_send = should_sync;
                            should_sync = false;
                        },
                        _ => panic!("Not covered!")
                    };

                    should_send = should_send && transfer_client.get_connected_count() > 0;

                    // Update position data
                    if should_send {
                        tx.send(json!({
                            "type": send_type,
                            "data": data_string,
                            "time": chrono::Utc::now().timestamp_millis()
                        })).expect("!");
                    }
                    if !has_control {
                        match (&last_pos_update, &pos_update) {
                            (Some(last), Some(current)) => {
                                // Interpolate previous recorded position with previous update
                                let mut updated_map = SimValue::new();

                                let alpha = instant.elapsed().as_millis() as f64/interpolation_time + add_alpha;
    
                                for (key, value) in last {
                                    match value {
                                        StructDataTypes::I32(_) => {
                                            updated_map.insert(key.to_string(), *value);
                                        }
                                        StructDataTypes::F64(v) => {
                                            if let StructDataTypes::F64(current_value) = current[key] {
                                                let interpolated = match key.as_str() {
                                                    "pitch" | "bank" | "heading" => interpolate_f64_degrees(*v, current_value, alpha),
                                                    _ => interpolate_f64(*v, current_value, alpha)
                                                };
                                                updated_map.insert(key.to_string(), StructDataTypes::F64(interpolated));
                                            }
                                        }
                                        _ => ()
                                    }
                                }
                                let (size_bytes, data_pointer) = definitions.sim_vars.write_to_data(&updated_map);
                                conn.set_data_on_sim_object(0, 0, 0, 0, size_bytes as u32, data_pointer);
                                current_pos = Some(updated_map);

                                if alpha > config.conn_timeout {
                                    last_pos_update = None;
                                    has_control = true;
                                    transfer_control(&conn, true);
                                    can_take_control = false;
                                    println!("No packet received within the last {} seconds, taking control.", config.conn_timeout);
                                }
                            },
                            _ => ()
                        }
                    }
                }
            },
            // Exception occured
            Ok(simconnect::DispatchResult::Exception(data)) => {
                unsafe {
                    println!("{:?}", (*data).dwException);
                }
            },
            Ok(simconnect::DispatchResult::Event(data)) => {
                unsafe {
                    match (*data).uGroupID {
                        0 => {
                            tx.send(json!({
                                "type": "event",
                                "eventid": (*data).uEventID,
                                "data": (*data).dwData
                            })).expect("!");
                        },
                        1 => {
                            if has_control {
                                println!("RELIEVING CONTROL");
                                tx.send(json!({
                                    "type": "relieve_control"
                                })).expect("!");
                            } else if can_take_control {
                                println!("TAKING CONTROL");
                                has_control = true;
                                tx.send(json!({
                                    "type": "transfer_control"
                                })).expect("!");
                            }
                        },

                        _ => ()
                    }
                }
            }
            _ => ()
        };
        
        match rx.try_recv() {
            Ok(ReceiveData::Data(value)) => match value["type"].as_str().unwrap() {
                "physics" => { // Interpolate position update
                    if !has_control {
                        match last_packet {
                            Some(p) => {
                                let cache_interpolation_time = interpolation_time;
                                interpolation_time = (value["time"].as_i64().unwrap()-p["time"].as_i64().unwrap()) as f64;
                                add_alpha = (instant.elapsed().as_secs_f64() - cache_interpolation_time/1000.0)/interpolation_time;
                                if add_alpha < 0.0 {add_alpha = 0.0}
                                instant = std::time::Instant::now();
                                last_pos_update = current_pos.take();
                            },
                            _ => (),
                        }
                        pos_update = Some(serde_json::from_str(value["data"].as_str().unwrap()).unwrap());
                        last_packet = Some(value);
                    }
                },
                "sync_toggle" => { // Initial synchronize
                    if definitions.has_synced_bool_values() {
                        let data: SimValue = serde_json::from_str(&value["data"].as_str().unwrap()).unwrap();
                        for (key, value) in data {
                            let current = data_type_as_bool(value).unwrap();
                            // Synchronize
                            definitions.sync_boolean(&conn, &key, current);
                        }
                    }
                },
                "event" => {
                    let event_id = value["eventid"].as_u64().unwrap();
                    let data = value["data"].as_i64().unwrap();
                    conn.transmit_client_event(1, event_id as u32, data as u32, 0, 0);
                },
                "relieve_control" => {
                    println!("CAN TAKE CONTROLS");
                    can_take_control = true;
                },
                "transfer_control" => {
                    if has_control {
                        println!("CONTROLS GIVEN UP");
                        has_control = false;
                        can_take_control = false;
                        transfer_control(&conn, false);
                    }
                }
                _ => ()
            },
            Ok(ReceiveData::NewConnection(ip)) => {
                println!("NEW CONNECTION: {}", ip);
                should_sync = true;
            },
            _ => ()
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
