use indexmap::IndexMap;
use serde_yaml::{self, Value};
use serde::{Deserialize, Serialize};
use simconnect::SimConnector;

use std::{collections::HashMap, collections::HashSet, collections::hash_map::Entry, fmt::Display, fs::File, time::Instant};
use crate::{interpolate::Interpolate, interpolate::InterpolateOptions, lvars::LVarResult, sync::AircraftVars, sync::Events, lvars::GetResult, sync::LVarSyncer, syncdefs::{NumDigitSet, NumIncrement, NumIncrementSet, NumSet, NumSetMultiply, NumSetSwap, SwitchOn, Syncable, ToggleSwitch, ToggleSwitchParam, ToggleSwitchTwo}, util::Category, util::InDataTypes, util::VarReaderTypes};

#[derive(Debug)]
pub enum ConfigLoadError {
    FileError,
    YamlError(serde_yaml::Error, String),
    ParseError(VarAddError, String)
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::FileError => write!(f, "Could not open file."),
            ConfigLoadError::YamlError(e, file_name) => write!(f, "Could not parse {} as YAML...{}", file_name, e.to_string()),
            ConfigLoadError::ParseError(e, file_name) => write!(f, "Error parsing configuration in {}...{}", file_name, e)
        }
    }
}

#[derive(Debug)]
pub enum VarAddError {
    MissingField(&'static str),
    InvalidSyncType(String),
    InvalidCategory(String),
    YamlParseError(serde_yaml::Error),
    IncludeError(String, String),
}

impl Display for VarAddError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarAddError::MissingField(s) => write!(f, r#"Missing field "{}""#, s),
            VarAddError::InvalidSyncType(s) => write!(f, r#"Invalid type "{}""#, s),
            VarAddError::InvalidCategory(s) => write!(f, r#"Invalid category "{}""#, s),
            VarAddError::YamlParseError(e) => write!(f, "Error parsing YAML...{}", e.to_string()),
            VarAddError::IncludeError(e_str, e) => write!(f, "{} in {}", e_str, e)
        }
    }
}

#[derive(PartialEq)]
pub enum SyncPermissions {
    Server,
    Master,
    Slave,
    ServerAndMaster
}

// Checks if a field in a Value exists, otherwise will return an error with the name of the field
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

// Tries to cast the value into a Yaml object, returns an error if failed
macro_rules! try_cast_yaml {
    ($value: ident) => {
        match serde_yaml::from_value($value) {
            Ok(y) => y,
            Err(e) => return Err(VarAddError::YamlParseError(e))
        }
    }
}

// Name of aircraft variable and the value of it
type AVarMap = HashMap<String, VarReaderTypes>;
// Name of local variable and the value of it
type LVarMap = HashMap<String, f64>;
// Name of the event and the DWORD data associated with it
type EventMap = HashMap<String, u32>;

enum VarType {
    AircraftVar,
    LocalVar
}

// Serde types
// Describes how an aircraft variable can be set using a SimEvent
#[derive(Deserialize)]
struct VarEventEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
}

// Describes how an aircraft variable can be set using a SimEvent
#[derive(Deserialize)]
struct NumSetEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    multiply_by: Option<i32>,
    #[serde(default)]
    interpolate: Option<InterpolateOptions>
}

#[derive(Deserialize)]
struct NumSetWithIndexEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    index_param: i32,
    multiply_by: Option<i32>,
}

// Describes how an aircraft variable can be set using a "TOGGLE" event
#[derive(Deserialize)]
struct ToggleSwitchParamEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    event_param: u32
}

// Describes how an aircraft variable can be set using an on and off event
#[derive(Deserialize)]
struct ToggleSwitchTwoEntry {
    var_name: String,
    var_units: Option<String>,
    on_event_name: String,
    off_event_name: String,
}

#[derive(Deserialize)]
struct IncrementEntry<T> {
    var_name: String,
    var_units: Option<String>,
    up_event_name: String,
    down_event_name: String,
    increment_by: T
}


