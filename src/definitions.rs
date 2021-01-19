use indexmap::{IndexMap};
use serde_yaml::{self, Value};
use serde::{Deserialize, Serialize};
use simconnect::SimConnector;

use std::{collections::HashMap, collections::{HashSet, hash_map}, fmt::{Debug, Display}, fs::File, path::{Path}, time::Instant};
use crate::{sync::{gaugecommunicator::{GetResult, InterpolateData, LVarResult, InterpolationType}, jscommunicator::{self, JSCommunicator}, transfer::{AircraftVars, Events, LVarSyncer}}, syncdefs::{CustomCalculator, NumDigitSet, NumIncrement, NumSet, Syncable, ToggleSwitch}, util::Category, util::InDataTypes, util::VarReaderTypes, velocity::VelocityCorrector};

#[derive(Debug)]
pub enum ConfigLoadError {
    FileError(std::io::Error),
    YamlError(serde_yaml::Error, String),
    ParseError(VarAddError, String),
    ParseBytesError(VarAddError),
    InvalidBytes(rmp_serde::decode::Error)
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::FileError(e) => write!(f, "Could not open file...{}", e),
            ConfigLoadError::YamlError(e, file_name) => write!(f, "Could not parse {} as YAML...{}", file_name, e),
            ConfigLoadError::ParseError(e, file_name) => write!(f, "Error parsing configuration in {}...{}", file_name, e),
            ConfigLoadError::ParseBytesError(e) => write!(f, "Error parsing bytes configuration...{}", e),
            ConfigLoadError::InvalidBytes(e) => write!(f, "Could not parse bytes as YAML...{}", e),
        }
    }
}

pub enum WriteDataError {
    MissingMapping(String)
}

impl Display for WriteDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteDataError::MissingMapping(mapping_name) => write!(f, "No definition exists for {}. Do you have matching .yaml files?", mapping_name),
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
    ($value: expr) => {
        match serde_yaml::from_value($value) {
            Ok(y) => y,
            Err(e) => return Err(VarAddError::YamlParseError(e))
        }
    }
}

macro_rules! execute_mapping {
    ($new_value_name: ident, $action_name: ident, $new_value: expr, $mapping: expr, $action: block, $var_only_action: block, $program_action: block) => {
        match $new_value {
            VarReaderTypes::Bool($new_value_name) => match &mut $mapping.action {
                ActionType::Bool($action_name) => $action
                ActionType::ProgramAction($action_name) => $program_action
                ActionType::VarOnly => $var_only_action
                _ => {}
            }

            VarReaderTypes::I32($new_value_name) => match &mut $mapping.action {
                ActionType::I32($action_name) => $action
                ActionType::ProgramAction($action_name) => $program_action
                ActionType::VarOnly => $var_only_action
                _ => {}
            }

            VarReaderTypes::F64($new_value_name) => match &mut $mapping.action {
                ActionType::F64($action_name) => $action
                ActionType::ProgramAction($action_name) => $program_action
                ActionType::VarOnly => $var_only_action
                _ => {}
            }
            _ => {}
        }
    }
}

fn get_data_type_from_string(string: &str) -> Result<InDataTypes, VarAddError> {
    Ok(
        match string {
            "i32" => InDataTypes::I32,
            "f64" => InDataTypes::F64,
            "bool" => InDataTypes::Bool,
            _ => return Err(VarAddError::MissingField("var_type"))
        }
    )
}

fn evalute_condition_values(condition: &Condition, value: &VarReaderTypes) -> bool {
    if let Some(data) = condition.equals {
        return data == *value
    }

    if let Some(data) = condition.greater_than {
        return data > *value
    }

    if let Some(data) = condition.less_than {
        return data < *value
    }

    return false
}

