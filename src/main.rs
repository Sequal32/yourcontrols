mod definitions;
mod simserver;
mod simclient;
mod simconfig;
mod syncdefs;
mod interpolate;

use chrono;
use simconnectsdk;
use simserver::Server;
use simclient::Client;
use interpolate::{interpolate_f64};
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, net::Ipv4Addr};
use simconfig::Config;
use simserver::ReceiveData;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct PosStruct {
    // 3D
    lat: f64,
    lon: f64,
    alt: f64,
    pitch: f64,
    bank: f64,
    heading: f64,
    // // Quadrant
    throttle: f64,
    mixture: f64,
    // prop: f64,
    // // Physics
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
    accel_x: f64,
    accel_y: f64,
    accel_z: f64,
    rotation_vel_x: f64,
    rotation_vel_y: f64,
    rotation_vel_z: f64,
    airspeed: f64,
    yoke_x: f64,
    yoke_y: f64,
    // Surfaces
    rudder_pedal: f64,
    rudder: f64,
    elevator: f64,
    aileron: f64,

    elevator_trim: f64,
    rudder_trim: f64,
    brake_left: f64,
    brake_right: f64,
    flaps: i32,

    // gear_handle: i16,
    // gear_center: f64,
    // gear_left: f64,
    // gear_right: f64,
}
#[derive(Serialize, Deserialize, Debug)]
#[repr(C)]
struct SyncStruct {
    strobe_on: bool,
    panel_on: bool,
    landing_on: bool,
    taxi_on: bool,
    beacon_on: bool,
    nav_on: bool,
    logo_on: bool,
    recognition_on: bool,
    cabin_on: bool,
}

