use indexmap::IndexMap;
use serde::Deserialize;
use serde_yaml::{self, Value};
use simconnect::SimConnector;
use std::collections::{hash_map, HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display};
use std::fs::File;
use std::mem::swap;
use std::path::Path;
use std::time::Instant;

use crate::sync::gaugecommunicator::{GetResult, InterpolateData, InterpolationType};
use crate::sync::jscommunicator::{JSCommunicator, JSPayloads};
use crate::sync::transfer::{AircraftVars, Events, LVarSyncer};
use crate::syncdefs::MultiplyDifferenceLocalVarSet;
use crate::syncdefs::ResetWhenEquals;
use crate::syncdefs::{
    CustomCalculator, NumDigitSet, NumIncrement, NumSet, Syncable, ToggleSwitch,
};
use crate::util::{Category, InDataTypes};
use crate::{corrector::Corrector, syncdefs::LocalVarProxy};

use yourcontrols_types::{AllNeedSync, Error, Event, EventData, VarMap, VarReaderTypes};

// Checks if a field in a Value exists, otherwise will return an error with the name of the field
macro_rules! check_and_return_field {
    ($field_name:expr, $var:ident, str) => {
        match $var[$field_name].as_str() {
            Some(s) => s,
            None => return Err(Error::MissingField($field_name)),
        }
    };

    ($field_name:expr, $var:ident, i64) => {
        match $var[$field_name].as_i64() {
            Some(s) => s,
            None => return Err(Error::MissingField($field_name)),
        };
    };
}

