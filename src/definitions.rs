use crate::{bytereader::{StructData, InDataTypes, StructDataTypes, data_type_as_bool}, syncdefs::{Syncable, ToggleSwitch, ToggleSwitchSet}};
use csv;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use indexmap::IndexMap;

#[derive(Deserialize)]
struct SimVar {
    var_name: String,
    unit_name: String,
    type_name: String
}

#[derive(Deserialize)]
struct SyncBool {
    var_name: String,
    unit_name: String,
    type_name: String,
    event_name: String,
    sync_type: String
}

pub struct Definitions {
    pub sim_vars: StructData,
    pub event_map: HashMap<String, u32>,
    
    bool_sync: HashMap<String, Box<dyn Syncable<bool>>>,
    last_bool_values: Option<IndexMap<String, StructDataTypes>>
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            sim_vars: StructData::new(),
            event_map: HashMap::new(),
            bool_sync: HashMap::new(),
            last_bool_values: None
        }
    }

    pub fn map_all(&mut self, conn: &simconnect::SimConnector) {
        self.map_data(conn);
        self.map_events(conn);
    }

    pub fn map_data(&mut self, conn: &simconnect::SimConnector) {
        let mut reader = csv::ReaderBuilder::new();
        reader.trim(csv::Trim::All);
        let mut reader = reader.from_path("sim_vars.dat").expect("Could not open sim_vars.dat! Ensure that the file is in the directory, or redownload.");

        for result in reader.deserialize() {
            let result: SimVar = result.unwrap();
            match result.type_name.as_str() {
                "f64" => {
                    conn.add_data_definition(0, result.var_name.as_str(), result.unit_name.as_str(), simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_FLOAT64, u32::MAX);
                    self.sim_vars.add_definition(result.var_name, InDataTypes::F64)
                }
                _ => ()
            };
        }
    }

    pub fn map_bool_sync_events(&mut self, conn: &simconnect::SimConnector, filename: &str, define_id: u32) -> StructData {
        let mut reader = csv::ReaderBuilder::new();
        reader.trim(csv::Trim::All);
        let mut reader = reader.from_path(filename).expect(format!("Could not open {}! Ensure that the file is in the directory, or redownload.", filename).as_str());
    
        let mut sim_vars = StructData::new();
        for result in reader.deserialize() {
    
            let result: SyncBool = result.unwrap();
            let event_id = self.event_map.get(&result.event_name).unwrap().clone();
    
            match result.type_name.as_str() {
                "i32" => {
                    conn.add_data_definition(define_id, result.var_name.as_str(), result.unit_name.as_str(), simconnect::SIMCONNECT_DATATYPE_SIMCONNECT_DATATYPE_INT32, u32::MAX);
                    sim_vars.add_definition(result.var_name.to_string(), InDataTypes::Bool);
                    match result.sync_type.as_str() {
                        "SWITCH" => {self.bool_sync.insert(result.var_name, Box::new(ToggleSwitch::new(event_id)));}
                        "SWITCHSET" => {self.bool_sync.insert(result.var_name, Box::new(ToggleSwitchSet::new(event_id)));}
                        _ => {}
                    };
                }
                _ => ()
            };
        }

        return sim_vars;
    }

    fn map_events(&mut self, conn: &simconnect::SimConnector)  {
        let reader = BufReader::new(File::open("sim_events.dat").expect("Could not open sim_vars.dat! Ensure that the file is in the directory, or redownload."));
    
        for result in reader.lines() {
            match result {
                Ok(line) => {
                    if line.trim() != "" {
                        let event_id = self.event_map.len() as u32;
                        self.event_map.insert(line.trim().to_string(), event_id);
                        conn.map_client_event_to_sim_event(event_id, line.as_str());
                        conn.add_client_event_to_notification_group(0, event_id, true);
                    }
                }
                Err(_) => ()
            }
        }
    }

    pub fn sync_boolean(&self, conn: &simconnect::SimConnector, sim_var: &String, to: bool) {
        let last_val = self.last_bool_values.as_ref().expect("Not synced yet!").get(sim_var).unwrap();

        self.bool_sync.get(sim_var)
            .expect("Could not sync! Value does not exist!")
            .sync(conn, data_type_as_bool(*last_val).unwrap(), to)
    }

    pub fn has_synced_bool_values(&self) -> bool {
        return self.last_bool_values.is_some();
    }

    pub fn record_boolean_values(&mut self, vals: IndexMap<String, StructDataTypes>) {
        self.last_bool_values = Some(vals);
    }
}