#[derive(Deserialize)]
struct NumDigitSetEntry {
    var_name: String,
    var_units: Option<String>,
    up_event_names: Vec<String>,
    down_event_names: Vec<String>,
}

// Describes an aircraft variable to listen for changes
#[derive(Deserialize)]
struct VarEntry {
    var_name: String,
    #[serde(default)]
    var_units: Option<String>,
    var_type: InDataTypes,
    #[serde(default)]
    interpolate: Option<InterpolateOptions>,
    #[serde(default)]
    update_every: Option<f64>
}

// For swapping frequencies
#[derive(Deserialize)]
struct NumSwapEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    swap_event_name: String
}

// Describes an event to be listened to for fires
#[derive(Deserialize)]
struct EventEntry {
    event_name: String
}

// The struct that get_need_sync returns. Holds all the aircraft/local variables and events that have changed since the last call.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct AllNeedSync {
    pub avars: AVarMap,
    pub lvars: LVarMap,
    pub events: EventMap
}

impl AllNeedSync {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_empty(&self) -> bool {
        return self.avars.len() == 0 && self.lvars.len() == 0 && self.events.len() == 0
    }

    pub fn clear(&mut self) {
        self.avars.clear();
        self.lvars.clear();
        self.events.clear();
    }
}

enum ActionType {
    BoolAction(Box<dyn Syncable<bool>>),
    NumAction(Box<dyn Syncable<i32>>),
    FloatAction(NumSetEntry),
    NumSetWithIndex(NumSetWithIndexEntry),
    NumFloatAction(Box<dyn Syncable<f64>>),
    // No BCD
    FreqSwapAction(Box<dyn Syncable<i32>>)
}

struct Period {
    time: f64,
    last_update: Option<Instant>
}

impl Period {
    fn new(time: f64) -> Self {
        Self {
            time,
            last_update: None
        }
    }

    fn do_update(&mut self) -> bool {
        match self.last_update {
            Some(time) => {
                if time.elapsed().as_secs_f64() >= self.time {
                    self.last_update = Some(Instant::now());
                    true
                } else {
                    false
                }
            }
            None => {
                self.last_update = Some(Instant::now());
                true
            }
        }
    }
}

pub struct Definitions {
    // Data that can be synced using booleans (ToggleSwitch, ToggleSwitchParam)
    action_maps: HashMap<String, Vec<ActionType>>,
    // Events to listen to
    events: Events,
    // Helper struct to retrieve and detect changes in local variables
    lvarstransfer: LVarSyncer,
    // Helper struct to retrieve *changed* aircraft variables using the CHANGED and TAGGED flags in SimConnect
    avarstransfer: AircraftVars,
    // Maps variable names to categories to determine when to sync
    categories: HashMap<String, Category>,
    // Maps variable names to periodical sync times
    periods: HashMap<String, Period>,
    // Aircraft variables that should be synced and not just detected for changes
    sync_vars: HashSet<String>,
    // Value to hold the current queue
    current_sync: AllNeedSync,
    last_written: HashMap<String, Instant>,
    
    interpolation_avars: Interpolate,
    interpolation_lvars: Interpolate,
    interpolate_names: HashSet<String>
}

