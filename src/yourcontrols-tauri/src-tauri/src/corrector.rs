#![allow(dead_code)]

use simconnect::SimConnector;
use yourcontrols_types::{VarMap, VarReaderTypes};

use crate::sync::transfer::AircraftVars;
use crate::util::InDataTypes;

#[derive(Default)]
struct Current {
    wind_z: f64,
}

// In order to synchronize groundspeed in different winds, the Z component is removed remotely and added locally
// Because A:PLANE ALT ABOVE GROUND is not directly setable by execute_calculate_code, the logic is done in this struct instead where the ground altitude is removed remotely below 1000 feet, and added back locally.
pub struct Corrector {
    avars: AircraftVars,
    current: Current,
}

impl Corrector {
    pub fn new(define_id: u32) -> Self {
        let mut avars = AircraftVars::new(define_id);

        avars.add_var("AIRCRAFT WIND Z", "Feet per second", InDataTypes::F64);

        Self {
            avars,
            current: Current::default(),
        }
    }

    pub fn remove_components(&self, data: &mut VarMap) {
        if let Some(VarReaderTypes::F64(velocity)) = data.get_mut("VELOCITY BODY Z") {
            *velocity -= self.current.wind_z
        }
    }

    pub fn add_components(&self, data: &mut VarMap) {
        if let Some(VarReaderTypes::F64(velocity)) = data.get_mut("VELOCITY BODY Z") {
            *velocity += self.current.wind_z
        }
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        self.avars.on_connected(conn);
        conn.request_data_on_sim_object(
            5829,
            self.avars.define_id,
            0,
            simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME,
            simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED
                | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED,
            0,
            0,
            0,
        );
    }

    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        if self.avars.define_id != data.dwDefineID {
            return;
        }

        if let Ok(data) = self.avars.read_vars(data) {
            if let Some(VarReaderTypes::F64(velocity)) = data.get("AIRCRAFT WIND Z") {
                self.current.wind_z = *velocity;
            }
        }
    }
}
