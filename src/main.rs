mod dualserver;
use simconnectsdk;

use dualserver::DualServer;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PosStruct {
    // 3D
    lat: f64,
    lon: f64,
    alt: f32,
    pitch: f64,
    bank: f64,
    heading: f64,
    // Physics
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
    acceleration_x: f32,
    acceleration_y: f32,
    acceleration_z: f32,
    rotation_velocity_x: f32,
    rotation_velocity_y: f32,
    rotation_velocity_z: f32,
    // Quadrant
    throttle: f32,
    mixture: f32,
    prop: f32
}
#[derive(Serialize, Deserialize)]
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

fn main() {
    let mut conn = simconnectsdk::SimConnector::new();
    conn.connect("Simple Shared Cockpit");
    conn.add_data_definition(0, "Plane Latitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "Plane Longitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "PLANE ALTITUDE", "Feet", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    
    conn.add_data_definition(0, "PLANE PITCH DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "PLANE BANK DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "PLANE HEADING DEGREES MAGNETIC", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);

    conn.add_data_definition(0, "VELOCITY WORLD X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "VELOCITY WORLD Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "VELOCITY WORLD Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ACCELERATION WORLD X", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ACCELERATION WORLD Y", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ACCELERATION WORLD Z", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "ROTATION VELOCITY BODY Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);

    conn.add_data_definition(0, "GENERAL ENG THROTTLE LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "GENERAL ENG MIXTURE LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    conn.add_data_definition(0, "GENERAL ENG PROP LEVER POSITION:1", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);

    conn.add_data_definition(1, "LIGHT STROBE", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT PANEL", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT LANDING", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT TAXI", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT BEACON", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT NAV", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT LOGO", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT WING", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT RECOGNITION", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);
    conn.add_data_definition(1, "LIGHT CABIN", "Bool", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32);

    conn.request_data_on_sim_object(0, 0, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME);
    conn.request_data_on_sim_object(1, 1, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SECOND);

    //

    let mut server = DualServer::new();
    let (tx, rx) = server.start(10053);

    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnectsdk::DispatchResult::SimobjectData(data)) => {
                unsafe {
                    let send_type: &str;
                    let data_string: String;

                    match (*data).dwDefineID {
                        0 => {
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
                        "type": "udp",
                        "payload": {
                            "type": send_type,
                            "data": data_string
                        }
                    }));
                }
                
            },
            Ok(simconnectsdk::DispatchResult::Exception(data)) => {
                unsafe {
                    println!("{:?}", (*data).dwException);
                }
            },
            Ok(simconnectsdk::DispatchResult::Null) => {
                println!("NOO");
            }
            _ => {}
        }
    }
}