fn evalute_condition(lvarstransfer: &LVarSyncer, avarstransfer: &AircraftVars, condition: Option<&Condition>, incoming_value: &VarReaderTypes) -> bool {
    let condition = match condition {
        Some(condition) => condition,
        None => return true
    };

    if let Some(var_data) = condition.var.as_ref() {
        if var_data.var_name.starts_with("L:") {
            if let Some(value) = lvarstransfer.get_var(&var_data.var_name) {
                return evalute_condition_values(condition, &VarReaderTypes::F64(value))
            }
        } else {
            if let Some(value) = avarstransfer.get_var(&var_data.var_name) {
                return evalute_condition_values(condition, value)
            }
        }
    } else {
        return evalute_condition_values(condition, incoming_value)
    }
    
    false
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventTriggered {
    pub event_name: String,
    pub data: u32,
}

// Name of aircraft variable and the value of it
pub type AVarMap = HashMap<String, VarReaderTypes>;
// Name of local variable and the value of it
pub type LVarMap = HashMap<String, f64>;
// Name of the event the DWORD data associated with it with how many times it got triggered (not a map as the event could've got triggered multiple times before the data could get send)
pub type EventData = Vec<EventTriggered>;

#[derive(Debug)]
enum VarType {
    AircraftVar,
    LocalVar
}

#[derive(Debug)]
pub struct SyncPermission {
    pub is_server: bool,
    pub is_master: bool,
    pub is_init: bool
}

// Serde types
// Describes how an aircraft variable can be set using a SimEvent
// Describes an event to be listened to for fires
#[derive(Deserialize)]
struct EventEntry {
    event_name: String
}

#[derive(Deserialize)]
struct VarData {
    var_name: String,
    var_units: Option<String>,
    var_type: InDataTypes,
}

#[derive(Deserialize)]
struct Condition {
    var: Option<VarData>,
    equals: Option<VarReaderTypes>,
    greater_than: Option<VarReaderTypes>,
    less_than: Option<VarReaderTypes>
}

// Describes an aircraft variable to listen for changes
#[derive(Deserialize)]
struct VarEntry {
    var_name: String,
    var_units: Option<String>,
    var_type: InDataTypes,
    update_every: Option<f64>,
    condition: Option<Condition>,
    interpolate: Option<InterpolationType>,
    #[serde(default)]
    constant: bool,
    #[serde(default)]
    unreliable: bool,
}

#[derive(Deserialize)]
struct ToggleSwitchGenericEntry {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    off_event_name: Option<String>,
    event_param: Option<u32>,
    #[serde(default)]
    switch_on: bool,
    #[serde(default)]
    use_calculator: bool,
    condition: Option<Condition>
}

#[derive(Deserialize)]
struct NumSetGenericEntry<T> {
    var_name: String,
    var_units: Option<String>,
    event_name: String,
    event_param: Option<u32>,
    multiply_by: Option<T>,
    add_by: Option<T>,
    interpolate: Option<InterpolationType>,
    #[serde(default)]
    use_calculator: bool,
    #[serde(default)]
    index_reversed: bool,
    // The event to call after the number is set
    swap_event_name: Option<String>,
    #[serde(default)]
    unreliable: bool,
}

#[derive(Deserialize)]
struct NumIncrementEntry<T> {
    var_name: String,
    var_units: Option<String>,
    up_event_name: String,
    down_event_name: String,
    increment_by: T,
    #[serde(default)]
    // If the difference of the values can be passed as a param in order to only make one event call
    pass_difference: bool,
}


#[derive(Deserialize)]
struct NumDigitSetEntry {
    var_name: String,
    var_units: Option<String>,
    up_event_names: Vec<String>,
    down_event_names: Vec<String>,
    condition: Option<Condition>
}

#[derive(Deserialize)]
struct CustomCalculatorEntry {
    get: String,
    set: String,
    condition: Option<Condition>
}

#[derive(Deserialize)]
struct ProgramActionEntry {
    var_name: String,
    var_units: Option<String>,
    var_type: InDataTypes,
    condition: Condition,
    action: ProgramAction
}

#[derive(Deserialize)]
enum ProgramAction {
    TakeControls
}

// The struct that get_need_sync returns. Holds all the aircraft/local variables and events that have changed since the last call.
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct AllNeedSync {
    pub avars: AVarMap,
    pub lvars: LVarMap,
    pub events: EventData
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

    // Filter the variables
    pub fn filter<F>(&mut self, filter_fn: F) where F: Fn(&str) -> bool {
        self.filter_keep(filter_fn);
    }

    // Keeps variables that matches the filter, returns the variables that don't
    pub fn filter_keep<F>(&mut self, filter_fn: F) -> AllNeedSync where F: Fn(&str) -> bool {
        let mut filtered = AllNeedSync::new();

        self.avars.retain(|name, var| {
            if filter_fn(&name) {return true;}
            filtered.avars.insert(name.clone(), var.clone());
            return false;
        });

        self.lvars.retain(|name, var| {
            if filter_fn(&name) {return true;}
            filtered.lvars.insert(name.clone(), var.clone()); 
            return false;
        });

        self.events.retain(|event| {
            if filter_fn(&event.event_name) {return true;}
            filtered.events.push(event.clone()); 
            return false;
            
        });

        return filtered;
    }
}

enum ActionType {
    F64(Box<dyn Syncable<f64>>),
    I32(Box<dyn Syncable<i32>>),
    Bool(Box<dyn Syncable<bool>>),
    ProgramAction(ProgramAction),
    VarOnly
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

struct Mapping {
    action: ActionType,
    condition: Option<Condition>
}

pub struct Definitions {
    // Serializable vec that houses all the definitions that can be sent over the network
    definitions_buffer: IndexMap<String, Vec<Value>>,
    // Data that can be synced using booleans (ToggleSwitch, ToggleSwitchParam)
    mappings: HashMap<String, Vec<Mapping>>,
    // Events to listen to
    events: Events,
    // Helper struct to execute calculator events
    lvarstransfer: LVarSyncer,
    // Helper struct to retrieve/set vars not settable in SimConnect
    jstransfer: JSCommunicator,
    // Helper struct to retrieve *changed* aircraft variables using the CHANGED and TAGGED flags in SimConnect
    avarstransfer: AircraftVars,
    // Categories for every mapping
    categories: HashMap<String, Category>,
    // Vars that shouldn't update every tick
    periods: HashMap<String, Period>,
    // Value to hold the current queue
    current_sync: AllNeedSync,
    last_written: HashMap<String, Instant>,
    // Vars that shouldn't be sent reliably
    unreliable_vars: HashSet<String>,
    // Helper struct to correct aircraft velocity
    velocity_corrector: VelocityCorrector,
    // Vars that should not be sent over the network
    do_not_sync: HashSet<String>,
    // Vars that need interpolation
    interpolate_vars: HashSet<String>,
    // For indicating that an event has been triggered and the control should be transferred to the next person
    pub control_transfer_requested: bool
}

fn get_category_from_string(category: &str) -> Result<Category, VarAddError> {
    match category.to_lowercase().as_str() {
        "shared" => Ok(Category::Shared),
        "master" => Ok(Category::Master),
        "server" => Ok(Category::Server),
        "init" => Ok(Category::Init),
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
            definitions_buffer: IndexMap::new(),
            mappings: HashMap::new(),
            events: Events::new(1),
            lvarstransfer: LVarSyncer::new(),
            jstransfer: JSCommunicator::new(),
            avarstransfer: AircraftVars::new(1),

            last_written: HashMap::new(),

            current_sync: AllNeedSync::new(),

            unreliable_vars: HashSet::new(),
            velocity_corrector: VelocityCorrector::new(2),
            do_not_sync: HashSet::new(),
            
            categories: HashMap::new(),
            periods: HashMap::new(),
            interpolate_vars: HashSet::new(),

            control_transfer_requested: false,
        }
    }

    fn add_var(&mut self, category: &str, var: VarEntry) -> Result<(), VarAddError> {
        let (var_name, var_type) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), var.var_type)?;

        // Handle interpolation for this variable
        if let Some(interpolate) = var.interpolate {
            self.interpolate_vars.insert(var_name.clone());
            
            if std::matches!(var_type, VarType::AircraftVar) {
                self.lvarstransfer.transfer.add_interpolate_mapping(&var.var_name, var_name.clone(), var.var_units.as_deref(), interpolate);
            }
        }

        if var.unreliable {
            self.unreliable_vars.insert(var_name.clone());
        }
        
        // Handle custom periods
        if let Some(period) = var.update_every {
            self.periods.insert(var_name.clone(), Period::new(period));
        }

        self.add_mapping(var_name, ActionType::VarOnly, var.condition)?;

        Ok(())
    }

    fn add_event(&mut self, category: &str, event: EventEntry) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.events.get_or_map_event_id(&event.event_name, true);
        self.categories.insert(event.event_name.clone(), category);

        Ok(())
    }
    
    fn add_aircraft_variable(&mut self, category: &str, var_name: &str, var_units: &str, var_type: InDataTypes) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.avarstransfer.add_var(var_name, var_units, var_type);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    fn add_local_variable(&mut self, category: &str, var_name: &str, var_units: Option<&str>) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;

        self.lvarstransfer.add_var(var_name.to_string(), var_units.map(String::from));
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    // Determines whether to add an aircraft variable or local variable based off the variable name
    fn add_var_string(&mut self, category: &str, var_name: &str, var_units: Option<&str>, var_type: InDataTypes) -> Result<(String, VarType), VarAddError> {
        if var_name.starts_with("L:") {
            // Keep var_name with L: in it to pass to execute_calculator code
            self.add_local_variable(category, var_name, var_units)?;

            return Ok((var_name.to_string(), VarType::LocalVar))

        } else {
            let actual_var_name = get_real_var_name(var_name);

            if let Some(var_units) = var_units {
                self.add_aircraft_variable(category, &actual_var_name, var_units, var_type)?;
            } else {
                return Err(VarAddError::MissingField("var_units"))
            }

            return Ok((actual_var_name, VarType::AircraftVar))
        }
    }

    fn add_mapping(&mut self, var_name: String, mapping: ActionType, condition: Option<Condition>) -> Result<(), VarAddError> {
        let mut condition = condition;

        if let Some(condition) = condition.as_mut() {
            if let Some(var_data) = condition.var.as_mut() {
                let (var_string, _) = self.add_var_string("shared", &var_data.var_name, var_data.var_units.as_deref(), var_data.var_type)?;
                var_data.var_name = var_string.clone();
                self.do_not_sync.insert(var_string);
            }
        }
        
        let mapping = Mapping {
            action: mapping,
            condition: condition,
        };

        match self.mappings.entry(var_name.to_string()) {
            hash_map::Entry::Occupied(mut o) => { 
                o.get_mut().push(mapping)
            }
            hash_map::Entry::Vacant(v) => { v.insert(vec![mapping]); }
        };

        Ok(())
    }

    fn add_toggle_switch(&mut self, category: &str, var: ToggleSwitchGenericEntry) -> Result<(), VarAddError> { 
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), InDataTypes::Bool)?;

        let mut action = ToggleSwitch::new(event_id);

        // Set optional features
        if var.use_calculator {
            action.set_calculator_event_name(Some(&var.event_name));
        }

        if let Some(off_event) = var.off_event_name.as_ref() {
            let off_event_id = self.events.get_or_map_event_id(off_event, false);
            action.set_off_event(off_event_id);
        }
        
        if let Some(event_param) = var.event_param {
            action.set_param(event_param);
        }

        action.set_switch_on(var.switch_on);

        self.add_mapping(var_string, ActionType::Bool(Box::new(action)), var.condition)?;

        Ok(())
    }

    fn add_num_set_generic<T>(&mut self, data_type: InDataTypes, category: &str, var: NumSetGenericEntry<T>) -> Result<(Option<Box<NumSet<T>>>, String), VarAddError> where T: Default {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), data_type)?;

        if let Some(interpolate_type) = var.interpolate {

            self.lvarstransfer.transfer.add_interpolate_mapping(&format!("K:{}", &var.event_name), var_string.clone(), var.var_units.as_deref(), interpolate_type);
            self.interpolate_vars.insert(var_string.clone());
            self.add_mapping(var_string.clone(), ActionType::VarOnly, None)?;

        } else {

            let mut action = Box::new(NumSet::new(event_id));

            if var.unreliable {
                self.unreliable_vars.insert(var.var_name.clone());
            }
    
            if var.use_calculator || var.event_param.is_some() {
                action.set_calculator_event_name(Some(&var.event_name), var.event_param.is_some())
            }
    
            if let Some(event_param) = var.event_param {
                action.set_param(event_param, var.index_reversed);
            }
    
            if let Some(multiply_by) = var.multiply_by {
                action.set_multiply_by(multiply_by);
            }
    
            if let Some(add_by) = var.add_by {
                action.set_add_by(add_by);
            }
    
            if let Some(swap_event) = var.swap_event_name.as_ref() {
                let swap_event_id = self.events.get_or_map_event_id(swap_event, false);
                action.set_swap_event(swap_event_id);
            }

            return Ok((Some(action), var_string))

        }
        
        return Ok((None, var_string))
    }

    fn add_num_set(&mut self, category: &str, var: Value) -> Result<(), VarAddError> {
        let data_type_string: &str = check_and_return_field!("var_type", var, str);
        let data_type = get_data_type_from_string(data_type_string)?;

        let condition = try_cast_yaml!(var["condition"].clone());
        
        match data_type {
            InDataTypes::I32 => {
                let (mapping, var_string) = self.add_num_set_generic::<i32>(data_type, category, try_cast_yaml!(var))?;
                if let Some(mapping) = mapping {
                    self.add_mapping(var_string, ActionType::I32(mapping), condition)?
                }
            }
            InDataTypes::F64 => {
                let (mapping, var_string) = self.add_num_set_generic::<f64>(data_type, category, try_cast_yaml!(var))?;
                if let Some(mapping) = mapping {
                    self.add_mapping(var_string, ActionType::F64(mapping), condition)?
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn add_num_increment_generic<T>(&mut self, data_type: InDataTypes, category: &str, var: NumIncrementEntry<T>) -> Result<(Box<NumIncrement<T>>, String), VarAddError> where T: Default {
        let up_event_id = self.events.get_or_map_event_id(&var.up_event_name, false);
        let down_event_id = self.events.get_or_map_event_id(&var.down_event_name, false);

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), data_type)?;

        let mut mapping = NumIncrement::new(up_event_id, down_event_id, var.increment_by);
        mapping.set_pass_difference(var.pass_difference);

        Ok((Box::new(mapping), var_string))
    }

    fn add_num_increment(&mut self, category: &str, var: Value) -> Result<(), VarAddError> {
        let data_type_string: &str = check_and_return_field!("var_type", var, str);
        let data_type = get_data_type_from_string(data_type_string)?;

        let condition = try_cast_yaml!(var["condition"].clone());

        match data_type {
            InDataTypes::I32 => {
                let (mapping, var_string) = self.add_num_increment_generic::<i32>(data_type, category, try_cast_yaml!(var))?;
                self.add_mapping(var_string, ActionType::I32(mapping), condition)?;
            }
            InDataTypes::F64 => {
                let (mapping, var_string) = self.add_num_increment_generic::<f64>(data_type, category, try_cast_yaml!(var))?;
                self.add_mapping(var_string, ActionType::F64(mapping), condition)?;
            }
            _ => {}
        };

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
        self.add_mapping(var_string, ActionType::I32(Box::new(NumDigitSet::new(up_event_ids, down_event_ids))), var.condition)?;

        Ok(())
    }

    fn add_custom_calculator(&mut self, category: &str, var: CustomCalculatorEntry) -> Result<(), VarAddError> {
        let category = get_category_from_string(category)?;
        
        let var_name = self.lvarstransfer.add_custom_var(var.get);

        self.categories.insert(var_name.clone(), category);
        self.add_mapping(var_name, ActionType::F64(Box::new(CustomCalculator::new(var.set))), var.condition)?;

        Ok(())
    }

    fn add_program_action(&mut self, category: &str, var: ProgramActionEntry) -> Result<(), VarAddError> {

        let (var_string, _) = self.add_var_string(category, &var.var_name, var.var_units.as_deref(), var.var_type)?;
        self.add_mapping(var_string, ActionType::ProgramAction(var.action), Some(var.condition))?;

        Ok(())
    }

    fn add_to_buffer(&mut self, category: String, value: Value) {
        match self.definitions_buffer.entry(category) {
            indexmap::map::Entry::Occupied(mut o) => {o.get_mut().push(value)},
            indexmap::map::Entry::Vacant(v) => {v.insert(vec![value]);}
        };
    }

    pub fn get_buffer_bytes(&mut self) -> Vec<u8> {
        rmp_serde::to_vec(&self.definitions_buffer).unwrap()
    }

    // Calls the correct method for the specified "action" type
    fn parse_var(&mut self, category: String, value: Value) -> Result<(), VarAddError> {
        let type_str = check_and_return_field!("type", value, str);

        // self.check_other_common_fields(&value);
        let value_clone = value.clone();


        match type_str.to_uppercase().as_str() {
            "VAR" => self.add_var(&category, try_cast_yaml!(value))?,
            "EVENT" => self.add_event(&category, try_cast_yaml!(value))?,
            "TOGGLESWITCH" => self.add_toggle_switch(&category, try_cast_yaml!(value))?,
            "NUMSET" => self.add_num_set(&category, try_cast_yaml!(value))?,
            "NUMINCREMENT" => self.add_num_increment(&category, try_cast_yaml!(value))?,
            "NUMDIGITSET" => self.add_num_digit_set(&category, try_cast_yaml!(value))?,
            "CUSTOMCALCULATOR" => self.add_custom_calculator(&category, try_cast_yaml!(value))?,
            "PROGRAMACTION" => self.add_program_action(&category, try_cast_yaml!(value))?,
            _ => return Err(VarAddError::InvalidSyncType(type_str.to_string()))
        };
        
        self.add_to_buffer(category, value_clone);

        return Ok(());
    }

    fn shrink_maps(&mut self) {
        self.mappings.shrink_to_fit();
        self.categories.shrink_to_fit();
        self.periods.shrink_to_fit();
        self.unreliable_vars.shrink_to_fit();
        self.do_not_sync.shrink_to_fit();
        self.interpolate_vars.shrink_to_fit();

        self.lvarstransfer.shrink_maps();
        self.events.shrink_maps();
        self.avarstransfer.shrink_maps();
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
                    self.parse_var(key.clone(), var_data)?;
                }
            }
            
        }

        // Shrink all maps
        self.shrink_maps();

        Ok(())
    }

    // Load yaml from file
    pub fn load_config(&mut self, path: impl AsRef<Path> + Display) -> Result<(), ConfigLoadError> {
        let path_string = path.to_string();

        let file = File::open(path)
            .map_err(|e| ConfigLoadError::FileError(e))?;

        let yaml: IndexMap<String, Vec<Value>> = serde_yaml::from_reader(file)
            .map_err(|e| ConfigLoadError::YamlError(e, path_string.clone()))?;

        self.parse_yaml(yaml)
            .map_err(|e| ConfigLoadError::ParseError(e, path_string.clone()))
    }

    pub fn load_config_from_bytes(&mut self, bytes: Box<[u8]>) -> Result<(), ConfigLoadError> {
        let yaml: IndexMap<String, Vec<Value>> = rmp_serde::from_slice(&bytes)
            .map_err(|e| ConfigLoadError::InvalidBytes(e))?;

        self.parse_yaml(yaml)
            .map_err(|e| ConfigLoadError::ParseBytesError(e))
    }

    #[allow(unused_variables)]
    fn process_local_var(&mut self, result: GetResult) {
        let mut should_write = self.did_write_recently(&result.var_name);

        if let Some(mappings) = self.mappings.get_mut(&result.var_name) {
            for mapping in mappings {

                if !evalute_condition(&self.lvarstransfer, &self.avarstransfer, mapping.condition.as_ref(), &VarReaderTypes::F64(result.var.floating)) {continue}

                execute_mapping!(new_value, action, VarReaderTypes::F64(result.var.floating), mapping, {
                    action.set_current(new_value)
                }, {}, {
                    match action {
                        ProgramAction::TakeControls => self.control_transfer_requested = true
                    }
                    should_write = false;
                });
                
            }
        }

        if !should_write {return}
        self.current_sync.lvars.insert(result.var_name, result.var.floating);
    }

    // Processes client data and adds to the result queue if it changed
    pub fn process_client_data(&mut self, conn: &simconnect::SimConnector, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) {
        // Get var data
        let lvar = match self.lvarstransfer.process_client_data(conn ,data) {
            Some(var) => var,
            None => return
        };

        match lvar {
            LVarResult::Single(result) => self.process_local_var(result),
            LVarResult::Multi(results) => {
                for result in results {
                    self.process_local_var(result);
                }
            }
        }
    }

    fn process_js_data(&mut self) {
        if let Some(payload) = self.jstransfer.poll() {
            match payload {
                jscommunicator::Payloads::Interaction { name: interaction_name } => {
                    self.current_sync.events.push(EventTriggered { event_name: interaction_name, data: 0})
                }
                _ => {}
            }
        };
    }

    // Processes event data name and the additional dword data
    pub fn process_event_data(&mut self, data: &simconnect::SIMCONNECT_RECV_EVENT) {
        // Not for us
        if data.uGroupID != self.events.group_id {return}
        
        // Regular KEY event
        let event_name = match self.events.match_event_id(data.uEventID) {
            Some(event_name) => event_name.clone(),
            None => return
        };

        // Check timer
        if self.did_write_recently(&event_name) {return;}

        self.current_sync.events.push(EventTriggered {
            event_name: event_name,
            data: data.dwData,
        });
    }

    // Process changed aircraft variables and update SyncActions related to it
    #[allow(unused_variables)]
    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        self.velocity_corrector.process_sim_object_data(data);

        if self.avarstransfer.define_id != data.dwDefineID {return}
        
        // Data might be bad/config files don't line up
        if let Ok(mut data) = self.avarstransfer.read_vars(data) {
            // Remove wind
            self.velocity_corrector.remove_wind_component(&mut data);
            // Update all syncactions with the changed values
            for (var_name, value) in data {
                // Set current var syncactions
                if let Some(mappings) = self.mappings.get_mut(&var_name) {
                    for mapping in mappings {
                        execute_mapping!(new_value, action, value, mapping, {
                            action.set_current(new_value)
                        }, {}, {});
                    }
                }

                // Determine if this variable should be updated
                let mut should_write = !self.did_write_recently(&var_name) && !self.do_not_sync.contains(&var_name);

                if let Some(period) = self.periods.get_mut(&var_name) {
                    should_write = should_write && period.do_update();
                }

                if should_write {
                    // Queue data for reading
                    self.current_sync.avars.insert(var_name.clone(), value);
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.process_js_data();
    }

    fn filter_all_sync(&self, data: &mut AllNeedSync, sync_permission: &SyncPermission) {
        data.filter(|name| self.can_sync(name, sync_permission));
    }

    fn split_unreliable(&self, data: &mut AllNeedSync) -> AllNeedSync {
        data.filter_keep(|name| self.interpolate_vars.contains(name) || self.unreliable_vars.contains(name))
    }

    pub fn get_need_sync(&mut self, sync_permission: &SyncPermission) -> (Option<AllNeedSync>, Option<AllNeedSync>) {
        let mut data = AllNeedSync::new();
        // Swap queued vars into local var
        std::mem::swap(&mut data, &mut self.current_sync);
        // Filter out based on what the client's current permissions are
        self.filter_all_sync(&mut data, sync_permission);
        // Split into interpolated vs non interpolated values - used for reliable/unreliable transmissions
        let regular = self.split_unreliable(&mut data);
        // Convert into options
        let unreliable = if data.is_empty() {None} else {Some(data)};
        let regular = if regular.is_empty() {None} else {Some(regular)};

        return (unreliable, regular);
    }

    fn can_sync(&self, var_name: &str, sync_permission: &SyncPermission) -> bool {
        // Check categories
        if let Some(category) = self.categories.get(var_name) {
            if *category == Category::Server && sync_permission.is_server {
                return true
            } else if *category == Category::Shared {
                return true
            } else if *category == Category::Master && sync_permission.is_master {
                return true
            } else if *category == Category::Init && sync_permission.is_init {
                return true
            }
            return false
        }
        return true
    }

    fn did_write_recently(&self, data_name: &str) -> bool {
        if let Some(timer) = self.last_written.get(data_name) {
            return timer.elapsed().as_secs() < 1
        } else {
            return false
        }
    }

    #[allow(unused_variables)]
    pub fn write_aircraft_data(&mut self, conn: &SimConnector, data: AVarMap, time: f64, interpolate: bool) {
        if data.len() == 0 {return}

        let mut to_sync = AVarMap::new();
        to_sync.reserve(data.len());

        // Correct velocity
        let mut data = data;
        self.velocity_corrector.add_wind_component(&mut data);

        let mut interpolation_data = Vec::new();

        // Only sync vars that are defined as so
        for (var_name, data) in data {
            self.last_written.insert(var_name.to_string(), Instant::now());

            // Otherwise sync them using defined events
            if let Some(mappings) = self.mappings.get_mut(&var_name) {
                for mapping in mappings {
                    if !evalute_condition(&self.lvarstransfer, &self.avarstransfer, mapping.condition.as_ref(), &data) {continue}

                    execute_mapping!(new_value, action, data, mapping, {
                        action.set_new(new_value, conn, &mut self.lvarstransfer)
                    }, {
                        if interpolate && self.interpolate_vars.contains(&var_name) {
                            // Queue data for interpolation
                            if let VarReaderTypes::F64(value) = data {
                                interpolation_data.push(InterpolateData {
                                   name: var_name.clone(),
                                   value,
                                   time
                                });
                            }
                        } else {
                                // Set data right away
                            to_sync.insert(var_name.clone(), data.clone());
                        }
                    }, {});
                }
            }
        }

        if interpolation_data.len() > 0 {
            self.lvarstransfer.transfer.send_new_interpolation_data(conn, time, &interpolation_data);
        }

        if to_sync.len() > 0 {
            self.avarstransfer.set_vars(conn, &to_sync);
        }
    }

    #[allow(unused_variables)]
    pub fn write_local_data(&mut self, conn: &SimConnector, data: LVarMap, _interpolate: bool) -> Result<(), WriteDataError> {
        for (var_name, value) in data {

            match self.mappings.get_mut(&var_name) {
                Some(mappings) => {
                    for mapping in mappings {

                        if !evalute_condition(&self.lvarstransfer, &self.avarstransfer, mapping.condition.as_ref(), &VarReaderTypes::F64(value)) {continue}
        
                        execute_mapping!(new_value, action, VarReaderTypes::F64(value), mapping, {
                            action.set_new(new_value, conn, &mut self.lvarstransfer)
                        }, {
                            self.lvarstransfer.set(conn, &var_name, value.to_string().as_ref());
                        }, {});
        
                        self.last_written.insert(var_name.clone(), Instant::now());

                    }
                }
                None => return Err(WriteDataError::MissingMapping(var_name))
            }
        }

        Ok(())
    }

    pub fn write_event_data(&mut self, conn: &SimConnector, data: EventData) -> Result<(), WriteDataError> {
        for event in data {

            if event.event_name.starts_with("H:") {
                // Use gauge to transmit H: event
                self.lvarstransfer.set_unchecked(conn, &event.event_name, None, "");

            } else {
                // Event doesn't exist
                if let Err(()) = self.events.trigger_event(conn, &event.event_name, event.data as u32) {
                    return Err(WriteDataError::MissingMapping(event.event_name))
                };

            }
            
            self.last_written.insert(event.event_name, Instant::now());
        }

        Ok(())
    }

    pub fn on_receive_data(&mut self, conn: &SimConnector, data: AllNeedSync, time: f64, sync_permission: &SyncPermission, interpolate: bool) -> Result<(), WriteDataError> {
        let mut data = data;
        self.filter_all_sync(&mut data, sync_permission);

        // In this specific order
        // Aircraft var data should overwrite any event data
        self.write_event_data(conn, data.events)?;
        self.write_aircraft_data(conn, data.avars, time, interpolate);
        self.write_local_data(conn, data.lvars, interpolate)?;

        Ok(())
    }

    // To be called when SimConnect connects
    pub fn on_connected(&mut self, conn: &SimConnector) -> Result<(), ()> {
        self.avarstransfer.on_connected(conn);
        self.events.on_connected(conn);
        self.lvarstransfer.on_connected(conn);
        self.velocity_corrector.on_connected(conn);

        // Might be running another instance
        self.jstransfer.start()
            .map_err(|_| ())?;

        // Get aircraft data
        conn.request_data_on_sim_object(0, self.avarstransfer.define_id, 0, simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME, simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED, 0, 0, 0);

        Ok(())
    }

    pub fn get_all_current(&self) -> AllNeedSync {
        AllNeedSync {
            avars: self.avarstransfer.get_all_vars().clone(),
            lvars: self.lvarstransfer.get_all_vars(),
            events: EventData::new(),
        }
    }

    pub fn clear_sync(&mut self) {
        self.current_sync.clear();
    }

    pub fn get_number_avars(&self) -> usize {
        return self.avarstransfer.get_number_defined()
    }

    pub fn get_number_events(&self) -> usize {
        return self.events.get_number_defined()
    }

    pub fn get_number_lvars(&self) -> usize {
        return self.lvarstransfer.get_number_defined()
    }
}