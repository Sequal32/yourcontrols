use simconnect::SimConnector;

use crate::{definitions::{AVarMap}, sync::AircraftVars, util::{VarReaderTypes, InDataTypes, Vector3}, varreader::SimValue};

fn get_vector3_from_data_map(data: &SimValue, key_prefix: &str) -> Option<Vector3> {
    Some(
        Vector3 {
            x: data.get(&format!("{} X", key_prefix))?.get_as_f64()?.clone(),
            y: data.get(&format!("{} Y", key_prefix))?.get_as_f64()?.clone(),
            z: data.get(&format!("{} Z", key_prefix))?.get_as_f64()?.clone(),
        }
    )
}

fn write_vector3_to_data_map(vec: Vector3, data: &mut SimValue, key_prefix: &str) {
    data.insert(format!("{} X", key_prefix), VarReaderTypes::F64(vec.x));
    data.insert(format!("{} Y", key_prefix), VarReaderTypes::F64(vec.y));
    data.insert(format!("{} Z", key_prefix), VarReaderTypes::F64(vec.z));
}

pub struct VelocityCorrector {
    avars: AircraftVars,
    current_winds: Vector3
}

impl VelocityCorrector {
    pub fn new(define_id: u32) -> Self {
        let mut avars = AircraftVars::new(define_id);

        avars.add_var("AIRCRAFT WIND X", "feet/second", InDataTypes::F64);
        avars.add_var("AIRCRAFT WIND Y", "feet/second", InDataTypes::F64);
        avars.add_var("AIRCRAFT WIND Z", "feet/second", InDataTypes::F64);

        Self {
            avars,
            current_winds: Vector3::default()
        }
    }

    pub fn remove_wind_component(&self, data: &mut AVarMap) {
        if let Some(velocity) = get_vector3_from_data_map(&data, "VELOCITY BODY") {
            write_vector3_to_data_map(velocity-self.current_winds, data, "VELOCITY BODY")
        }
    }

    pub fn add_wind_component(&self, data: &mut AVarMap) {
        if let Some(velocity) = get_vector3_from_data_map(&data, "VELOCITY BODY") {
            write_vector3_to_data_map(velocity+self.current_winds, data, "VELOCITY BODY")
        }
    }
    
    pub fn on_connected(&self, conn: &SimConnector) {
        self.avars.on_connected(conn);
        conn.request_data_on_sim_object(0, self.avars.define_id, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
    }

    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        if self.avars.define_id != data.dwDefineID {return}
        
        if let Ok(data) = self.avars.read_vars(data) {
            if let Some(data) = get_vector3_from_data_map(&data, "AIRCRAFT WIND") {
                self.current_winds = data;
            }
        }
    }
}