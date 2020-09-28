use indexmap::IndexMap;
use serde_yaml::{self, Value};
use serde::{Deserialize, Serialize};
use simconnect::SimConnector;

use std::{collections::HashMap, collections::HashSet, collections::hash_map::Entry, fs::File, rc::Rc, time::Instant};
use std::io::Read;

use crate::{sync::AircraftVars, sync::Events, sync::LVarSyncer, syncdefs::{NumSet, Syncable, ToggleSwitch, ToggleSwitchParam}, util::Category, util::InDataTypes, util::VarReaderTypes};
use crate::util::LocalVar;

pub enum ConfigLoadError {
    FileError,
    ReadError,
    ParseError
}

#[derive(Debug)]
pub enum VarAddError {
    MissingField(&'static str),
    MissingEvent(&'static str),
    InvalidVarType(&'static str),
    InvalidSyncType(String),
    InvalidCategory(String),
    YamlParseError(serde_yaml::Error)
}

macro_rules! check_and_return_field {
    ($field_name:expr, $var:ident, str) => {
        match $var[$field_name].as_str() {
            Some(s) => s,
            None => return Err(VarAddError::MissingField($field_name))
        };
    };

    ($field_name:expr, $var:ident, i64) => {
        match $var[$field_name].as_i64() {
            Some(s) => s,
            None => return Err(VarAddError::MissingField($field_name))
        };
    };
}

macro_rules! try_cast_yaml {
    ($value: ident) => {
        match serde_yaml::from_value($value) {
            Ok(y) => y,
            Err(e) => return Err(VarAddError::YamlParseError(e))
        }
    }
}

type AVarMap = HashMap<String, VarReaderTypes>;
type LVarMap = HashMap<String, f64>;
type EventMap = HashMap<String, u32>;

// Serde types
#[derive(Deserialize)]
struct VarEventEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String
}

#[derive(Deserialize)]
struct ToggleSwitchParamEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    event_param: u32
}

#[derive(Deserialize)]
struct VarEntry {
    var_name: String,
    #[serde(default)]
    var_units: Option<String>,
    var_type: InDataTypes
}

#[derive(Deserialize)]
struct EventEntry {
    event_name: String
}

#[derive(Deserialize)]
struct BothSetEntry {
    vars: Vec<VarEntry>,
    mapping: Vec<Value>
}

struct SyncAction<T> {
    category: String,
    action: Box<dyn Syncable<T>>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AllNeedSync {
    pub avars: AVarMap,
    pub lvars: LVarMap,
    pub events: EventMap
}

pub struct Definitions {
    bool_maps: HashMap<String, Vec<SyncAction<bool>>>,
    num_maps: HashMap<String, Vec<SyncAction<u32>>>,
    events: Events,

    lvarstransfer: LVarSyncer,
    avarstransfer: AircraftVars,

    categories: HashMap<String, Category>,
    sync_vars: HashSet<String>,

    last_lvar_check: Instant,
    // Queues
    aircraft_var_queue: AVarMap,
    local_var_queue: LVarMap,
    event_queue: EventMap
}

fn get_category_from_string(category: &str) -> Result<Category, VarAddError> {
    match category.to_lowercase().as_str() {
        "shared" => Ok(Category::Shared),
        "master" => Ok(Category::Master),
        _ => return Err(VarAddError::InvalidCategory(category.to_string()))
    }
}

fn get_real_var_name(var_name: &str) -> String {
    if var_name.as_bytes()[1] == b':' {
        return var_name[2..].to_string()
    } else {
        return var_name.to_string()
    }
}

impl Definitions {
    pub fn new() -> Self {
        Self {
            bool_maps: HashMap::new(),
            num_maps: HashMap::new(),
            events: Events::new(1),
            lvarstransfer: LVarSyncer::new(1),
            avarstransfer: AircraftVars::new(1),
            sync_vars: HashSet::new(),
            categories: HashMap::new(),

            last_lvar_check: Instant::now(),
            aircraft_var_queue: HashMap::new(),
            local_var_queue: HashMap::new(),
            event_queue: HashMap::new(),
        }
    }

