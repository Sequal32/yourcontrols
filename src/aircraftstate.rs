use crate::syncdefs::*;
use std::collections::HashMap;

pub struct AircraftState {
    bool_sync: HashMap<&'static str, Box<dyn Syncable<bool>>>,
    int_sync: HashMap<&'static str, Box<dyn Syncable<i32>>>,
    f_sync: HashMap<&'static str, Box<dyn Syncable<f64>>>,
}

impl AircraftState {
    pub fn new() -> Self {
        return Self {
            bool_sync: HashMap::new(),
            int_sync: HashMap::new(),
            f_sync: HashMap::new(),
        }
    }

    pub fn add_bool_sync(&mut self, sim_var_name: &'static str, switch: Box<dyn Syncable<bool>>) {
        self.bool_sync.insert(sim_var_name, switch);
    }

    pub fn sync_bool(&self, conn: &simconnect::SimConnector, sim_var_name: &'static str) -> Result((), &str) {
        match self.bool_sync.get(sim_var_name) {
            Some(toggle) => {
                toggle.sync(conn, from, to);
            }
            None => {}
        }
    }
}