fn main() {
    // Load configuration file
    let config = Config::read_from_file("config.json").unwrap_or_default();
    
    // Set up sim connect
    let mut conn = simconnectsdk::SimConnector::new();
    conn.connect("Simple Shared Cockpit");

    definitions::map_data(&conn);
    let sync_map = definitions::map_events(&conn);

    conn.request_data_on_sim_object(0, 0, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME);

    conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
    conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
    conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    conn.map_client_event_to_sim_event(1003, "FREEZE_LATITUDE_LONGITUDE_TOGGLE");
    conn.map_client_event_to_sim_event(1004, "FREEZE_ALTITUDE_TOGGLE");
    conn.map_client_event_to_sim_event(1005, "FREEZE_ATTITUDE_TOGGLE");

    conn.map_input_event_to_client_event(2, "Shift+U", 1005, 0, u32::MAX, 0, false);
    conn.set_input_priority(2, simconnectsdk::SIMCONNECT_GROUP_PRIORITY_HIGHEST);
    conn.set_input_group_state(2, 1);
    conn.add_client_event_to_notification_group(1, 1005, false);
    // Init syncable stuff

    // Whether to start a client or a server

    let mut has_control;
    let mut can_take_control = false;

    let result: String = text_io::read!("{}");
    let (tx, rx) = match result.len() {
        1 => { 
            let mut server = Server::new();
            match server.start(config.port) {
                Ok((tx, rx)) => {
                    println!("Server started!");
                    has_control = true;
                    (tx, rx)
                },
                Err(err) => panic!("Could not start server! {:?}", err)
            }
        },
        _ => match Ipv4Addr::from_str(&result) {
            Ok(ip) => match Client::start(ip, config.port) {
                Ok((tx, rx)) => {
                    println!("Client connected!");
                    has_control = false;
                    (tx, rx)
                },
                Err(err) => panic!("Could not start client! {:?}", err)
            },
            Err(_) => panic!("Invalid ip provided!")
        }
    };
    conn.transmit_client_event(1, 1000, 0, 5, 0);
    conn.transmit_client_event(1, 1001, 0, 5, 0);
    conn.transmit_client_event(1, 1002, 0, 5, 0);
    if has_control {
        conn.transmit_client_event(1, 1003, 0, 5, 0);
        conn.transmit_client_event(1, 1004, 0, 5, 0);
        conn.transmit_client_event(1, 1005, 0, 5, 0);
    }

    let mut instant = std::time::Instant::now();
    let mut last_pos_update: Option<Value> = None;
    let mut pos_update: Option<Value> = None;
    let mut current_pos: Option<Value> = None;
    // Set data upon receipt
    let mut interpolation_time = 0.0;
    let mut add_alpha = 0.0;
    let mut last_packet: Option<Value> = None;

    let mut last_bool_sync: Option<Value> = None;

    let mut tick = 0;
    
    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnectsdk::DispatchResult::SimobjectData(data)) => {
                // Send data to clients or to server
                unsafe {
                    let send_type: &str;
                    let data_string: String;

                    tick += 1;

                    let mut should_send = true;

                    match (*data).dwDefineID {
                        0 => {
                            let sim_data: PosStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "physics";
                            let val = serde_json::to_value(&sim_data).unwrap();
                            data_string = val.to_string();
                            current_pos = Some(val);

                            should_send = tick % 15 == 0;
                        },
                        1 => {
                            let sim_data: SyncStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "sync_toggle";
                            let val = serde_json::to_value(&sim_data).unwrap();
                            data_string = val.to_string();
                            last_bool_sync = Some(val);
                        },
                        _ => panic!("Not covered!")
                    };

                    // Update position data
                    if has_control {
                        if should_send {
                            tx.send(json!({
                                "type": send_type,
                                "data": data_string,
                                "time": chrono::Utc::now().timestamp_millis()
                            })).expect("!");
                        }
                    } else {
                        match (&last_pos_update, &pos_update) {
                            (Some(last), Some(current)) => {
                                // Interpolate previous recorded position with previous update
                                let current_object = current.as_object().unwrap();
                                let mut updated_map = serde_json::Map::new();

                                let alpha = instant.elapsed().as_millis() as f64/interpolation_time + add_alpha;
    
                                for (key, value) in last.as_object().unwrap() {
                                    if value.is_f64() {
                                        let interpolated = interpolate_f64(value.as_f64().unwrap(), current_object[key].as_f64().unwrap(), alpha);
                                        updated_map.insert(key.to_string(), Value::from(interpolated));
                                    } else {
                                        updated_map.insert(key.to_string(), value.clone());
                                    }
                                }

                                let mut updated: PosStruct = serde_json::from_value(serde_json::value::to_value(updated_map).unwrap()).unwrap();
                                let data_pointer: *mut std::ffi::c_void = &mut updated as *mut PosStruct as *mut std::ffi::c_void;
                                conn.set_data_on_sim_object(0, 0, 0, 0, std::mem::size_of::<PosStruct>() as u32, data_pointer);
                            },
                            _ => ()
                        }
                    }
                }
            },
            // Exception occured
            Ok(simconnectsdk::DispatchResult::Exception(data)) => {
                unsafe {
                    println!("{:?}", (*data).dwException);
                }
            },
            Ok(simconnectsdk::DispatchResult::Event(data)) => {
                unsafe {
                println!("{:?}", *data);
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
                "physics" => {
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
                },
                "sync_toggle" => {
                    let data: Value = serde_json::from_str(&value["data"].as_str().unwrap()).unwrap();
                    let last = (&last_bool_sync).as_ref().unwrap();
                    for (key, value) in data.as_object().unwrap() {
                        let last = last[key].as_bool().unwrap();
                        let current = value.as_bool().unwrap();
                        sync_map.get(key.as_str()).unwrap().sync(&conn, last, current);
                    }
                },
                "event" => {
                    let event_id = value["eventid"].as_u64().unwrap();
                    let data = value["data"].as_i64().unwrap();
                    conn.transmit_client_event(1, event_id as u32, data as u32, 0, 0);
                },
                "relievecontrol" => {
                    can_take_control = true;
                },
                "transfercontrol" => {
                    if has_control {has_control = false;}
                }
                _ => ()
            },
            Ok(ReceiveData::NewConnection) => {
                if has_control {
                    conn.request_data_on_sim_object(1, 1, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_ONCE);
                }
            },
            _ => ()
        }

        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