    fn add_bool_mapping(&mut self, category: &str, var_name: &str, mapping: Box<dyn Syncable<bool>>) {
        let mapping = SyncAction {
            category: category.to_string(),
            action: mapping,
        };

        match self.bool_maps.entry(var_name.to_string()) {
            Entry::Occupied(mut o) => { 
                o.get_mut().push(mapping) 
            }
            Entry::Vacant(v) => { v.insert(vec![mapping]); }
        };
    }

    fn add_num_mapping(&mut self, category: &str, var_name: &str, mapping: Box<dyn Syncable<u32>>) {
        let mapping = SyncAction {
            category: category.to_string(),
            action: mapping,
        };

        match self.num_maps.entry(get_real_var_name(var_name)) {
            Entry::Occupied(mut o) => { o.get_mut().push(mapping) }
            Entry::Vacant(v) => { v.insert(vec![mapping]); }
        };
    }

    fn add_aircraft_variable(&mut self, category: &str, var_name: &str, var_units: &str, var_type: InDataTypes) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.avarstransfer.add_var(var_name, var_units, var_type);
        self.categories.insert(var_name.to_string(), category);
        // self.avars.insert(var_name.to_string(), AircraftVar {
        //     category: category,
        //     units: var_units.to_string(),
        //     var_type: var_type,
        // });

        Ok(())
    }

    fn add_local_variable(&mut self, category: &str, var_name: &str, var_units: Option<&str>) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.lvarstransfer.add_var(var_name, var_units);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    fn add_var_string(&mut self, category: &str, var_name: &str, var_units: Option<&str>, var_type: InDataTypes) -> Result<String, VarAddError> {
        let actual_var_name = get_real_var_name(var_name);

        if var_name.starts_with("L:") {
            // Keep var_name with L: in it to pass to execute_calculator code
            self.add_local_variable(category, var_name, var_units)?;
        } else {
            if let Some(var_units) = var_units {
                self.add_aircraft_variable(category, &actual_var_name, var_units, var_type)?;
            } else {
                return Err(VarAddError::MissingField("var_units"))
            }
        }

        Ok(actual_var_name)
    }

    fn add_toggle_switch(&mut self, category: &str, var: VarEventEntry) -> Result<(), VarAddError> { 
        let event_id = self.events.get_or_map_event_id(&var.event_name);

        let var_string = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping(category, &var_string, Box::new(ToggleSwitch::new(event_id)));

        Ok(())
    }

    fn add_toggle_switch_param(&mut self, category: &str, var: ToggleSwitchParamEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name);

        let var_string = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping(category, &var_string, Box::new(ToggleSwitchParam::new(event_id, var.event_param as u32)));

        Ok(())
    }

    fn add_num_set(&mut self, category: &str, var: VarEventEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name);
        self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(category, &var.var_name, Box::new(NumSet::new(event_id)));

