use simconnect::SimConnector;
use yourcontrols_types::{AVarMap, VarReaderTypes};

use crate::sync::transfer::AircraftVars;
use crate::util::InDataTypes;

const ALTITUDE_CHANGE_THRESHOLD: f64 = 1000.0;

#[derive(Default)]
struct Current {
    wind_z: f64,
    ground_alt: f64,
}

fn average(new_value: f64, average: f64) -> f64 {
    return average + (new_value - average) / 5 as f64;
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
        avars.add_var("GROUND ALTITUDE", "Feet", InDataTypes::F64);

        Self {
            avars,
            current: Current::default(),
        }
    }

    pub fn remove_components(&self, data: &mut AVarMap) {
        if let Some(VarReaderTypes::F64(velocity)) = data.get_mut("VELOCITY BODY Z") {
            *velocity -= self.current.wind_z
        }

        if let Some(VarReaderTypes::F64(altitude)) = data.get_mut("PLANE ALTITUDE") {
            if *altitude <= ALTITUDE_CHANGE_THRESHOLD {
                *altitude -= self.current.ground_alt
            }
        }
    }

    pub fn add_components(&self, data: &mut AVarMap) {
        if let Some(VarReaderTypes::F64(velocity)) = data.get_mut("VELOCITY BODY Z") {
            *velocity += self.current.wind_z
        }

        if let Some(VarReaderTypes::F64(altitude)) = data.get_mut("PLANE ALTITUDE") {
            if *altitude <= ALTITUDE_CHANGE_THRESHOLD {
                *altitude += self.current.ground_alt
            }
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

            if let Some(VarReaderTypes::F64(altitude)) = data.get("GROUND ALTITUDE") {
                self.current.ground_alt = average(*altitude, self.current.ground_alt);
            }
        }
    }
}