// Tries to cast the value into a Yaml object, returns an error if failed
macro_rules! try_cast_yaml {
    ($value: expr) => {
        match serde_yaml::from_value($value) {
            Ok(y) => y,
            Err(e) => return Err(Error::YamlError(e, String::new())),
        }
    };
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

fn set_did_write_recently(map: &mut HashMap<String, Instant>, data_name: &str) {
    if let Some(instant) = map.get_mut(data_name) {
        *instant = Instant::now();
    } else {
        map.insert(data_name.to_string(), Instant::now());
    }
}

fn check_did_write_recently(map: &mut HashMap<String, Instant>, data_name: &str) -> bool {
    return map
        .get(data_name)
        .map(|x| x.elapsed().as_secs() < 1)
        .unwrap_or(false);
}

fn get_data_type_from_string(string: &str) -> Result<InDataTypes, Error> {
    Ok(match string {
        "i32" => InDataTypes::I32,
        "f64" => InDataTypes::F64,
        "bool" => InDataTypes::Bool,
        _ => return Err(Error::MissingField("var_type")),
    })
}

fn evalute_condition_values(condition: &Condition, value: &VarReaderTypes) -> bool {
    if let Some(data) = condition.equals {
        return *value == data;
    }

    if let Some(data) = condition.greater_than {
        return *value > data;
    }

    if let Some(data) = condition.less_than {
        return *value < data;
    }

    false
}

fn evalute_condition(
    lvarstransfer: &LVarSyncer,
    avarstransfer: &AircraftVars,
    condition: &Condition,
    incoming_value: &VarReaderTypes,
    other_incoming_values: Option<&HashMap<String, VarReaderTypes>>,
) -> bool {
    let var_data = match condition.var.as_ref() {
        Some(v) => v,
        None => return evalute_condition_values(condition, incoming_value),
    };

    // Check other incoming values for presence of condition var
    if let Some(other_incoming_values) = other_incoming_values {
        if let Some(data) = other_incoming_values.get(&var_data.var_name) {
            return evalute_condition_values(condition, data);
        }
    }

    if var_data.var_name.starts_with("L:") {
        lvarstransfer
            .get_var(&var_data.var_name)
            .map(|x| evalute_condition_values(condition, &VarReaderTypes::F64(x)))
            .unwrap_or(true)
    } else {
        avarstransfer
            .get_var(&var_data.var_name)
            .map(|x| evalute_condition_values(condition, x))
            .unwrap_or(true)
    }
}

fn evaluate_conditions(
    lvarstransfer: &LVarSyncer,
    avarstransfer: &AircraftVars,
    condition: Option<&Condition>,
    incoming_value: &VarReaderTypes,
    other_incoming_values: Option<&HashMap<String, VarReaderTypes>>,
) -> bool {
    let condition = match condition {
        Some(c) => c,
        None => return true,
    };

    let mut satisfied = evalute_condition(
        lvarstransfer,
        avarstransfer,
        condition,
        incoming_value,
        other_incoming_values,
    );

    if let Some(other) = &condition.other {
        match other.as_ref() {
            ConditionExpression::And(c) => {
                satisfied &= evalute_condition(
                    lvarstransfer,
                    avarstransfer,
                    c,
                    incoming_value,
                    other_incoming_values,
                )
            }
            ConditionExpression::Or(c) => {
                satisfied |= evalute_condition(
                    lvarstransfer,
                    avarstransfer,
                    c,
                    incoming_value,
                    other_incoming_values,
                )
            }
        };
    }

    if !satisfied {
        return false;
    }

    true
}

#[derive(Debug)]
enum VarType {
    AircraftVar,
    LocalVar,
}

#[derive(Debug)]
pub struct SyncPermission {
    pub is_server: bool,
    pub is_master: bool,
    pub is_init: bool,
}

// Serde types
#[derive(Deserialize)]
struct EventEntry {
    event_name: String,
    #[serde(default)]
    use_calculator: bool,
    #[serde(default)]
    cancel_h_events: bool,
    condition: Option<Condition>,
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
    less_than: Option<VarReaderTypes>,
    #[serde(flatten)]
    other: Option<Box<ConditionExpression>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConditionExpression {
    And(Condition),
    Or(Condition),
}

// Describes an aircraft variable to listen for changes
#[derive(Deserialize)]
struct VarEntry {
    var_name: String,
    var_units: Option<String>,
    var_type: Option<InDataTypes>,
    update_every: Option<f64>,
    condition: Option<Condition>,
    interpolate: Option<InterpolationType>,
    #[serde(default)]
    unreliable: bool,
    #[serde(default)]
    cancel_h_events: bool,
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
    on_condition_value: Option<f64>,
    condition: Option<Condition>,
    #[serde(default)]
    cancel_h_events: bool,
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
    is_user_event: bool,
    #[serde(default)]
    index_reversed: bool,
    condition: Option<Condition>,
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
    up_event_param: Option<T>,
    down_event_name: String,
    down_event_param: Option<T>,
    increment_by: T,
    // A condition object is manually parsed in this implementation
    // condition: Option<Condition>,
    #[serde(default)]
    // If the difference of the values can be passed as a param in order to only make one event call
    pass_difference: bool,
    #[serde(default)]
    // Whether to transmit the client event as a user/aircraft event
    is_user_event: bool,
    // Whether to use execute_calculator_code to transmit the event
    #[serde(default)]
    use_calculator: bool,
}

#[derive(Deserialize)]
struct NumDigitSetEntry {
    var_name: String,
    var_units: Option<String>,
    up_event_names: Vec<String>,
    down_event_names: Vec<String>,
    condition: Option<Condition>,
    #[serde(default)]
    cancel_h_events: bool,
}

#[derive(Deserialize)]
struct CustomCalculatorEntry {
    get: String,
    set: String,
    condition: Option<Condition>,
    #[serde(default)]
    cancel_h_events: bool,
}

#[derive(Deserialize)]
struct LocalVarProxyEntry {
    var_name: String,
    target: String,
    #[serde(default)]
    loopback: bool,
    condition: Option<Condition>,
}

#[derive(Deserialize)]
struct MultiplyDifferenceLocalVarEntry {
    var_name: String,
    target: String,
    multiply_by: f64,
    max_val: f64,
    #[serde(default)]
    loopback: bool,
    condition: Option<Condition>,
}

#[derive(Deserialize)]
struct ResetWhenEqualsEntry {
    var_name: String,
    target: String,
    equals: Vec<f64>,
    condition: Option<Condition>,
}

#[derive(Deserialize)]
struct ProgramActionEntry {
    var_name: String,
    var_units: Option<String>,
    var_type: InDataTypes,
    condition: Option<Condition>,
    action: ProgramAction,
}

#[derive(Deserialize)]
struct ProgramActionEventEntry {
    event_name: String,
    action: ProgramAction,
}

#[derive(Deserialize, Clone)]
pub enum ProgramAction {
    TakeControls,
    TransferControls,
}

struct EventMapping {
    use_calculator: bool,
}

enum ActionType {
    F64(Box<dyn Syncable<f64>>),
    I32(Box<dyn Syncable<i32>>),
    Bool(Box<dyn Syncable<bool>>),
    ProgramAction(ProgramAction),
    Event(EventMapping),
    VarOnly,
}

struct Period {
    time: f64,
    last_update: Option<Instant>,
}

impl Period {
    fn new(time: f64) -> Self {
        Self {
            time,
            last_update: None,
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
    condition: Option<Condition>,
    cancel_h_events: bool,
}

impl Default for Mapping {
    fn default() -> Self {
        Self {
            action: ActionType::VarOnly,
            condition: None,
            cancel_h_events: false,
        }
    }
}

pub struct Definitions {
    // Serializable vec that houses all the definitions that can be sent over the network
    definitions_buffer: IndexMap<String, Vec<Value>>,
    // Data that can be synced using booleans (ToggleSwitch, ToggleSwitchParam)
    mappings: HashMap<String, Vec<Mapping>>,
    // Events to listen to
    events: Events,
    // Helper struct to execute calculator events
    pub lvarstransfer: LVarSyncer,
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
    // Keep track of which definitions just got written so we don't sync them again
    last_written: HashMap<String, Instant>,
    // Helper struct to calculate velocity and correct plane/ground altitude
    physics_corrector: Corrector,
    // Delay events by 100ms in order for them to get synced correctly
    event_queue: VecDeque<Event>,
    event_timer: Instant,
    event_cancel_timer: Instant,
    // Vars that shouldn't be sent reliably
    unreliable_vars: HashSet<String>,
    // Vars that should not be sent over the network
    do_not_sync: HashSet<String>,
    // Vars that need interpolation
    interpolate_vars: HashSet<String>,
    // For indicating that an event has been triggered and the control should be transferred to the next person
    pending_action: Option<ProgramAction>,
}

fn get_category_from_string(category: &str) -> Result<Category, Error> {
    match category.to_lowercase().as_str() {
        "shared" => Ok(Category::Shared),
        "master" => Ok(Category::Master),
        "server" => Ok(Category::Server),
        "init" => Ok(Category::Init),
        _ => Err(Error::InvalidCategory(category.to_string())),
    }
}

fn get_real_var_name(var_name: &str) -> String {
    if var_name.as_bytes()[1] == b':' {
        var_name[2..].to_string()
    } else {
        var_name.to_string()
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

            physics_corrector: Corrector::new(2),

            current_sync: AllNeedSync::new(),
            event_queue: VecDeque::new(),
            event_timer: Instant::now(),
            event_cancel_timer: Instant::now(),

            unreliable_vars: HashSet::new(),
            do_not_sync: HashSet::new(),

            categories: HashMap::new(),
            periods: HashMap::new(),
            interpolate_vars: HashSet::new(),

            pending_action: None,
        }
    }

    fn add_var(&mut self, category: &str, var: VarEntry) -> Result<(), Error> {
        let (var_name, var_type) = self.add_var_string(
            category,
            &var.var_name,
            var.var_units.as_deref(),
            var.var_type.unwrap_or(InDataTypes::F64),
        )?;

        // Handle interpolation for this variable
        if let Some(interpolate) = var.interpolate {
            self.interpolate_vars.insert(var_name.clone());

            if std::matches!(var_type, VarType::AircraftVar) {
                self.lvarstransfer.transfer.add_interpolate_mapping(
                    &var.var_name,
                    var_name.clone(),
                    var.var_units.as_deref(),
                    interpolate,
                );
            }
        }

        if var.unreliable {
            self.unreliable_vars.insert(var_name.clone());
        }

        // Handle custom periods
        if let Some(period) = var.update_every {
            self.periods.insert(var_name.clone(), Period::new(period));
        }

        self.add_mapping(
            var_name,
            Mapping {
                action: ActionType::VarOnly,
                condition: var.condition,
                cancel_h_events: var.cancel_h_events,
            },
        )?;

        Ok(())
    }

    fn add_event(&mut self, category: &str, event: EventEntry) -> Result<(), Error> {
        let category = get_category_from_string(category)?;

        self.events.get_or_map_event_id(&event.event_name, true);
        self.categories.insert(event.event_name.clone(), category);

        self.add_mapping(
            event.event_name,
            Mapping {
                action: ActionType::Event(EventMapping {
                    use_calculator: event.use_calculator,
                }),
                condition: event.condition,
                cancel_h_events: event.cancel_h_events,
            },
        )?;

        Ok(())
    }

    fn add_aircraft_variable(
        &mut self,
        category: &str,
        var_name: &str,
        var_units: &str,
        var_type: InDataTypes,
    ) -> Result<(), Error> {
        let category = get_category_from_string(category)?;

        self.avarstransfer.add_var(var_name, var_units, var_type);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    fn add_local_variable(
        &mut self,
        category: &str,
        var_name: &str,
        var_units: Option<&str>,
    ) -> Result<(), Error> {
        let category = get_category_from_string(category)?;

        self.lvarstransfer.add_var(var_name.to_string(), var_units);
        self.categories.insert(var_name.to_string(), category);

        Ok(())
    }

    // Determines whether to add an aircraft variable or local variable based off the variable name
    fn add_var_string(
        &mut self,
        category: &str,
        var_name: &str,
        var_units: Option<&str>,
        var_type: InDataTypes,
    ) -> Result<(String, VarType), Error> {
        if var_name.starts_with("L:") {
            // Keep var_name with L: in it to pass to execute_calculator code
            self.add_local_variable(category, var_name, var_units)?;

            Ok((var_name.to_string(), VarType::LocalVar))
        } else {
            let actual_var_name = get_real_var_name(var_name);

            if let Some(var_units) = var_units {
                self.add_aircraft_variable(category, &actual_var_name, var_units, var_type)?;
            } else {
                return Err(Error::MissingField("var_units"));
            }

            Ok((actual_var_name, VarType::AircraftVar))
        }
    }

    fn process_new_condition(&mut self, condition: &mut Condition) -> Result<(), Error> {
        if let Some(var_data) = &mut condition.var {
            // Add new var to watch for
            let (var_string, _) = self.add_var_string(
                "shared",
                &var_data.var_name,
                var_data.var_units.as_deref(),
                var_data.var_type,
            )?;

            var_data.var_name = var_string;
        }

        Ok(())
    }

    fn add_mapping(&mut self, var_name: String, mapping: Mapping) -> Result<(), Error> {
        let mut mapping = mapping;

        // Conditions
        if let Some(condition) = &mut mapping.condition {
            self.process_new_condition(condition)?;
        }

        // If the var name was already added to do not sync... remove it
        self.do_not_sync.remove(&var_name);

        match self.mappings.entry(var_name.to_string()) {
            hash_map::Entry::Occupied(mut o) => o.get_mut().push(mapping),
            hash_map::Entry::Vacant(v) => {
                v.insert(vec![mapping]);
            }
        };

        Ok(())
    }

    fn add_toggle_switch(
        &mut self,
        category: &str,
        var: ToggleSwitchGenericEntry,
    ) -> Result<(), Error> {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, var_type) = self.add_var_string(
            category,
            &var.var_name,
            var.var_units.as_deref(),
            InDataTypes::Bool,
        )?;

        let mut action = ToggleSwitch::new(event_id);

        // Set optional features
        if var.use_calculator {
            action.set_calculator_event_name(var.event_name);
        }

        if let Some(off_event) = var.off_event_name.as_ref() {
            let off_event_id = self.events.get_or_map_event_id(off_event, false);
            action.set_off_event(off_event_id);
        }

        if let Some(event_param) = var.event_param {
            action.set_param(event_param);
        }

        if let Some(on_condition_value) = var.on_condition_value {
            action.set_on_condition_value(on_condition_value);
        }

        action.set_switch_on(var.switch_on);

        let mapping = match var_type {
            VarType::AircraftVar => ActionType::Bool(Box::new(action)),
            VarType::LocalVar => ActionType::F64(Box::new(action)),
        };

        self.add_mapping(
            var_string,
            Mapping {
                action: mapping,
                condition: var.condition,
                cancel_h_events: var.cancel_h_events,
            },
        )?;

        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn add_num_set_generic<T>(
        &mut self,
        data_type: InDataTypes,
        category: &str,
        var: NumSetGenericEntry<T>,
    ) -> Result<(Option<Box<NumSet<T>>>, String), Error>
    where
        T: Default,
    {
        let event_id = self.events.get_or_map_event_id(&var.event_name, false);

        let (var_string, _) =
            self.add_var_string(category, &var.var_name, var.var_units.as_deref(), data_type)?;

        if let Some(interpolate_type) = var.interpolate {
            self.lvarstransfer.transfer.add_interpolate_mapping(
                &format!("K:{}", &var.event_name),
                var_string.clone(),
                var.var_units.as_deref(),
                interpolate_type,
            );
            self.interpolate_vars.insert(var_string.clone());
            self.add_mapping(
                var_string.clone(),
                Mapping {
                    action: ActionType::VarOnly,
                    condition: var.condition,
                    ..Default::default()
                },
            )?;
        } else {
            let mut action = Box::new(NumSet::new(event_id));

            if var.unreliable {
                self.unreliable_vars.insert(var_string.clone());
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

            action.set_is_user_event(var.is_user_event);

            return Ok((Some(action), var_string));
        }

        Ok((None, var_string))
    }

    fn add_num_set(&mut self, category: &str, var: Value) -> Result<(), Error> {
        let data_type_string: &str = check_and_return_field!("var_type", var, str);
        let data_type = get_data_type_from_string(data_type_string)?;

        let condition = try_cast_yaml!(var["condition"].clone());
        let cancel_h_events = var["cancel_h_events"].as_bool().unwrap_or(false);

        match data_type {
            InDataTypes::I32 => {
                let (mapping, var_string) =
                    self.add_num_set_generic::<i32>(data_type, category, try_cast_yaml!(var))?;
                if let Some(mapping) = mapping {
                    self.add_mapping(
                        var_string,
                        Mapping {
                            action: ActionType::I32(mapping),
                            condition,
                            cancel_h_events,
                        },
                    )?
                }
            }
            InDataTypes::F64 => {
                let (mapping, var_string) =
                    self.add_num_set_generic::<f64>(data_type, category, try_cast_yaml!(var))?;
                if let Some(mapping) = mapping {
                    self.add_mapping(
                        var_string,
                        Mapping {
                            action: ActionType::F64(mapping),
                            condition,
                            cancel_h_events,
                        },
                    )?
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn add_num_increment_generic<T: ToString>(
        &mut self,
        data_type: InDataTypes,
        category: &str,
        var: NumIncrementEntry<T>,
    ) -> Result<(Box<NumIncrement<T>>, String), Error>
    where
        T: Default,
    {
        let (var_string, _) =
            self.add_var_string(category, &var.var_name, var.var_units.as_deref(), data_type)?;

        let mut mapping = NumIncrement::new(var.is_user_event, var.increment_by);
        mapping.set_pass_difference(var.pass_difference);

        if let Some(up_event_param) = var.up_event_param {
            mapping.set_up_event_param(up_event_param);
        }

        if let Some(down_event_param) = var.down_event_param {
            mapping.set_down_event_param(down_event_param);
        }

        if var.use_calculator {
            mapping.set_up_event_name(var.up_event_name);
            mapping.set_down_event_name(var.down_event_name);
        } else {
            let up_event_id = self.events.get_or_map_event_id(&var.up_event_name, false);
            let down_event_id = self.events.get_or_map_event_id(&var.down_event_name, false);

            mapping.set_up_event_id(up_event_id);
            mapping.set_down_event_id(down_event_id);
        }

        Ok((Box::new(mapping), var_string))
    }

    fn add_num_increment(&mut self, category: &str, var: Value) -> Result<(), Error> {
        let data_type_string: &str = check_and_return_field!("var_type", var, str);
        let data_type = get_data_type_from_string(data_type_string)?;

        let condition = try_cast_yaml!(var["conditions"].clone());
        let cancel_h_events = var["cancel_h_events"].as_bool().unwrap_or(false);

        match data_type {
            InDataTypes::I32 => {
                let (mapping, var_string) = self.add_num_increment_generic::<i32>(
                    data_type,
                    category,
                    try_cast_yaml!(var),
                )?;
                self.add_mapping(
                    var_string,
                    Mapping {
                        action: ActionType::I32(mapping),
                        condition,
                        cancel_h_events,
                    },
                )?
            }
            InDataTypes::F64 => {
                let (mapping, var_string) = self.add_num_increment_generic::<f64>(
                    data_type,
                    category,
                    try_cast_yaml!(var),
                )?;
                self.add_mapping(
                    var_string,
                    Mapping {
                        action: ActionType::F64(mapping),
                        condition,
                        cancel_h_events,
                    },
                )?
            }
            _ => {}
        };

        Ok(())
    }

    fn add_num_digit_set(&mut self, category: &str, var: NumDigitSetEntry) -> Result<(), Error> {
        let mut up_event_ids = Vec::new();
        let mut down_event_ids = Vec::new();

        for up_event_name in var.up_event_names.iter() {
            up_event_ids.push(self.events.get_or_map_event_id(up_event_name, false));
        }

        for down_event_name in var.down_event_names.iter() {
            down_event_ids.push(self.events.get_or_map_event_id(down_event_name, false));
        }

        let (var_string, _) = self.add_var_string(
            category,
            &var.var_name,
            var.var_units.as_deref(),
            InDataTypes::I32,
        )?;
        self.add_mapping(
            var_string,
            Mapping {
                action: ActionType::I32(Box::new(NumDigitSet::new(up_event_ids, down_event_ids))),
                condition: var.condition,
                cancel_h_events: var.cancel_h_events,
            },
        )?;

        Ok(())
    }

    fn add_custom_calculator(
        &mut self,
        category: &str,
        var: CustomCalculatorEntry,
    ) -> Result<(), Error> {
        let category = get_category_from_string(category)?;

        let var_name = self.lvarstransfer.add_custom_var(var.get);

        self.categories.insert(var_name.clone(), category);
        self.add_mapping(
            var_name,
            Mapping {
                action: ActionType::F64(Box::new(CustomCalculator::new(var.set))),
                condition: var.condition,
                cancel_h_events: var.cancel_h_events,
            },
        )?;

        Ok(())
    }

    fn add_local_var_proxy(
        &mut self,
        category: &str,
        var: LocalVarProxyEntry,
    ) -> Result<(), Error> {
        let (var_string, _) =
            self.add_var_string(category, &var.var_name, None, InDataTypes::F64)?;

        let loopback = if var.loopback {
            Some(var_string.clone())
        } else {
            None
        };

        self.add_mapping(
            var_string,
            Mapping {
                action: ActionType::F64(Box::new(LocalVarProxy::new(var.target, loopback))),
                condition: var.condition,
                ..Default::default()
            },
        )
    }

    fn add_multiply_difference_local_var(
        &mut self,
        category: &str,
        var: MultiplyDifferenceLocalVarEntry,
    ) -> Result<(), Error> {
        let (var_string, _) =
            self.add_var_string(category, &var.var_name, None, InDataTypes::F64)?;

        let loopback = if var.loopback {
            Some(var_string.clone())
        } else {
            None
        };

        self.add_mapping(
            var_string,
            Mapping {
                action: ActionType::F64(Box::new(MultiplyDifferenceLocalVarSet::new(
                    var.target,
                    var.multiply_by,
                    var.max_val,
                    loopback,
                ))),
                condition: var.condition,
                ..Default::default()
            },
        )
    }

    fn add_reset_when_equals(
        &mut self,
        category: &str,
        var: ResetWhenEqualsEntry,
    ) -> Result<(), Error> {
        let (var_string, _) =
            self.add_var_string(category, &var.var_name, None, InDataTypes::F64)?;

        self.add_mapping(
            var_string,
            Mapping {
                action: ActionType::F64(Box::new(ResetWhenEquals::new(var.target, var.equals))),
                condition: var.condition,
                ..Default::default()
            },
        )
    }

    fn add_program_action(&mut self, category: &str, var: ProgramActionEntry) -> Result<(), Error> {
        let (var_string, _) = self.add_var_string(
            category,
            &var.var_name,
            var.var_units.as_deref(),
            var.var_type,
        )?;
        self.add_mapping(
            var_string,
            Mapping {
                action: ActionType::ProgramAction(var.action),
                condition: var.condition,
                ..Default::default()
            },
        )?;

        Ok(())
    }

    fn add_program_action_event(
        &mut self,
        category: &str,
        event: ProgramActionEventEntry,
    ) -> Result<(), Error> {
        let category = get_category_from_string(category)?;

        self.events.get_or_map_event_id(&event.event_name, true);
        self.categories.insert(event.event_name.clone(), category);

        self.add_mapping(
            event.event_name,
            Mapping {
                action: ActionType::ProgramAction(event.action),
                ..Default::default()
            },
        )?;

        Ok(())
    }

    fn add_to_buffer(&mut self, category: String, value: Value) {
        match self.definitions_buffer.entry(category) {
            indexmap::map::Entry::Occupied(mut o) => o.get_mut().push(value),
            indexmap::map::Entry::Vacant(v) => {
                v.insert(vec![value]);
            }
        };
    }

    pub fn get_buffer_bytes(&mut self) -> Vec<u8> {
        rmp_serde::to_vec(&self.definitions_buffer).unwrap()
    }

    // Calls the correct method for the specified "action" type
    fn parse_var(&mut self, category: String, value: Value) -> Result<(), Error> {
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
            "LOCALVARPROXY" => self.add_local_var_proxy(&category, try_cast_yaml!(value))?,
            "RESETWHENEQUALS" => self.add_reset_when_equals(&category, try_cast_yaml!(value))?,
            "MULTIPLYDIFFERENCELOCALVAR" => {
                self.add_multiply_difference_local_var(&category, try_cast_yaml!(value))?
            }
            "PROGRAMACTIONEVENT" => {
                self.add_program_action_event(&category, try_cast_yaml!(value))?
            }
            _ => return Err(Error::InvalidSyncType(type_str.to_string())),
        };

        self.add_to_buffer(category, value_clone);

        Ok(())
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
    fn parse_yaml(&mut self, yaml: IndexMap<String, Vec<Value>>) -> Result<(), Error> {
        for (key, value) in yaml {
            if key == "include" {
                for include_file in value {
                    let file_name = include_file.as_str().unwrap();

                    match self.load_config(file_name) {
                        Ok(_) => (),
                        Err(e) => {
                            if let Error::YamlError(e, _) = e {
                                return Err(Error::IncludeError(
                                    e.to_string(),
                                    file_name.to_string(),
                                ));
                            };
                        }
                    }
                }
            } else if key == "ignore" {
                for ignore_value in value {
                    self.do_not_sync
                        .insert(ignore_value.as_str().unwrap().to_string());
                    self.add_to_buffer(key.clone(), ignore_value);
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
    pub fn load_config(&mut self, path: impl AsRef<Path> + Display) -> Result<(), Error> {
        let path_string = path.to_string();

        let file = File::open(path).map_err(Error::IOError)?;

        let yaml: IndexMap<String, Vec<Value>> =
            serde_yaml::from_reader(file).map_err(|e| Error::YamlError(e, path_string.clone()))?;

        self.parse_yaml(yaml)
    }

    pub fn load_config_from_bytes(&mut self, bytes: Box<[u8]>) -> Result<(), Error> {
        let yaml: IndexMap<String, Vec<Value>> = rmp_serde::from_slice(&bytes)?;

        self.parse_yaml(yaml)
    }

    fn process_local_var(&mut self, result: GetResult) {
        let mut should_write = !check_did_write_recently(&mut self.last_written, &result.var_name)
            && !self.do_not_sync.contains(&result.var_name);

        if let Some(mappings) = self.mappings.get_mut(&result.var_name) {
            for mapping in mappings {
                if mapping.cancel_h_events {
                    self.event_cancel_timer = Instant::now();
                }

                execute_mapping!(
                    new_value,
                    action,
                    VarReaderTypes::F64(result.value),
                    mapping,
                    { action.set_current(new_value) },
                    {},
                    {
                        self.pending_action = Some(action.clone());
                        should_write = false;
                    }
                );
            }
        }

        if !should_write {
            return;
        }

        self.current_sync
            .lvars
            .insert(result.var_name, VarReaderTypes::F64(result.value));
    }

    // Processes client data and adds to the result queue if it changed
    pub fn process_client_data(&mut self, data: &simconnect::SIMCONNECT_RECV_CLIENT_DATA) {
        for value in self.lvarstransfer.process_client_data(data) {
            self.process_local_var(value);
        }
    }

    fn process_js_data(&mut self) {
        if let Some(message) = self.jstransfer.poll() {
            match message.payload {
                JSPayloads::Interaction { name } => {
                    if self.do_not_sync.contains(&name[5..])
                        || self.event_cancel_timer.elapsed().as_millis() < 300
                    {
                        return;
                    };
                    self.current_sync.events.push(Event::JSEvent { name });
                }
                JSPayloads::Input { id, value } => {
                    let mut input_full_name = String::new();
                    input_full_name.push_str(&message.instrument_name);
                    input_full_name.push('#');
                    input_full_name.push_str(&id);

                    if self.do_not_sync.contains(&input_full_name) {
                        return;
                    }

                    self.current_sync.events.push(Event::JSInput {
                        instrument: message.instrument_name,
                        value,
                        id,
                    })
                }
                JSPayloads::Time {
                    hour,
                    minute,
                    day,
                    year,
                } => self.current_sync.events.push(Event::Time {
                    hour,
                    minute,
                    day,
                    year,
                }),
                _ => {}
            }
        };
    }

    // Processes event data name and the additional dword data
    pub fn process_event_data(&mut self, data: &simconnect::SIMCONNECT_RECV_EVENT) {
        // Not for us
        if data.uGroupID != self.events.group_id {
            return;
        }

        // Regular KEY event
        let event_name = match self.events.match_event_id(data.uEventID) {
            Some(event_name) => event_name.clone(),
            None => return,
        };

        let mut should_write = true;

        if let Some(mappings) = self.mappings.get(&event_name) {
            for mapping in mappings {
                if !evaluate_conditions(
                    &self.lvarstransfer,
                    &self.avarstransfer,
                    mapping.condition.as_ref(),
                    &VarReaderTypes::Bool(false),
                    None,
                ) {
                    should_write = false;
                    continue;
                }

                if let ActionType::ProgramAction(action) = &mapping.action {
                    self.pending_action = Some(action.clone());
                    should_write = false;
                    continue;
                }

                if mapping.cancel_h_events {
                    self.event_cancel_timer = Instant::now();
                }

                // Check timer
                if check_did_write_recently(&mut self.last_written, &event_name) {
                    return;
                }
            }
        }

        if should_write {
            self.current_sync.events.push(Event::KeyEvent {
                name: event_name,
                value: data.dwData,
            });
        }
    }

    // Process changed aircraft variables and update SyncActions related to it
    #[allow(unused_variables)]
    pub fn process_sim_object_data(&mut self, data: &simconnect::SIMCONNECT_RECV_SIMOBJECT_DATA) {
        // self.physics_corrector.process_sim_object_data(data);
        if self.avarstransfer.define_id != data.dwDefineID {
            return;
        }
        // Data might be bad/config files don't line up
        if let Ok(data) = self.avarstransfer.read_vars(data) {
            // Remove some computed components
            // self.physics_corrector.remove_components(&mut data);
            // Update all syncactions with the changed values
            for (var_name, value) in &data {
                // Determine if this variable should be updated
                let mut should_write = !check_did_write_recently(&mut self.last_written, var_name)
                    && !self.do_not_sync.contains(var_name);
                // Set current var syncactions
                if let Some(mappings) = self.mappings.get_mut(var_name) {
                    for mapping in mappings {
                        if mapping.cancel_h_events {
                            self.event_cancel_timer = Instant::now();
                        }

                        execute_mapping!(
                            new_value,
                            action,
                            value,
                            mapping,
                            { action.set_current(*new_value) },
                            {},
                            {}
                        );
                    }
                }

                if let Some(period) = self.periods.get_mut(var_name) {
                    should_write = should_write && period.do_update();
                }

                if should_write {
                    // Queue data for reading
                    self.current_sync.avars.insert(var_name.clone(), *value);
                }
            }
        }
    }

    fn process_js_interaction(&mut self, conn: &SimConnector, name: String) {
        if self.event_timer.elapsed().as_millis() < 50 {
            self.event_queue.push_front(Event::JSEvent { name });
            return;
        }

        // Use gauge to transmit H: event
        self.lvarstransfer.set_unchecked(conn, &name, None, "");

        self.event_timer = Instant::now();
    }

    fn process_key_event(
        &mut self,
        conn: &simconnect::SimConnector,
        name: String,
        value: u32,
    ) -> Result<(), Error> {
        for mapping in self
            .mappings
            .get(&name)
            .ok_or_else(|| Error::MissingMapping(name.clone()))?
        {
            if let ActionType::Event(mapping) = &mapping.action {
                if mapping.use_calculator {
                    self.lvarstransfer.set_unchecked(
                        conn,
                        &format!("K:{}", name),
                        None,
                        value.to_string().as_str(),
                    );
                } else {
                    self.events.trigger_event(conn, &name, value).unwrap();
                }
            }
        }

        Ok(())
    }

    fn process_events(&mut self, conn: &SimConnector) -> Result<(), Error> {
        if let Some(event) = self.event_queue.pop_front() {
            match event {
                Event::JSEvent { name } => self.process_js_interaction(conn, name),

                Event::JSInput {
                    id,
                    value,
                    instrument,
                } => self
                    .jstransfer
                    .write_payload(JSPayloads::Input { id, value }, Some(&instrument)),
                Event::Time {
                    hour,
                    minute,
                    day,
                    year,
                } => self.jstransfer.write_payload(
                    JSPayloads::Time {
                        hour,
                        minute,
                        day,
                        year,
                    },
                    None,
                ),

                Event::KeyEvent { name, value } => self.process_key_event(conn, name, value)?,
            }
        }

        Ok(())
    }

    pub fn request_time(&mut self) {
        self.jstransfer.write_payload(JSPayloads::RequestTime, None);
    }

    pub fn step(&mut self, conn: &SimConnector) -> Result<(), Error> {
        self.process_js_data();
        self.process_events(conn)
    }

    fn split_unreliable(&self, data: &mut AllNeedSync) -> AllNeedSync {
        data.filter_keep(|name| {
            self.interpolate_vars.contains(name) || self.unreliable_vars.contains(name)
        })
    }

    fn filter_all_sync(
        &self,
        mut data: AllNeedSync,
        sync_permission: &SyncPermission,
    ) -> (Option<AllNeedSync>, Option<AllNeedSync>) {
        // Filter out based on what the client's current permissions are
        data.filter(|name| self.can_sync(name, sync_permission));
        // Split into interpolated vs non interpolated values - used for reliable/unreliable transmissions
        let regular = self.split_unreliable(&mut data);
        // Convert into options
        let unreliable = if data.is_empty() { None } else { Some(data) };
        let regular = if regular.is_empty() {
            None
        } else {
            Some(regular)
        };

        (unreliable, regular)
    }

    pub fn get_sync(
        &mut self,
        sync_permission: &SyncPermission,
    ) -> (Option<AllNeedSync>, Option<AllNeedSync>) {
        let mut data = AllNeedSync::new();
        std::mem::swap(&mut self.current_sync, &mut data);
        self.filter_all_sync(data, sync_permission)
    }

    fn can_sync(&self, var_name: &str, sync_permission: &SyncPermission) -> bool {
        // Check categories
        match self.categories.get(var_name) {
            Some(Category::Shared) => true,
            Some(Category::Master) => sync_permission.is_master,
            Some(Category::Server) => sync_permission.is_server,
            Some(Category::Init) => sync_permission.is_init,
            _ => true,
        }
    }

    #[allow(unused_variables)]
    fn write_aircraft_data(&mut self, conn: &SimConnector, data: VarMap, time: f64) {
        if data.is_empty() {
            return;
        }

        let mut to_sync = VarMap::new();
        to_sync.reserve(data.len());

        let mut interpolation_data = Vec::new();

        let data = data;
        // Add some local computed components
        // self.physics_corrector.add_components(&mut data);

        // Only sync vars that are defined as so
        for (var_name, data) in data {
            set_did_write_recently(&mut self.last_written, &var_name);

            // Otherwise sync them using defined events
            if let Some(mappings) = self.mappings.get_mut(&var_name) {
                for mapping in mappings {
                    if !evaluate_conditions(
                        &self.lvarstransfer,
                        &self.avarstransfer,
                        mapping.condition.as_ref(),
                        &data,
                        None,
                    ) {
                        continue;
                    }

                    execute_mapping!(
                        new_value,
                        action,
                        data,
                        mapping,
                        { action.set_new(new_value, conn, &mut self.lvarstransfer) },
                        {
                            if self.interpolate_vars.contains(&var_name) {
                                // Queue data for interpolation
                                interpolation_data.push(InterpolateData {
                                    name: var_name.clone(),
                                    value: data.get_as_f64(),
                                    time,
                                });
                            } else {
                                // Set data right away
                                to_sync.insert(var_name.clone(), data);
                            }
                        },
                        {}
                    );
                }
            }
        }

        if !interpolation_data.is_empty() {
            self.lvarstransfer.transfer.send_new_interpolation_data(
                conn,
                time,
                &interpolation_data,
            );
        }

        if !to_sync.is_empty() {
            self.avarstransfer.set_vars(conn, &to_sync);
        }
    }

    #[allow(unused_variables)]
    fn write_local_data(&mut self, conn: &SimConnector, data: VarMap) -> Result<(), Error> {
        for (var_name, value) in &data {
            match self.mappings.get_mut(var_name) {
                Some(mappings) => {
                    for mapping in mappings {
                        if !evaluate_conditions(
                            &self.lvarstransfer,
                            &self.avarstransfer,
                            mapping.condition.as_ref(),
                            value,
                            Some(&data),
                        ) {
                            continue;
                        }

                        execute_mapping!(
                            new_value,
                            action,
                            value,
                            mapping,
                            { action.set_new(*new_value, conn, &mut self.lvarstransfer) },
                            {
                                self.lvarstransfer
                                    .set(conn, var_name, value.to_string().as_ref());
                            },
                            {}
                        );

                        set_did_write_recently(&mut self.last_written, var_name);
                    }
                }
                None => return Err(Error::MissingMapping(var_name.clone())),
            }
        }

        Ok(())
    }

    fn write_event_data(&mut self, data: EventData) -> Result<(), Error> {
        for event in data {
            self.event_queue.push_back(event);
        }

        Ok(())
    }

    pub fn on_receive_data(
        &mut self,
        conn: &SimConnector,
        mut data: AllNeedSync,
        time: f64,
        sync_permission: &SyncPermission,
    ) -> Result<(), Error> {
        data.filter(|name| self.can_sync(name, sync_permission));

        // In this specific order
        // Aircraft var data should overwrite any event data
        self.write_event_data(data.events)?;
        self.write_aircraft_data(conn, data.avars, time);
        self.write_local_data(conn, data.lvars)?;

        Ok(())
    }

    // To be called when SimConnect connects
    pub fn on_connected(&mut self, conn: &SimConnector) -> Result<(), ()> {
        self.avarstransfer.on_connected(conn);
        self.events.on_connected(conn);
        self.lvarstransfer.on_connected(conn);
        self.physics_corrector.on_connected(conn);

        // Might be running another instance
        #[cfg(not(feature = "skip_sim_connect"))]
        self.jstransfer.start().map_err(|_| ())?;

        // Get aircraft data
        conn.request_data_on_sim_object(
            0,
            self.avarstransfer.define_id,
            0,
            simconnect::SIMCONNECT_PERIOD_SIMCONNECT_PERIOD_SIM_FRAME,
            simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_CHANGED
                | simconnect::SIMCONNECT_CLIENT_DATA_REQUEST_FLAG_TAGGED,
            0,
            0,
            0,
        );

        Ok(())
    }

    pub fn get_all_current(&self) -> AllNeedSync {
        let avars = self
            .avarstransfer
            .get_all_vars()
            .clone()
            .into_iter()
            .filter(|(x, _)| !self.do_not_sync.contains(x))
            .collect();

        // self.physics_corrector.remove_components(&mut avars);

        AllNeedSync {
            avars,
            lvars: self
                .lvarstransfer
                .get_all_vars()
                .into_iter()
                .filter(|(x, _)| !self.do_not_sync.contains(x))
                .map(|(k, v)| (k, VarReaderTypes::F64(v)))
                .collect(),
            events: EventData::new(),
        }
    }

    pub fn reset_sync(&mut self) {
        self.current_sync.clear();
        self.last_written.clear();
    }

    pub fn get_number_avars(&self) -> usize {
        self.avarstransfer.get_number_defined()
    }

    pub fn get_number_events(&self) -> usize {
        self.events.get_number_defined()
    }

    pub fn get_number_lvars(&self) -> usize {
        self.lvarstransfer.get_number_defined()
    }

    pub fn get_next_pending_action(&mut self) -> Option<ProgramAction> {
        self.pending_action.as_ref()?;

        let mut next_action = None;
        swap(&mut self.pending_action, &mut next_action);

        next_action
    }
}