        Ok(())
    }

    fn add_var(&mut self, category: &str, var: VarEntry) -> Result<(), VarAddError> {
        self.add_var_string(category, &var.var_name, var.var_units.as_deref(), var.var_type)?;

        Ok(())
    }

    fn add_both_set(&mut self, category: &str, var: BothSetEntry) -> Result<(), VarAddError> {
        Ok(())
    }

    fn add_event(&mut self, category: &str, event: EventEntry) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.events.get_or_map_event_id(&event.event_name);
        self.categories.insert(event.event_name.clone(), category);

        Ok(())
    }

    fn parse_var(&mut self, category: &str, value: Value) -> Result<(), VarAddError> {
        let type_str = check_and_return_field!("type", value, str);

        match type_str.to_uppercase().as_str() {
            "TOGGLESWITCH" => self.add_toggle_switch(category, try_cast_yaml!(value))?,
            "TOGGLESWITCHPARAM" => self.add_toggle_switch_param(category, try_cast_yaml!(value))?,
            "VAR" => self.add_var(category, try_cast_yaml!(value))?,
            "BOTHSET" => self.add_both_set(category, try_cast_yaml!(value))?,
            "NUMSET" => self.add_num_set(category, try_cast_yaml!(value))?,
            "EVENT" => self.add_event(category, try_cast_yaml!(value))?,
            _ => return Err(VarAddError::InvalidSyncType(type_str.to_string()))
        };

        return Ok(());
    }

    fn parse_yaml(&mut self, yaml: IndexMap<String, Vec<Value>>) -> Result<(), VarAddError> {
        for (key, value) in yaml {
            for var_data in value {
                self.parse_var(key.as_str(), var_data)?;
            }
        }

        Ok(())
    }

    pub fn load_config(&mut self, filename: &str) -> Result<(), ConfigLoadError> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(ConfigLoadError::FileError)
        };

        let yaml = match serde_yaml::from_reader(file) {
            Ok(y) => y,
            Err(e) => return Err(ConfigLoadError::FileError)
        };

        println!("{:?} DONE", self.parse_yaml(yaml));

        Ok(())
    }

    pub fn process_client_data(&mut self, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) {
        self.lvarstransfer.process_client_data(data);
        
    }

    pub fn process_event_data(&mut self, data: &simconnect::SIMCONNECT_RECV_EVENT) {
        if data.uGroupID != self.events.group_id {return}

        let event_name = self.events.match_event_id(data.uEventID);
        self.event_queue.insert(event_name.clone(), data.dwData);
    }

    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        if self.avarstransfer.define_id != data.dwDefineID {return}
        // TODO sync
        for (var_name, value) in self.avarstransfer.read_vars(data) {
            if let Some(actions) = self.bool_maps.get_mut(&var_name) {
                for action in actions {
                    action.action.set_current(if data.dwData == 0 {false} else {true})
                }
            }
    
            if let Some(actions) = self.num_maps.get_mut(&var_name) {
                for action in actions {
                    action.action.set_current(data.dwData)
                }
            }

            self.aircraft_var_queue.insert(var_name, value);
        }
    }

    pub fn step(&mut self, conn: &SimConnector) {
        // Fetch all lvars
        if self.last_lvar_check.elapsed().as_secs() > 1 {
            self.lvarstransfer.fetch_all(conn);
            self.last_lvar_check = Instant::now();
        }

        // Check for changes
        if let Some(lvar) = self.lvarstransfer.get_next_need_sync() {
            self.local_var_queue.insert(lvar.var_name.to_string(), lvar.var.floating);
        }
    }

    pub fn get_need_sync(&mut self) -> Option<AllNeedSync> {
        if self.aircraft_var_queue.len() == 0 && self.local_var_queue.len() == 0 && self.event_queue.len() == 0 {return None}

        let data = AllNeedSync {
            avars: self.aircraft_var_queue.clone(),
            lvars: self.local_var_queue.clone(),
            events: self.event_queue.clone()
        };

        self.aircraft_var_queue.clear();
        self.local_var_queue.clear();
        self.event_queue.clear();

        return Some(data);
    }

    pub fn write_aircraft_data(&self, conn: &SimConnector, data: &AVarMap) {
        self.avarstransfer.set_vars(conn, data);
    }

    pub fn write_local_data(&mut self, conn: &SimConnector, data: &LVarMap) {
        for (var_name, value) in data {
            self.lvarstransfer.set(conn, var_name, value.to_string().as_ref())
        }
    }

    pub fn write_event_data(&mut self, conn: &SimConnector, data: &EventMap) {
        for (event_name, value) in data {
            if let Some(actions) = self.bool_maps.get_mut(event_name) {
                for action in actions {
                    action.action.set_new(if *value == 0 {false} else {true}, conn)
                }
                continue
            }
    
            if let Some(actions) = self.num_maps.get_mut(event_name) {
                for action in actions {
                    action.action.set_new(*value, conn)
                }
                continue
            }

            self.events.trigger_event(conn, event_name, *value);        
        }
    }

    pub fn on_connected(&self, conn: &SimConnector) {
        self.avarstransfer.on_connected(conn);
        self.events.on_connected(conn);
        self.lvarstransfer.on_connected(conn);

        conn.request_data_on_sim_object(0, self.avarstransfer.define_id, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
    }
}