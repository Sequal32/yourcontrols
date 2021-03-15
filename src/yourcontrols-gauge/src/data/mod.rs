use std::{cell::RefCell, rc::Rc};

pub mod datum;
pub mod diff;
mod util;
pub mod watcher;
#[cfg(any(target_arch = "wasm32"))]
use msfs::legacy::{
    execute_calculator_code, AircraftVariable, CompiledCalculatorCode, NamedVariable,
};
use msfs::sim_connect::SimConnect;
use yourcontrols_types::{DatumValue, Error, Result};

/// A wrapper struct for NamedVariable, AircraftVariable, and calculator codes.
///
/// `get()` for `Variable` will return the first non-none variable.
/// `set()` for `Variable` will either directly set a NamedVariable, or use execute_calculator_code to set AircraftVariable or a calculator code.
#[cfg(any(target_arch = "wasm32"))]
#[derive(Default)]
pub struct GenericVariable {
    named: Option<NamedVariable>,
    var: Option<AircraftVariable>,
    // Without parentheses & greater than
    calculator_set: Option<String>,
    calculator_get: Option<CompiledCalculatorCode>,
}

#[cfg(any(target_arch = "wasm32"))]
impl GenericVariable {
    pub fn new_var(name: &str, units: &str, index: Option<usize>) -> Result<Self> {
        let index = index.unwrap_or(0);

        Ok(Self {
            var: Some(
                AircraftVariable::from(name, units, index)
                    .map_err(|_| Error::VariableInitializeError)?,
            ),
            calculator_set: Some(format!("(>{}:{}, {})", name, index, units)),
            ..Default::default()
        })
    }

    pub fn new_named(name: &str) -> Result<Self> {
        Ok(Self {
            named: Some(NamedVariable::from(name)),
            ..Default::default()
        })
    }

    pub fn new_calculator(get: String, set: String) -> Result<Self> {
        Ok(Self {
            calculator_get: Some(
                CompiledCalculatorCode::new(&get).ok_or(Error::VariableInitializeError)?,
            ),
            calculator_set: Some(set),
            ..Default::default()
        })
    }
}

#[cfg(any(target_arch = "wasm32"))]
impl Variable for GenericVariable {
    fn get(&self) -> DatumValue {
        if let Some(named) = self.named.as_ref() {
            return named.get_value();
        }

        if let Some(var) = self.var.as_ref() {
            return var.get();
        }

        if let Some(calculator) = self.calculator_get.as_ref() {
            return calculator.execute().unwrap_or(0.0);
        }

        0.0
    }

    fn set(&mut self, value: DatumValue) {
        if let Some(named) = self.named.as_ref() {
            named.set_value(value);
        }

        // Handles aircraft variables too
        if let Some(calculator) = self.calculator_set.as_ref() {
            execute_calculator_code::<()>(calculator);
        }
    }
}

#[cfg(any(target_arch = "wasm32"))]
impl Syncable for GenericVariable {
    fn process_incoming(&mut self, value: DatumValue) {
        if self.get() == value {
            return;
        }
        self.set(value)
    }
}

/// Provides multiple `set` implementations for an `event_name` and an `event_index`.
pub struct EventSet {
    event_name: String,
    event_index: Option<u32>,
    index_reversed: bool,
}

#[cfg(any(target_arch = "wasm32"))]
impl EventSet {
    /// The event will be executed with a value and an index.
    ///
    /// Format:
    /// `{value} {index} (>{event_name})`
    ///
    /// or with index_reversed:
    /// `{index} {value} (>{event_name})`
    fn set_with_value_and_index(&self, value: DatumValue, index: u32) {
        if self.index_reversed {
            execute_calculator_code::<DatumValue>(&format!(
                "{} {} (>K:2:{})",
                value, index, self.event_name
            ));
        } else {
            execute_calculator_code::<DatumValue>(&format!(
                "{} {} (>K:2:{})",
                index, value, self.event_name
            ));
        }
    }

    /// The event will be executed with a value.
    ///
    /// Format:
    /// `{value} (>{event_name})`
    fn set_with_value_only(&self, value: DatumValue) {
        execute_calculator_code::<DatumValue>(&format!("{} (>K:{})", value, self.event_name));
    }
}

#[cfg(any(target_arch = "wasm32"))]
impl Settable for EventSet {
    fn set(&mut self) {
        execute_calculator_code::<DatumValue>(&format!("(>K:{})", self.event_name));
    }

    fn set_with_value(&mut self, value: DatumValue) {
        if let Some(index) = self.event_index {
            self.set_with_value_and_index(value, index);
        } else {
            self.set_with_value_only(value);
        }
    }
}

/// Listens to an `event_name` and keeps track of how many times it was triggered.
pub struct KeyEvent {
    trigger_count: u32,
    event_name: String,
    id: u32,
}

impl KeyEvent {
    /// Constructs and starts listening for triggers of `event_name`.
    pub fn new(simconnect: &mut SimConnect, event_name: String) -> Self {
        Self {
            id: simconnect
                .map_client_event_to_sim_event(&event_name, false)
                .unwrap_or(0),
            trigger_count: 0,
            event_name,
        }
    }

    /// Increments `trigger_count`
    pub fn increment_count(&mut self) {
        self.trigger_count += 1;
    }

    pub fn reset_count(&mut self) {
        self.trigger_count = 0;
    }

    /// Getter for `trigger_counter`.
    pub fn trigger_count(&self) -> u32 {
        self.trigger_count
    }
}

#[cfg(any(target_arch = "wasm32"))]
impl Syncable for KeyEvent {
    fn process_incoming(&mut self, value: DatumValue) {
        execute_calculator_code::<DatumValue>(&format!("{} (>K:{})", value, self.event_name));
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Syncable for KeyEvent {
    fn process_incoming(&mut self, value: DatumValue) {}
}

pub type MultiMutable<T> = Rc<RefCell<T>>;
/// A clonable, reference counted variable.
pub type RcVariable = MultiMutable<dyn Variable>;
/// A clonable, reference counted settable.
pub type RcSettable = MultiMutable<dyn Settable>;
/// Used to execute a task upon receiving a value.
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Syncable {
    fn process_incoming(&mut self, value: DatumValue);
}
#[cfg_attr(test, automock)]
pub trait Variable {
    fn get(&self) -> DatumValue;
    fn get_bool(&self) -> bool {
        self.get() == 1.0
    }
    fn set(&mut self, value: DatumValue);
}
#[cfg_attr(test, automock)]
pub trait Settable {
    fn set(&mut self) {}
    fn set_with_value(&mut self, value: DatumValue);
}
