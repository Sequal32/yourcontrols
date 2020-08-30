mod app;
mod bytereader;
mod definitions;
mod interpolate;
mod simclient;
mod simconfig;
mod simserver;
mod syncdefs;

use app::{App, AppMessage};
use bytereader::{StructDataTypes, data_type_as_bool};
use definitions::Definitions;
use indexmap::IndexMap;
use interpolate::{InterpolateStruct};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use simclient::Client;
use simconfig::Config;
use simconnect;
use simserver::{TransferClient, ReceiveData};
use simserver::Server;
use std::{str::FromStr, net::Ipv4Addr};
use crossbeam_channel::{Receiver, Sender};

struct TransferStruct {
    tx: Sender<Value>,
    rx: Receiver<ReceiveData>,
    client: Box<dyn TransferClient>
}

fn start_server(port: u16) -> Result<TransferStruct, std::io::Error> {
    let mut server = Server::new();
    let (tx, rx) = server.start(port)?;

    Ok(TransferStruct {
        tx, rx, client: Box::new(server)
    })
}

fn start_client(ip: Ipv4Addr, port: u16) -> Result<TransferStruct, std::io::Error> {
    let client = Client::new();
    let (tx, rx) = client.start(ip, port)?;

    Ok(TransferStruct {
        tx, rx, client: Box::new(client)
    })
}

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

    let mut app_interface = App::setup();

    // Transfer
    let mut transfer_client: Option<TransferStruct> = None;

    // Whether to start a client or a server

    let mut has_control = false;
    let mut can_take_control = false;
    let mut should_sync = false;
    // Interpolation Vars //
    let mut interpolation = InterpolateStruct::new();
    interpolation.add_special_floats(&mut vec!["PLANE PITCH DEGREES".to_string(), "PLANE BANK DEGREES".to_string(), "PLANE HEADING DEGREES MAGNETIC".to_string()]);
    // Set data upon receipt
    let mut tick = 0;

    loop {
        if let Some(transfer_client) = transfer_client.as_mut() {
            let tx = &transfer_client.tx;
            let rx = &transfer_client.rx;

            let message = conn.get_next_message();
            // Simconnect message
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
                                should_send = has_control && tick % config.update_rate == 0;
                                interpolation.record_current(sim_data);
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

                        should_send = should_send && transfer_client.client.get_connected_count() > 0;

                        // Update position data
                        if should_send {
                            tx.send(json!({
                                "type": send_type,
                                "data": data_string,
                                "time": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                            })).expect("!");
                        }
                        if !has_control {
                            if let Some(updated_map) = interpolation.interpolate() {
                                let (size_bytes, data_pointer) = definitions.sim_vars.write_to_data(&updated_map);
                                conn.set_data_on_sim_object(0, 0, 0, 0, size_bytes as u32, data_pointer);
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
                                    tx.send(json!({
                                        "type": "relieve_control"
                                    })).expect("!");
                                } else if can_take_control {
                                    tx.send(json!({
                                        "type": "transfer_control"
                                    })).expect("!");
                                    has_control = true;
                                    transfer_control(&conn, has_control);
                                }
                            },

                            _ => ()
                        }
                    }
                }
                _ => ()
            };
            // Data from the person in control
            match rx.try_recv() {
                Ok(ReceiveData::Data(value)) => match value["type"].as_str().unwrap() {
                    "physics" => { // Interpolate position update
                        if !has_control {
                            interpolation.record_latest(serde_json::from_value(value["data"].clone()).unwrap());
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
                            has_control = true;
                            transfer_control(&conn, has_control);
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
        }

        // GUI
        match app_interface.rx.try_recv() {
            Ok(msg) => match msg {
                AppMessage::Server(port ) => {
                    match start_server(port) {
                        Ok(transfer) => {
                            app_interface.server_started(0);
                            transfer_client = Some(transfer);

                            has_control = true;
                            transfer_control(&conn, has_control);
                        },
                        Err(_) => app_interface.error("Could not start server!")
                    }
                }
                AppMessage::Connect(ip, port) => {
                    match start_client(ip, port) {
                        Ok(transfer) => {
                            app_interface.connected();
                            transfer_client = Some(transfer);

                            has_control = false;
                            transfer_control(&conn, has_control);
                        }
                        Err(_) => app_interface.error("Could not connect to the server!")
                    }
                }
                AppMessage::Disconnect => {
                    transfer_client.as_ref().unwrap().client.stop();
                    app_interface.disconnected();
                }
                AppMessage::TakeControl => {}
                AppMessage::RelieveControl => {}
            }
            Err(_) => {}
        }

        app_interface.step();

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
