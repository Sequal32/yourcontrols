use crate::{util::InDataTypes, varreader::SimValue, varreader::VarReader};
use bimap::BiHashMap;
use simconnect::SimConnector;
use std::{
    collections::{HashMap, HashSet},
    io,
};
use yourcontrols_types::VarReaderTypes;

use super::gaugecommunicator::{GaugeCommunicator, GetResult};

pub struct Events {
    event_map: BiHashMap<String, u32>,
    should_notify: HashSet<u32>,
    pub group_id: u32,
}

impl Events {
    pub fn new(group_id: u32) -> Self {
        Self {
            event_map: BiHashMap::new(),
            should_notify: HashSet::new(),
            group_id,
        }
    }

    pub fn get_or_map_event_id(&mut self, event_name: &str, should_notify: bool) -> u32 {
        let next_event_id = self.event_map.len() as u32;

        if let Some(event_id) = self.event_map.get_by_left(&event_name.to_string()) {
            *event_id
        } else {
            self.event_map.insert(event_name.to_string(), next_event_id);

            if should_notify {
                self.should_notify.insert(next_event_id);
            }

            next_event_id
        }
    }

    pub fn match_event_id(&self, event_id: u32) -> Option<&String> {
        self.event_map.get_by_right(&event_id)
    }

    pub fn trigger_event(
        &self,
        conn: &SimConnector,
        event_name: &str,
        data: u32,
    ) -> Result<(), ()> {
        let event_id = match self.event_map.get_by_left(&event_name.to_string()) {
            Some(id) => *id,
            None => return Err(()),
        };

        conn.transmit_client_event(1, event_id, data, 0, 0);

        Ok(())
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        for (event_name, event_id) in self.event_map.iter() {
            conn.map_client_event_to_sim_event(*event_id, event_name);

            if self.should_notify.contains(event_id) {
                conn.add_client_event_to_notification_group(self.group_id, *event_id, true);
            }
        }
    }

    pub fn shrink_maps(&mut self) {
        self.should_notify.shrink_to_fit();
    }

    pub fn get_number_defined(&self) -> usize {
        self.event_map.len()
    }
}
pub struct LVarSyncer {
    pub transfer: GaugeCommunicator,
    current_values: HashMap<String, f64>,
    raw_count: u32,
}

impl LVarSyncer {
    pub fn new() -> Self {
        Self {
            transfer: GaugeCommunicator::new(),
            current_values: HashMap::new(),
            raw_count: 0,
        }
    }

    pub fn add_var(&mut self, var_name: String, var_units: Option<&str>) {
        self.transfer.add_definition(var_name, var_units);
    }

    pub fn add_custom_var(&mut self, calculator: String) -> String {
        let custom_var_name = format!("CustomLVar{}", self.raw_count);

        self.transfer
            .add_definition_raw(calculator, custom_var_name.clone());
        self.raw_count += 1;

        custom_var_name
    }

    pub fn process_client_data(
        &mut self,
        data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA,
    ) -> Vec<GetResult> {
        let values = self.transfer.process_client_data(data);

        for value in values.iter() {
            self.current_values
                .insert(value.var_name.clone(), value.value);
        }

        values
    }

    // FIXME: var_units missing
    pub fn set(&mut self, conn: &SimConnector, var_name: &str, value: &str) {
        self.transfer.set(conn, var_name, None, value);
    }

    pub fn set_unchecked(
        &mut self,
        conn: &SimConnector,
        var_name: &str,
        var_units: Option<&str>,
        value: &str,
    ) {
        self.transfer.set(conn, var_name, var_units, value);
    }

    pub fn send_raw(&mut self, conn: &SimConnector, raw_string: &str) {
        self.transfer.send_raw(conn, raw_string);
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        self.transfer.on_connected(conn);
        self.transfer.send_definitions(conn);
    }

    pub fn get_var(&self, var_name: &str) -> Option<f64> {
        self.current_values.get(var_name).copied()
    }

    pub fn get_all_vars(&self) -> HashMap<String, f64> {
        self.current_values.clone()
    }

    pub fn get_number_defined(&self) -> usize {
        self.transfer.get_number_defined()
    }

    pub fn shrink_maps(&mut self) {
        self.current_values.shrink_to_fit();
    }
}

pub struct AircraftVars {
    pub define_id: u32,
    vars: HashMap<String, AircraftVar>,
    current_values: SimValue,
    reader: VarReader,
}

struct AircraftVar {
    datum_id: u32,
    var_units: String,
    var_type: InDataTypes,
}

impl AircraftVars {
    pub fn new(define_id: u32) -> Self {
        Self {
            define_id,
            vars: HashMap::new(),
            current_values: HashMap::new(),
            reader: VarReader::new(),
        }
    }

    pub fn add_var(&mut self, var_name: &str, var_units: &str, data_type: InDataTypes) {
        if self.vars.contains_key(var_name) {
            return;
        }

        self.vars.insert(
            var_name.to_string(),
            AircraftVar {
                var_type: data_type,
                var_units: var_units.to_string(),
                datum_id: self.reader.add_definition(var_name, data_type),
            },
        );
    }

    pub fn read_vars(
        &mut self,
        data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA,
    ) -> Result<SimValue, io::Error> {
        let vars = match self.reader.read_from_bytes(
            data.dwDefineCount,
            std::ptr::addr_of!(data.dwData) as *const u32,
        ) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        for (var_name, value) in vars.iter() {
            self.current_values.insert(var_name.clone(), *value);
        }

        Ok(vars)
    }

    pub fn get_all_vars(&self) -> &SimValue {
        &self.current_values
    }

    pub fn set_vars(&self, conn: &SimConnector, data: &SimValue) {
        let mut bytes = self.reader.write_to_data(data);

        unsafe {
            conn.set_data_on_sim_object(
                self.define_id,
                0,
                simconnect::SIMCONNECT_CLIENT_DATA_SET_FLAG_TAGGED,
                data.len() as u32,
                bytes.len() as u32,
                bytes.as_mut_ptr() as *mut std::ffi::c_void,
            );
        }
    }

    pub fn get_var(&self, var_name: &str) -> Option<&VarReaderTypes> {
        self.current_values.get(var_name)
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        conn.clear_data_definition(self.define_id);
        for (var_name, var_data) in self.vars.iter() {
            match var_data.var_type {
                InDataTypes::Bool | InDataTypes::I32 => {
                    conn.add_data_definition(
                        self.define_id,
                        var_name,
                        &var_data.var_units,
                        simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32,
                        var_data.datum_id,
                    );
                }
                InDataTypes::I64 => {}
                InDataTypes::F64 => {
                    conn.add_data_definition(
                        self.define_id,
                        var_name,
                        &var_data.var_units,
                        simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64,
                        var_data.datum_id,
                    );
                }
            }
        }
    }

    pub fn get_number_defined(&self) -> usize {
        self.vars.len()
    }

    pub fn shrink_maps(&mut self) {
        self.vars.shrink_to_fit();
    }
}
