use bimap::BiHashMap;
use std::{collections::{HashMap, HashSet}, io};
use simconnect::SimConnector;
use crate::{util::InDataTypes, varreader::SimValue, varreader::VarReader, util::VarReaderTypes};

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

    pub fn match_event_id(&self, event_id: u32) -> Option<&String> {
        return self.event_map.get_by_right(&event_id);
    }

    pub fn trigger_event(&self, conn: &SimConnector, event_name: &str, data: u32) {
        conn.transmit_client_event(1, *self.event_map.get_by_left(&event_name.to_string()).unwrap(), data, 0, 0);
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        for (event_name, event_id) in self.event_map.iter() {
            conn.map_client_event_to_sim_event(*event_id, event_name);

            if self.should_notify.contains(event_id) {
                conn.add_client_event_to_notification_group(self.group_id, *event_id, true);
            }
        }
    }

    pub fn get_number_defined(&self) -> usize {
        return self.event_map.len()
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
        let vars = match self.reader.read_from_bytes(data.dwDefineCount, unsafe{ &data.dwData as *const u32 }) {
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

    pub fn get_var(&self, var_name: &str) -> Option<&VarReaderTypes> {
        self.current_values.get(var_name)
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        conn.clear_data_definition(self.define_id);
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

    pub fn get_number_defined(&self) -> usize {
        return self.vars.len()
    }
}