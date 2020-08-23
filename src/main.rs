mod simserver;
mod simclient;
mod simconfig;
use simconnectsdk;

use simserver::Server;
use simclient::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, net::Ipv4Addr};
use simconfig::Config;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
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
    conn.add_data_definition(0, "Plane Latitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "Plane Longitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE ALTITUDE", "Feet", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    
    conn.add_data_definition(0, "PLANE PITCH DEGREES", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE BANK DEGREES", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
    conn.add_data_definition(0, "PLANE HEADING DEGREES MAGNETIC", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);

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

    println!("Enter ip to connect or type s to start server: ");
    let result: String = text_io::read!("{}");
    let is_server: bool;
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
    
    let mut tick = 0;

    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnectsdk::DispatchResult::SimobjectData(data)) => {
                // Send data to clients or to server
                unsafe {
                    let send_type: &str;
                    let data_string: String;

                    match (*data).dwDefineID {
                        0 => {
                            // Reduce send rate
                            // if tick % 5 != 0 || !is_server {continue}
                            tick += 1;

                            let sim_data: PosStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "physics";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                        },
                        1 => {
                            let sim_data: PeriodicalStruct = std::mem::transmute_copy(&(*data).dwData);
                            send_type = "periodical";
                            data_string = serde_json::to_string(&sim_data).unwrap();
                        },
                        _ => panic!("Not covered!")
                    };

                    tx.send(json!({
                        "type": send_type,
                        "data": data_string
                    })).expect("!");
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
        // Set data upon receipt
        match rx.try_recv() {
            Ok(value) => match value["type"].as_str().unwrap() {
                "physics" => {
                    let mut data: PosStruct = serde_json::from_str(value["data"].as_str().unwrap()).unwrap();
                    println!("{:?}", data);
                    let data_pointer: *mut std::ffi::c_void = &mut data as *mut PosStruct as *mut std::ffi::c_void;
                    conn.set_data_on_sim_object(0, 0, 0, 0, std::mem::size_of::<PosStruct>() as u32, data_pointer);
                },
                _ => ()
            },
            Err(_) => {continue;}
        }
    }
}
