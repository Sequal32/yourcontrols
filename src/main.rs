mod dualserver;
use simconnectsdk;

struct PosStruct {
    // 3D
    pos: simconnectsdk::SIMCONNECT_DATA_LATLONALT,
    pitch: f64,
    bank: f64,
    heading: f64,
    // Physics
    velocity: simconnectsdk::SIMCONNECT_DATA_XYZ,
    acceleration: simconnectsdk::SIMCONNECT_DATA_XYZ,
    rotation_velocity: simconnectsdk::SIMCONNECT_DATA_XYZ,
    // Quadrant
    throttle: f64,
    mixture: f64,
    prop: f64
}

fn main() {
    let mut conn = simconnectsdk::SimConnector::new();
    conn.connect("Simple Shared Cockpit");
    conn.add_data_definition(0, "STRUCT LATLONALT", "SIMCONNECT_DATA_LATLONALT", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_LATLONALT);
    conn.add_data_definition(0, "PLANE PITCH DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "PLANE BANK DEGREES", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "PLANE HEADING DEGREES MAGNETIC", "Radians", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);

    conn.add_data_definition(0, "STRUCT WORLD VELOCITY", "SIMCONNECT_DATA_XYZ", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_XYZ);
    conn.add_data_definition(0, "STRUCT WORLD ACCELERATION", "SIMCONNECT_DATA_XYZ", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_XYZ);
    conn.add_data_definition(0, "STRUCT WORLD ROTATION VELOCITY", "SIMCONNECT_DATA_XYZ", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_XYZ);

    conn.add_data_definition(0, "GENERAL ENG THROTTLE LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "GENERAL ENG MIXTURE LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);
    conn.add_data_definition(0, "GENERAL ENG PROP LEVER POSITION:index", "Percent", simconnectsdk::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64);


    conn.request_data_on_sim_object(0, 0, 0, simconnectsdk::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SECOND);


    loop {
        let message = conn.get_next_message();
        match message {
            Ok(simconnectsdk::DispatchResult::SimobjectData(data)) => {
                
            },
            Ok(simconnectsdk::DispatchResult::Exception(data)) => {
                println!("{:?}", data.dwException);
            },
            Ok(simconnectsdk::DispatchResult::Null) => {
                println!("NOO");
            }
            _ => {}
        }
    }
}
