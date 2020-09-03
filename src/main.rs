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
use simclient::Client;
use simconfig::Config;
use simconnect;
use simserver::{TransferClient, ReceiveData};
use simserver::Server;
use std::{time::{Duration, Instant}, net::Ipv4Addr, io::Error};
use crossbeam_channel::{Receiver, Sender};

struct TransferStruct {
    tx: Sender<Value>,
    rx: Receiver<ReceiveData>,
    client: Box<dyn TransferClient>
}

fn start_server(port: u16) -> Result<TransferStruct, Error> {
    let mut server = Server::new();
    let (tx, rx) = server.start(port)?;

    Ok(TransferStruct {
        tx, rx, client: Box::new(server)
    })
}

fn start_client(ip: Ipv4Addr, port: u16) -> Result<TransferStruct, Error> {
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
const config_filename: &str = "config.json";
fn main() {
    // Load configuration file
    let mut config = match Config::read_from_file(config_filename) {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            config.write_to_file(config_filename).expect(format!("Could not write to {}!", config_filename).as_str());
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

    //
    let mut update_rate_instant = Instant::now();
    let update_rate = 1.0 / config.update_rate as f64;
    // Whether to start a client or a server

    let mut has_control = false;
    let mut can_take_control = false;
    let mut should_sync = false;
    let mut was_error = false;
    let mut time_since_control = Instant::now();
    // Interpolation Vars //
    let mut interpolation = InterpolateStruct::new(update_rate);
    interpolation.add_special_floats_regular(&mut vec!["PLANE HEADING DEGREES MAGNETIC".to_string()]);
    interpolation.add_special_floats_wrap90(&mut vec!["PLANE PITCH DEGREES".to_string()]);
    interpolation.add_special_floats_wrap180(&mut vec!["PLANE BANK DEGREES".to_string()]);

    app_interface.set_ip(config.ip.as_str());
    app_interface.set_port(config.port);

    loop {
        if let Some(client) = transfer_client.as_mut() {
            let tx = &client.tx;
            let rx = &client.rx;

            let message = conn.get_next_message();
            // Simconnect message
            match message {
                Ok(simconnect::DispatchResult::SimobjectData(data)) => {
                    // Send data to clients or to server
                    let send_type: &str;
                    let data_string: String;

                    let mut should_send = false;
                    let define_id = unsafe{(*data).dwDefineID};
                    let dw_data: [u8; 1024] = unsafe {std::mem::transmute_copy(&(*data).dwData)};

                    match define_id {
                        0 => {
                            let sim_data: SimValue = definitions.sim_vars.read_from_dword(dw_data);
                            send_type = "physics";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                            interpolation.record_current(sim_data);
                            // Update when time elapsed > than calculated update rate
                            if update_rate_instant.elapsed().as_secs_f64() > update_rate {
                                update_rate_instant = Instant::now();
                                should_send = has_control;
                            }
                        },
                        1 => {
                            let sim_data: SimValue = bool_defs.read_from_dword(dw_data);
                            send_type = "sync_toggle";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                            definitions.record_boolean_values(sim_data);
                            should_send = should_sync && has_control;
                            should_sync = false;
                        },
                        _ => panic!("Not covered!")
                    };

                    should_send = should_send && client.client.get_connected_count() > 0;

                    // Update position data
                    if should_send {
                        tx.send(json!({
                            "type": send_type,
                            "data": data_string,
                            "time": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64()
                        })).ok();
                    }

                    if !has_control {
                        if let Some(updated_map) = interpolation.interpolate() {
                            let mut bytes = definitions.sim_vars.write_to_data(&updated_map);
                            conn.set_data_on_sim_object(0, 0, 0, 0, bytes.len() as u32, bytes.as_mut_ptr() as *mut std::ffi::c_void);
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
                    let event_id = unsafe{(*data).uEventID};
                    let dw_data = unsafe{(*data).dwData};
                    let group_id = unsafe{(*data).uGroupID};
                    match group_id {
                        0 => {
                            tx.send(json!({
                                "type": "event",
                                "eventid": event_id,
                                "data": dw_data
                            })).ok();
                        },
                        1 => {
                            if has_control {
                                tx.send(json!({
                                    "type": "relieve_control"
                                })).ok();
                            } else if can_take_control {
                                tx.send(json!({
                                    "type": "transfer_control"
                                })).ok();
                                has_control = true;
                                app_interface.gain_control();
                                transfer_control(&conn, has_control);
                            }
                        },

                        _ => ()
                    }
                }
                _ => ()
            };
            // Data from the person in control
            match rx.try_recv() {
                Ok(ReceiveData::Data(value)) => match value["type"].as_str().unwrap() {
                    "physics" => { // Interpolate position update
                        if !has_control {
                            interpolation.record_latest(serde_json::from_str(value["data"].as_str().unwrap()).unwrap(), value["time"].as_f64().unwrap());
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
                        app_interface.can_take_control();
                        can_take_control = true;
                    },
                    "transfer_control" => {
                        if has_control {
                            app_interface.lose_control();
                            has_control = false;
                            time_since_control = Instant::now();
                            transfer_control(&conn, has_control);
                        }
                    }
                    _ => ()
                },
                Ok(ReceiveData::NewConnection(_)) | Ok(ReceiveData::ConnectionLost(_)) => {
                    app_interface.server_started(client.client.get_connected_count());
                    should_sync = true;
                },
                _ => ()
            }
            if !has_control && time_since_control.elapsed().as_secs() > 10 {
                if let Some(t) = interpolation.get_time_since_last_position() {
                    if t > 30.0 {
                        if !client.client.is_server() {
                            client.client.stop();
                            app_interface.client_fail("Peer timeout.");
                            was_error = true;
                        };
                    }
                }
            }

            if client.client.stopped() {
                transfer_client = None;

                if !was_error {app_interface.disconnected()}

                has_control = true;
                was_error = false;
                transfer_control(&conn, has_control);
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

                            app_interface.gain_control();
                            has_control = true;
                            transfer_control(&conn, has_control);
                        },
                        Err(e) => {
                            app_interface.server_fail(e.to_string().as_str());
                            // Was error not nessecary here, stopped does not get fired
                        }
                    }
                }
                AppMessage::Connect(ip, port) => {
                    app_interface.attempt();
                    match start_client(ip, port) {
                        Ok(transfer) => {
                            app_interface.connected();
                            transfer_client = Some(transfer);
                            time_since_control = Instant::now();

                            has_control = false;
                            transfer_control(&conn, has_control);
                        }
                        Err(e) => {
                            app_interface.client_fail(e.to_string().as_str());
                            // Was error not nessecary here
                        }
                    }
                    config.set_ip(ip.to_string());
                    config.write_to_file(config_filename);
                }
                AppMessage::Disconnect => {
                    if let Some(client) = transfer_client.as_ref() {
                        client.client.stop();
                    }
                }
                AppMessage::TakeControl => {
                    if can_take_control {
                        transfer_client.as_ref().unwrap().tx.send(json!({
                            "type": "transfer_control"
                        })).ok();
                        has_control = true;
                        app_interface.gain_control();
                        transfer_control(&conn, has_control);
                    }
                }
                AppMessage::RelieveControl => {
                    transfer_client.as_ref().unwrap().tx.send(json!({
                        "type": "relieve_control"
                    })).ok();
                }
            }
            Err(_) => {}
        }
        if app_interface.exited() {break}
        std::thread::sleep(Duration::from_millis(1));
    }
}
