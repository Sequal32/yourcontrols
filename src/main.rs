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
    rotation_acceleration_x: f32,
    rotation_acceleration_y: f32,
    rotation_acceleration_z: f32,
    // Quadrant
    throttle: f64,
    mixture: f64,
    prop: f64
}

fn main() {
    let mut conn = simconnectsdk::SimConnector::new();
    conn.connect("Simple Shared Cockpit");
    conn.add_data_definition(0, "Plane Latitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "Plane Longitude", "degrees", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    // conn.add_data_definition(0, "PLANE ALTITUDE", "Feet", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    
    // conn.add_data_definition(0, "PLANE PITCH DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    // conn.add_data_definition(0, "PLANE BANK DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    // conn.add_data_definition(0, "PLANE HEADING DEGREES MAGNETIC", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);

    // conn.add_data_definition(0, "VELOCITY WORLD X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "VELOCITY WORLD Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "VELOCITY WORLD Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ACCELERATION WORLD X", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ACCELERATION WORLD Y", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ACCELERATION WORLD Z", "Feet per second squared", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ROTATION VELOCITY BODY X", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ROTATION VELOCITY BODY Y", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);
    // conn.add_data_definition(0, "ROTATION VELOCITY BODY Z", "Feet per second", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT32);

    // conn.add_data_definition(0, "GENERAL ENG THROTTLE LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    // conn.add_data_definition(0, "GENERAL ENG MIXTURE LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    // conn.add_data_definition(0, "GENERAL ENG PROP LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);

    conn.request_data_on_sim_object(0, 0, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME);

    //

    let mut server = DualServer::new();
    let (mut tx, rx) = server.start(10053);

    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnectsdk::DispatchResult::SimobjectData(data)) => {
                unsafe {
                    let pos_data: PosStruct = std::mem::transmute_copy(&(*data).dwData);
                    tx.send(json!({
                        "type": "udp",
                        "payload": serde_json::to_string(&pos_data).unwrap()
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
