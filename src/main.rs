mod simserver;
mod simclient;
mod simconfig;
mod interpolate;

use chrono;
use simconnectsdk;
use simserver::Server;
use simclient::Client;
use interpolate::{interpolate_f64};
use serde_json::{json, Value, Number};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, net::Ipv4Addr};
use simconfig::Config;

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
struct PeriodicalStruct {
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

fn map_data(conn: &simconnectsdk::SimConnector) {
    conn.add_data_definition(0, "Plane Latitude", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "Plane Longitude", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE ALTITUDE", "Feet", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    
    conn.add_data_definition(0, "PLANE PITCH DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE BANK DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE HEADING DEGREES MAGNETIC", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(0, "GENERAL ENG THROTTLE LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "GENERAL ENG MIXTURE LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    // conn.add_data_definition(0, "GENERAL ENG PROP LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(0, "VELOCITY WORLD X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "VELOCITY WORLD Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "VELOCITY WORLD Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ACCELERATION WORLD X", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ACCELERATION WORLD Y", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ACCELERATION WORLD Z", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(0, "AIRSPEED TRUE", "Knots", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "YOKE X POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "YOKE Y POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(0, "RUDDER PEDAL POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "RUDDER POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "ELEVATOR POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "AILERON POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(0, "ELEVATOR TRIM POSITION", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "RUDDER POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "BRAKE LEFT POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "BRAKE RIGHT POSITION", "Position", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "FLAPS HANDLE INDEX", "Number", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    // conn.add_data_definition(0, "GEAR HANDLE POSITION", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, u32::MAX);
    // conn.add_data_definition(0, "GEAR CENTER POSITION", "Percent Over 100", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    // conn.add_data_definition(0, "GEAR LEFT POSITION", "Percent Over 100", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    // conn.add_data_definition(0, "GEAR RIGHT POSITION", "Percent Over 100", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

    conn.add_data_definition(1, "LIGHT STROBE", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 0);
    conn.add_data_definition(1, "LIGHT PANEL", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 1);
    conn.add_data_definition(1, "LIGHT LANDING", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 2);
    conn.add_data_definition(1, "LIGHT TAXI", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 3);
    conn.add_data_definition(1, "LIGHT BEACON", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 4);
    conn.add_data_definition(1, "LIGHT NAV", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 5);
    conn.add_data_definition(1, "LIGHT LOGO", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 6);
    conn.add_data_definition(1, "LIGHT WING", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 7);
    conn.add_data_definition(1, "LIGHT RECOGNITION", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 8);
    conn.add_data_definition(1, "LIGHT CABIN", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, 9);
}

fn main() {
    // Load configuration file
    let config = Config::read_from_file("config.json").unwrap_or_default();
    
    // Set up sim connect
    let mut conn = simconnectsdk::SimConnector::new();
    conn.connect("Simple Shared Cockpit");
    map_data(&conn);
    conn.request_data_on_sim_object(0, 0, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME);
    conn.request_data_on_sim_object(1, 1, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SECOND);

    // Whether to start a client or a server

    let is_server;

    println!("Enter ip to connect or type s to start server: ");
    let result: String = text_io::read!("{}");
    let (tx, rx) = match result.len() {
        1 => { 
            let mut server = Server::new();
            match server.start(config.port) {
                Ok((tx, rx)) => {
                    println!("Server started!");
                    is_server = true;
                    (tx, rx)
                },
                Err(err) => panic!("Could not start server! {:?}", err)
            }
        },
        _ => match Ipv4Addr::from_str(&result) {
            Ok(ip) => match Client::start(ip, config.port) {
                Ok((tx, rx)) => {
                    println!("Client connected!");
                    is_server = false;
                    (tx, rx)
                },
                Err(err) => panic!("Could not start client! {:?}", err)
            },
            Err(_) => panic!("Invalid ip provided!")
        }
    };

    let mut instant = std::time::Instant::now();
    let mut last_pos_update: Option<Value> = None;
    let mut pos_update: Option<Value> = None;
    // Set data upon receipt
    let mut interpolation_time = 0.0;
    let mut last_packet: Option<Value> = None;

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

                    match (*data).dwDefineID {
                        0 => {
                            let sim_data: PosStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "physics";
                            let val = serde_json::to_value(&sim_data).unwrap();
                            data_string = val.to_string();
                        },
                        1 => {
                            let sim_data: PeriodicalStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "periodical";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                        },
                        _ => panic!("Not covered!")
                    };

                    // Update position data
                    if is_server && tick % 15 == 0 {
                        tx.send(json!({
                            "type": send_type,
                            "data": data_string,
                            "time": chrono::Utc::now().timestamp_millis()
                        })).expect("!");
                    } else if !is_server {
                        match (&last_pos_update, &pos_update) {
                            (Some(last), Some(current)) => {
                                // Interpolate previous recorded position with previous update
                                let current_object = current.as_object().unwrap();
                                let mut updated_map = serde_json::Map::new();

                                let alpha = instant.elapsed().as_millis() as f64/interpolation_time;
    
                                for (key, value) in last.as_object().unwrap() {
                                    if value.is_f64() {
                                        let interpolated = interpolate_f64(value.as_f64().unwrap(), current_object[key].as_f64().unwrap(), alpha);
                                        updated_map.insert(key.to_string(), Value::from(interpolated));
                                    } else {
                                        updated_map.insert(key.to_string(), value.clone());
                                    }
                                }
                                println!("{:?} {:?} {:?} {:?}", last["lat"], current["lat"], alpha, updated_map["lat"]);
    
                                let mut updated: PosStruct = serde_json::from_value(serde_json::value::to_value(updated_map).unwrap()).unwrap();
                                // println!("{:?}", updated);
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
            _ => {}
        };
        
        match rx.try_recv() {
            Ok(value) => match value["type"].as_str().unwrap() {
                "physics" => {
                    match last_packet {
                        Some(p) => {
                            interpolation_time = (value["time"].as_i64().unwrap()-p["time"].as_i64().unwrap()) as f64;
                            instant = std::time::Instant::now();
                            last_pos_update = pos_update.clone();
                        },
                        _ => (),
                    }
                    pos_update = Some(serde_json::from_str(value["data"].as_str().unwrap()).unwrap());
                    last_packet = Some(value);
                },
                _ => ()
            },
            Err(_) => {}
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
