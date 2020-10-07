pub mod control;

use bimap::BiHashMap;
use std::{collections::{HashMap, HashSet}, io};
use simconnect::SimConnector;
use crate::{lvars::{LVar, LVars, DiffChecker, GetResult},util::InDataTypes, varreader::SimValue, varreader::VarReader};

pub struct Events {
    event_map: BiHashMap<String, u32>,
    should_notify: HashSet<u32>,
    pub group_id: u32
}

impl Events {
    pub fn new(group_id: u32) -> Self {
        Self { 
            event_map: BiHashMap::new(),
            should_notify: HashSet::new(),
            group_id
        } 
    }

    pub fn get_or_map_event_id(&mut self, event_name: &str, should_notify: bool) -> u32 {
        let next_event_id = self.event_map.len() as u32;

        if let Some(event_id) = self.event_map.get_by_left(&event_name.to_string()) {

            return *event_id;

        } else {

            self.event_map.insert(event_name.to_string(), next_event_id);

            if should_notify {
                self.should_notify.insert(next_event_id);
            }

            return next_event_id
        }
    }

    pub fn match_event_id(&self, event_id: u32) -> &String {
        return self.event_map.get_by_right(&event_id).unwrap();
    }

    pub fn trigger_event(&self, conn: &SimConnector, event_name: &str, data: u32) {
        conn.transmit_client_event(0, *self.event_map.get_by_left(&event_name.to_string()).unwrap(), data, 0, 0);
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        for (event_name, event_id) in self.event_map.iter() {
            conn.map_client_event_to_sim_event(*event_id, event_name);

            if self.should_notify.contains(event_id) {
                conn.add_client_event_to_notification_group(self.group_id, *event_id, true);
            }
        }
    }
}

struct LocalVarEntry {
    units: Option<String>,
}

pub struct LVarSyncer {
    transfer: LVars,
    tracked: DiffChecker<String, LVar>,
    vars: HashMap<String, LocalVarEntry>,
}

impl LVarSyncer {
    pub fn new(request_id: u32) -> Self {
        Self {
            transfer: LVars::new(request_id),
            tracked: DiffChecker::new(),
            vars: HashMap::new(),
        }
    }

    pub fn add_var(&mut self, var_name: &str, var_units: Option<&str>) {
        if self.vars.contains_key(var_name) {return}
        
        self.vars.insert(var_name.to_string(), LocalVarEntry {
            units: if var_units.is_some() {Some(var_units.unwrap().to_string())} else {None},
        }); 
    }

    pub fn fetch_all(&mut self, conn: &SimConnector) {
        for (var_name, var_data) in &self.vars {
            self.transfer.get(conn, var_name, var_data.units.as_deref());
        }
    }

    pub fn process_client_data(&mut self, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) -> Option<GetResult> {
        if let Some(data) = self.transfer.process_client_data(data) {
            // If the data is different...
            if self.tracked.record_and_is_diff(&data.var_name, data.var.clone()) {
                return Some(data)
            }
        }
        None
    }

    pub fn set(&mut self, conn: &SimConnector, var_name: &str, value: &str) {
        if let Some(var_data) = self.vars.get(var_name) {
            self.transfer.set(conn, var_name, var_data.units.as_deref(), value);
        }
    }

    pub fn set_unchecked(&mut self, conn: &SimConnector, var_name: &str, var_units: Option<&str>, value: &str) {
        self.transfer.set(conn, var_name, var_units, value);
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        self.transfer.on_connected(conn);
    }

    pub fn get_all_vars(&self) -> HashMap<String, f64> {
        let mut return_map = HashMap::new();

        for (var_name, value) in self.tracked.get_all() {
            return_map.insert(var_name.clone(), value.floating);
        }

        return return_map
    }
}

pub struct AircraftVars {
    pub define_id: u32,
    vars: HashMap<String, AircraftVar>,
    current_values: SimValue,
    reader: VarReader
}

struct AircraftVar {
    datum_id: u32,
    var_units: String,
    var_type: InDataTypes
}

impl AircraftVars {
    pub fn new(define_id: u32) -> Self {
        Self {
            define_id,
            vars: HashMap::new(),
            current_values: HashMap::new(),
            reader: VarReader::new()
        }
    }

    pub fn add_var(&mut self, var_name: &str, var_units: &str, data_type: InDataTypes) {
        if self.vars.contains_key(var_name) {return}

        self.vars.insert(var_name.to_string(), AircraftVar {
            var_type: data_type,
            var_units: var_units.to_string(),
            datum_id: self.reader.add_definition(var_name, data_type)
        });
    }

    pub fn read_vars(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) -> Result<SimValue, io::Error> {
        let vars = match self.reader.read_from_bytes(data.dwDefineCount, &data.dwData as *const u32) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        for (var_name, value) in vars.iter() {
            self.current_values.insert(var_name.clone(), value.clone());
        }
        
        return Ok(vars);
    }

    pub fn get_all_vars(&self) -> &SimValue {
        return &self.current_values;
    }

    pub fn set_vars(&self, conn: &SimConnector, data: &SimValue) {
        let mut bytes = self.reader.write_to_data(data);
        conn.set_data_on_sim_object(self.define_id, 0, simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED, data.len() as u32, bytes.len() as u32, bytes.as_mut_ptr() as *mut std::ffi::c_void);
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        for (var_name, var_data) in self.vars.iter() {
            match var_data.var_type {
                InDataTypes::Bool | InDataTypes::I32 => {
                    conn.add_data_definition(self.define_id, var_name, &var_data.var_units, simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, var_data.datum_id);
                }
                InDataTypes::I64 => {}
                InDataTypes::F64 => {
                    conn.add_data_definition(self.define_id, var_name, &var_data.var_units, simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, var_data.datum_id);
                }
            }
        }
    }
}

// pub struct AVarSyncer {
//     cache_vars: Option<SimValue>,
//     needs_sync: bool,
//     tracked: DiffChecker<String, VarReaderTypes>,
//     vars: AircraftVars,
// }

// impl AVarSyncer {
//     pub fn new(conn: Rc<SimConnector>, define_id: u32) -> Self {
//         Self {
//             needs_sync: false,
//             cache_vars: None,
//             vars: AircraftVars::new(conn, define_id),
//             tracked: DiffChecker::new()
//         }
//     }

//     pub fn add_var(&mut self, var_name: &str, var_units: &str, data_type: InDataTypes) {
//         self.vars.add_var(var_name, var_units, data_type);
//     }

//     // Returns list of aircraft vars that need syncing
//     pub fn process_if_need_sync(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) -> Vec<String> {
//         let vars = self.vars.read_vars(data);
//         let mut need_sync_vars: Vec<String> = Vec::new();

//         if let Some(prev_vars) = self.cache_vars.as_ref() {
//             // Check for diff
//             for (var_name, value) in vars.iter() {
//                 if prev_vars.get(var_name).unwrap() != value {
//                     need_sync_vars.push(var_name.clone());
//                 }
//             }
//         }
        
//         self.cache_vars = Some(vars);

//         return need_sync_vars;
//     }

//     pub fn set(&self, data: &SimValue) {
//         self.vars.set_vars(&data);
//     }

//     pub fn get_var(&self, var_name: &str) -> Option<&VarReaderTypes> {
//         let vars = self.cache_vars.as_ref()?;
//         return vars.get(&var_name.to_string());
//     }
// }