fn get_category_from_string(category: &str) -> Result<Category, VarAddError> {
    match category.to_lowercase().as_str() {
        "shared" => Ok(Category::Shared),
        "master" => Ok(Category::Master),
        "server" => Ok(Category::Server),
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
    pub fn new(buffer_size: usize) -> Self {
        Self {
            action_maps: HashMap::new(),
            events: Events::new(1),
            lvarstransfer: LVarSyncer::new(1),
            avarstransfer: AircraftVars::new(1),
            sync_vars: HashSet::new(),
            categories: HashMap::new(),
            periods: HashMap::new(),

            last_written: HashMap::new(),

            current_sync: AllNeedSync::new(),

            interpolation_avars: Interpolate::new(buffer_size),
            interpolation_lvars: Interpolate::new(buffer_size),
            interpolate_names: HashSet::new()
        }
    }

    fn add_mapping(&mut self, var_name: String, mapping: ActionType) {
        match self.action_maps.entry(var_name.to_string()) {
            Entry::Occupied(mut o) => { 
                o.get_mut().push(mapping)
            }
            Entry::Vacant(v) => { v.insert(vec![mapping]); }
        };
    }

    fn add_bool_mapping(&mut self, var_name: String, mapping: Box<dyn Syncable<bool>>) {
        let mapping = ActionType::BoolAction(mapping);
        self.add_mapping(var_name, mapping);
    }

    fn add_num_mapping(&mut self, var_name: String, mapping: Box<dyn Syncable<i32>>) {
        let mapping = ActionType::NumAction(mapping);
        self.add_mapping(var_name, mapping);
    }

    fn add_float_mapping(&mut self, var_name: String, mapping: Box<dyn Syncable<f64>>) {
        let mapping = ActionType::NumFloatAction(mapping);
        self.add_mapping(var_name, mapping);
    }

    fn add_aircraft_variable(&mut self, category: &str, var_name: &str, var_units: &str, var_type: InDataTypes) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.avarstransfer.add_var(var_name, var_units, var_type);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    fn add_local_variable(&mut self, category: &str, var_name: &str, var_units: Option<&str>) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        let units = match var_units {
            Some(s) => Some(s.to_string()),
            None => None
        };

        self.lvarstransfer.add_var(var_name.to_string(), units);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    // Determines whether to add an aircraft variable or local variable based off the variable name
    fn add_var_string(&mut self, category: &str, var_name: &str, var_units: Option<&str>, var_type: InDataTypes) -> Result<(String, VarType), VarAddError> {
        let actual_var_name = get_real_var_name(var_name);

        if var_name.starts_with("L:") {
            // Keep var_name with L: in it to pass to execute_calculator code
            self.add_local_variable(category, var_name, var_units)?;

            return Ok((actual_var_name, VarType::LocalVar))

        } else {
            if let Some(var_units) = var_units {
                self.add_aircraft_variable(category, &actual_var_name, var_units, var_type)?;
            } else {
                return Err(VarAddError::MissingField("var_units"))
            }

            return Ok((actual_var_name, VarType::AircraftVar))
        }
    }

    fn add_toggle_switch(&mut self, category: &str, var: VarEventEntry) -> Result<(), VarAddError> { 
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping(var_string, Box::new(ToggleSwitch::new(event_id)));

        Ok(())
    }

    fn add_toggle_switch_param(&mut self, category: &str, var: ToggleSwitchParamEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping( var_string, Box::new(ToggleSwitchParam::new(event_id, var.event_param as u32)));

        Ok(())
    }

    fn add_toggle_switch_two(&mut self, category: &str, var: ToggleSwitchTwoEntry) -> Result<(), VarAddError> {
        let on_event_id = self.events.get_or_map_event_id(&var.on_event_name, false);
        let off_event_id = self.events.get_or_map_event_id(&var.off_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping( var_string, Box::new(ToggleSwitchTwo::new(off_event_id, on_event_id)));

        Ok(())
    }

    fn add_switch_on(&mut self, category: &str, var: VarEventEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);
        // Store SyncAction
        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;
        self.add_bool_mapping(var_string, Box::new(SwitchOn::new(event_id)));

        Ok(())
    }

    fn add_num_set(&mut self, category: &str, var: NumSetEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let action: Box<dyn Syncable<i32>> = match var.multiply_by {
            Some(n) => Box::new(NumSetMultiply::new(event_id, n)),
            None => Box::new(NumSet::new(event_id))
        };

        // Store SyncAction
        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(var_string, action);

        Ok(())
    }

    fn add_num_set_with_index(&mut self, category: &str, var: NumSetWithIndexEntry) -> Result<(), VarAddError> {
        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_mapping(var_string, ActionType::NumSetWithIndex(var));

        Ok(())
    }

    fn add_num_swap(&mut self, category: &str, var: NumSwapEntry) -> Result<(), VarAddError> {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);
        let swap_event_id = self.events.get_or_map_event_id(&var.swap_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(var_string, Box::new(NumSetSwap::new(event_id, swap_event_id)));

        Ok(())
    }

    fn add_num_increment(&mut self, category: &str, var: IncrementEntry<i32>) -> Result<(), VarAddError> {
        let up_event_id = self.events.get_or_map_event_id(&var.up_event_name, false);
        let down_event_id = self.events.get_or_map_event_id(&var.down_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(var_string, Box::new(NumIncrement::new(up_event_id, down_event_id, var.increment_by)));

        Ok(())
    }

    fn add_num_increment_set(&mut self, category: &str, var: IncrementEntry<i32>) -> Result<(), VarAddError> {
        let up_event_id = self.events.get_or_map_event_id(&var.up_event_name, false);
        let down_event_id = self.events.get_or_map_event_id(&var.down_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(var_string, Box::new(NumIncrementSet::new(up_event_id, down_event_id)));

        Ok(())
    }

    fn add_num_increment_float(&mut self, category: &str, var: IncrementEntry<f64>) -> Result<(), VarAddError> {
        let up_event_id = self.events.get_or_map_event_id(&var.up_event_name, false);
        let down_event_id = self.events.get_or_map_event_id(&var.down_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::F64)?;
        self.add_float_mapping(var_string, Box::new(NumIncrement::<f64>::new(up_event_id, down_event_id, var.increment_by)));

        Ok(())
    }

    fn add_num_digit_set(&mut self, category: &str, var: NumDigitSetEntry) -> Result<(), VarAddError> {
        let mut up_event_ids = Vec::new();
        let mut down_event_ids = Vec::new();

        for up_event_name in var.up_event_names.iter() {
            up_event_ids.push(self.events.get_or_map_event_id(up_event_name, false));
        }

        for down_event_name in var.down_event_names.iter() {
            down_event_ids.push(self.events.get_or_map_event_id(down_event_name, false));
        }

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::I32)?;
        self.add_num_mapping(var_string, Box::new(NumDigitSet::new(up_event_ids, down_event_ids)));

        Ok(())
    }

    fn add_float_var(&mut self, category: &str, var: NumSetEntry) -> Result<(), VarAddError> {
        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::F64)?;
        self.add_mapping(var_string, ActionType::FloatAction(var));
        Ok(())
    }

    fn add_var(&mut self, category: &str, var: VarEntry) -> Result<(), VarAddError> {
        let (var_name, var_type) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), var.var_type)?;
        // Tell definitions to sync this variable
        self.sync_vars.insert(var_name.clone());

        // Handle interpolation for this variable
        if let Some(options) = var.interpolate {
            match var_type {
                VarType::AircraftVar => {
                    self.interpolation_avars.set_key_options(&var_name, options);
                }
                VarType::LocalVar => {
                    self.interpolation_lvars.set_key_options(&var_name, options);
                }
            }
            
            self.interpolate_names.insert(var_name.clone());
        }

        // Handle custom periods
        if let Some(period) = var.update_every {
            self.periods.insert(var_name.clone(), Period::new(period));
        }

        Ok(())
    }

    fn add_event(&mut self, category: &str, event: EventEntry) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.events.get_or_map_event_id(&event.event_name, true);
        self.categories.insert(event.event_name.clone(), category);

        Ok(())
    }

    // Calls the correct method for the specified "action" type
    fn parse_var(&mut self, category: &str, value: Value) -> Result<(), VarAddError> {
        let type_str = check_and_return_field!("type", value, str);

        // self.check_other_common_fields(&value);

        match type_str.to_uppercase().as_str() {
            "TOGGLESWITCH" => self.add_toggle_switch(category, try_cast_yaml!(value))?,
            "TOGGLESWITCHPARAM" => self.add_toggle_switch_param(category, try_cast_yaml!(value))?,
            "TOGGLESWITCHTWO" => self.add_toggle_switch_two(category, try_cast_yaml!(value))?,
            "NUMINCREMENTFLOAT" => self.add_num_increment_float(category, try_cast_yaml!(value))?,
            "NUMINCREMENT" => self.add_num_increment(category, try_cast_yaml!(value))?,
            "NUMINCREMENTSET" => self.add_num_increment_set(category, try_cast_yaml!(value))?,
            "SWITCHON" => self.add_switch_on(category, try_cast_yaml!(value))?,
            // Uses LVar
            "NUMSETFLOAT" => self.add_float_var(category, try_cast_yaml!(value))?,
            "NUMSETWITHINDEX" => self.add_num_set_with_index(category, try_cast_yaml!(value))?,
            "NUMDIGITSET" => self.add_num_digit_set(category, try_cast_yaml!(value))?,
            "NUMSWAP" => self.add_num_swap(category, try_cast_yaml!(value))?,
            "VAR" => self.add_var(category, try_cast_yaml!(value))?,
            "NUMSET" => self.add_num_set(category, try_cast_yaml!(value))?,
            "EVENT" => self.add_event(category, try_cast_yaml!(value))?,
            _ => return Err(VarAddError::InvalidSyncType(type_str.to_string()))
        };

        return Ok(());
    }

    // Iterates over the yaml's "actions"
    fn parse_yaml(&mut self, yaml: IndexMap<String, Vec<Value>>) -> Result<(), VarAddError> {
        for (key, value) in yaml {
            if key == "include" {
                for include_file in value {
                    let file_name = include_file.as_str().unwrap();

                    match self.load_config(file_name) {
                        Ok(_) => (),
                        Err(e) => {
                            if let ConfigLoadError::ParseError(e, _) = e {
                                return Err(VarAddError::IncludeError(e.to_string(), file_name.to_string()));
                            };
                        }
                    }
                }
            } else {
                for var_data in value {
                    self.parse_var(key.as_str(), var_data)?;
                }
            }
            
        }

        Ok(())
    }

    // Load yaml from file
    pub fn load_config(&mut self, filename: &str) -> Result<(), ConfigLoadError> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(ConfigLoadError::FileError)
        };

        let yaml = match serde_yaml::from_reader(file) {
            Ok(y) => y,
            Err(e) => return Err(ConfigLoadError::YamlError(e, filename.to_string()))
        };

        match self.parse_yaml(yaml) {
            Ok(_) => Ok(()),
            Err(e) => Err(ConfigLoadError::ParseError(e, filename.to_string()))
        }
    }

    fn process_lvar(&mut self, lvar_data: GetResult) {
        // Check timer
        if let Some(timer) = self.last_written.get(&lvar_data.var_name) {
            if timer.elapsed().as_secs() < 1 {return}
        };

        self.current_sync.lvars.insert(lvar_data.var_name.to_string(), lvar_data.var.floating);
    }

    // Processes client data and adds to the result queue if it changed
    pub fn process_client_data(&mut self, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) {
        // Guard clauses
        // Get var data
        let lvar = match self.lvarstransfer.process_client_data(data) {
            Some(var) => var,
            None => return
        };

        match lvar {
            LVarResult::Single(result) => self.process_lvar(result),
            LVarResult::Multi(results) => {
                for result in results {
                    self.process_lvar(result);
                }
            }
        }
    }

    // Processes event data name and the additional dword data
    pub fn process_event_data(&mut self, data: &simconnect::SIMCONNECT_RECV_EVENT) {
        if data.uGroupID != self.events.group_id {return}

        let event_name = self.events.match_event_id(data.uEventID);

        // Check timer
        if let Some(timer) = self.last_written.get(event_name) {
            if timer.elapsed().as_secs() < 1 {return}
        };

        self.current_sync.events.insert(event_name.clone(), data.dwData);
    }

    // Process changed aircraft variables and update SyncActions related to it
    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        if self.avarstransfer.define_id != data.dwDefineID {return}
        
        // Data might be bad/config files don't line up
        if let Ok(data) = self.avarstransfer.read_vars(data) {
            // Update all syncactions with the changed values
            for (var_name, value) in data {
                // Set current var syncactions
                if let Some(actions) = self.action_maps.get_mut(&var_name) {

                    for action in actions {
                        match value {
                            VarReaderTypes::Bool(value) => match action {
                                ActionType::BoolAction(action) => {
                                    action.set_current(value);
                                }
                                _ => {}
                            }

                            VarReaderTypes::I32(value) => match action {
                                ActionType::NumAction(action) | ActionType::FreqSwapAction(action) => {
                                    action.set_current(value);
                                }
                                _ => {}
                            }

                            VarReaderTypes::F64(value) => match action {
                                ActionType::NumFloatAction(action) => {
                                    action.set_current(value);
                                }
                                _ => {}
                            }

                            _ => {}
                        }
                    }
                    
                }
    
                // Determine if this variable should be updated
                let mut should_write = true;
                if let Some(period) = self.periods.get_mut(&var_name) {
                    should_write = period.do_update();
                }

                if let Some(last_time) = self.last_written.get(&var_name) {
                    should_write = should_write && last_time.elapsed().as_secs() > 1
                }

                if should_write {
                    // Queue data for reading
                    self.current_sync.avars.insert(var_name, value);      
                }
            }

        }
    }

    pub fn step_interpolate(&mut self, conn: &SimConnector) {
        // Interpolate AVARS
        let aircraft_interpolation_data = self.interpolation_avars.step();
        if aircraft_interpolation_data.len() > 0 {
            self.write_aircraft_data_unchecked(conn, &aircraft_interpolation_data);
        }
    }

    pub fn get_need_sync(&mut self, sync_permission: &SyncPermissions) -> Option<AllNeedSync> {
        if self.sync_vars.is_empty() {return None}

        let mut return_data = AllNeedSync::new();

        std::mem::swap(&mut return_data, &mut self.current_sync);
        return_data = self.filter_all_sync(return_data, sync_permission);

        if return_data.is_empty() {return None}
        
        return Some(return_data);
    }

    // Skip checking with self.sync_vars and creating a new hashmap - used for interpolation
    fn write_aircraft_data_unchecked(&mut self, conn: &SimConnector, data: &AVarMap) {
        if data.len() == 0 {return}
        self.avarstransfer.set_vars(conn, data);
    }

    fn can_sync(&self, var_name: &str, sync_permission: &SyncPermissions) -> bool {
        // Check categories
        if let Some(category) = self.categories.get(var_name) {
            if *category == Category::Server && (*sync_permission == SyncPermissions::Server || *sync_permission == SyncPermissions::ServerAndMaster) {
                return true
            } else if *category == Category::Shared {
                return true
            } else if * category == Category::Master && (*sync_permission == SyncPermissions::Master || *sync_permission == SyncPermissions::ServerAndMaster) {
                return true
            }
            return false
        }
        return true
    }

    fn filter_all_sync(&self, all_sync: AllNeedSync, sync_permission: &SyncPermissions) -> AllNeedSync {
        let mut return_data = AllNeedSync::new();

        for (name, data) in all_sync.avars.into_iter() {
            if !self.can_sync(&name, sync_permission) {continue;}
            return_data.avars.insert(name, data);
        }

        for (name, data) in all_sync.lvars.into_iter() {
            if !self.can_sync(&name, sync_permission) {continue;}
            return_data.lvars.insert(name, data);
        }

        for (name, data) in all_sync.events.into_iter() {
            if !self.can_sync(&name, sync_permission) {continue;}
            return_data.events.insert(name, data);
        }

        return_data
    }

    pub fn write_aircraft_data(&mut self, conn: &SimConnector, data: &AVarMap, interpolate: bool) {
        if data.len() == 0 {return}

        let mut to_sync = AVarMap::new();
        to_sync.reserve(data.len());
        
        // Only sync vars that are defined as so
        for (var_name, data) in data {
            self.last_written.insert(var_name.to_string(), Instant::now());
            // Can directly be set through SetDataOnSimObject
            if self.sync_vars.contains(var_name) {

                if interpolate && self.interpolate_names.contains(var_name) {
                    // Queue data for interpolation
                    if let VarReaderTypes::F64(value) = data {
                        self.interpolation_avars.queue_interpolate(&var_name, *value)
                    }
                } else {
                    // Set data right away
                    to_sync.insert(var_name.clone(), data.clone());
                }
                
            // Needs to be set using an event
            } else {
                // Otherwise sync them using defined events
                if let Some(actions) = self.action_maps.get_mut(var_name) {
                    for action in actions {

                        match data {
                            VarReaderTypes::Bool(value) => match action {
                                ActionType::BoolAction(action) => {
                                    action.set_new(*value, conn)
                                }
                                _ => {}
                            }

                            VarReaderTypes::I32(value) => match action {
                                // Format of INDEX VALUE (>K:2:NAME)
                                ActionType::NumSetWithIndex(action) => {
                                    self.lvarstransfer.set_unchecked(conn, &format!("K:2:{}", action.event_name), Some(""), &format!("{} {}", action.index_param, value * action.multiply_by.unwrap_or(1)));
                                }
                                ActionType::NumAction(action) => {
                                    action.set_new(*value, conn);
                                }
                                _ => {}
                            }

                            VarReaderTypes::F64(value) => match action {
                                ActionType::FloatAction(action) => {
                                    self.lvarstransfer.set_unchecked(conn, &format!("K:{}", action.event_name), Some(""), &value.to_string());
                                }
                                ActionType::NumFloatAction(action) => {
                                    action.set_new(*value, conn);
                                }
                                _ => {}
                            }

                            _ => {}
                        }
                        
                    }
                }
            }
        }

        if to_sync.len() == 0 {return;}

        self.avarstransfer.set_vars(conn, &to_sync);
    }

    pub fn write_local_data(&mut self, conn: &SimConnector, data: &LVarMap, interpolate: bool) {
        for (var_name, value) in data {
            if interpolate && self.interpolate_names.contains(var_name) {
                self.interpolation_lvars.queue_interpolate(var_name, *value);
            } else {
                self.lvarstransfer.set(conn, var_name, value.to_string().as_ref())
            }
            self.last_written.insert(var_name.to_string(), Instant::now());
        }
    }

    pub fn write_event_data(&mut self, conn: &SimConnector, data: &EventMap) {
        for (event_name, value) in data {
            self.events.trigger_event(conn, event_name, *value as u32);

            self.last_written.insert(event_name.clone(), Instant::now());
        }
    }

    pub fn on_receive_data(&mut self, conn: &SimConnector, data: AllNeedSync, sync_permission: &SyncPermissions, interpolate: bool) {
        let data = self.filter_all_sync(data, sync_permission);

        self.write_aircraft_data(conn, &data.avars, interpolate);
        self.write_local_data(conn, &data.lvars,  interpolate);
        self.write_event_data(conn, &data.events);
    }

    // To be called when SimConnect connects
    pub fn on_connected(&mut self, conn: &SimConnector) {
        self.interpolation_avars.reset();

        self.avarstransfer.on_connected(conn);
        self.events.on_connected(conn);
        self.lvarstransfer.on_connected(conn);

        conn.request_data_on_sim_object(0, self.avarstransfer.define_id, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);
    }

    pub fn get_all_current(&self) -> AllNeedSync {
        AllNeedSync {
            avars: self.avarstransfer.get_all_vars().clone(),
            lvars: self.lvarstransfer.get_all_vars(),
            events: EventMap::new(),
        }
    }

    pub fn clear_sync(&mut self) {
        self.current_sync.clear();
    }

    pub fn get_number_avars(&self) -> usize {
        return self.avarstransfer.get_number_defined()
    }

    pub fn get_number_lvars(&self) -> usize {
        return self.lvarstransfer.get_number_defined()
    }

    pub fn get_number_events(&self) -> usize {
        return self.events.get_number_defined()
    }

    pub fn reset_interpolate(&mut self) {
        self.interpolation_avars.reset();
        self.interpolation_lvars.reset();
    }
}