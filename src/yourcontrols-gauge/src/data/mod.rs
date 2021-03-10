use std::{cell::RefCell, rc::Rc};

use crate::util::Error;
use crate::util::GenericResult;

pub mod datum;
pub mod diff;
mod util;
pub mod watcher;

#[cfg(any(target_arch = "wasm32", doc))]
use msfs::legacy::{
    execute_calculator_code, AircraftVariable, CompiledCalculatorCode, NamedVariable,
};

/// A wrapper struct for NamedVariable, AircraftVariable, and calculator codes.
///
/// `get()` for `Variable` will return the first non-none variable.
/// `set()` for `Variable` will either directly set a NamedVariable, or use execute_calculator_code to set AircraftVariable or a calculator code.
#[derive(Default)]
#[cfg(any(target_arch = "wasm32", doc))]
pub struct GenericVariable {
    named: Option<NamedVariable>,
    var: Option<AircraftVariable>,
    // Without parentheses & greater than
    calculator_partial: Option<String>,
    compiled_get: Option<CompiledCalculatorCode>,
}

#[cfg(any(target_arch = "wasm32", doc))]
impl GenericVariable {
    pub fn new_var(name: &str, units: &str, index: Option<usize>) -> GenericResult<Self> {
        let index = index.unwrap_or(0);

        Ok(Self {
            var: Some(AircraftVariable::from(name, units, index)?),
            calculator_partial: Some(format!("{}:{}, {}", name, index, units)),
            ..Default::default()
        })
    }

    pub fn new_named(name: &str) -> GenericResult<Self> {
        Ok(Self {
            named: Some(NamedVariable::from(name)),
            ..Default::default()
        })
    }

    pub fn new_calculator(left: String, right: Option<String>) -> GenericResult<Self> {
        let right = right.unwrap_or(String::new());

        Ok(Self {
            compiled_get: Some(
                CompiledCalculatorCode::new(&format!("({}, {})", left, right))
                    .ok_or(Error::VariableInitializeError)?,
            ),
            calculator_partial: Some(format!("{}, {}", left, right)),
            ..Default::default()
        })
    }
}

#[cfg(any(target_arch = "wasm32", doc))]
impl Variable for GenericVariable {
    fn get(&self) -> f64 {
        if let Some(named) = self.named.as_ref() {
            return named.get_value();
        }

        if let Some(var) = self.var.as_ref() {
            return var.get();
        }

        if let Some(calculator) = self.compiled_get.as_ref() {
            return calculator.execute().unwrap_or(0.0);
        }

        0.0
    }

    fn set(&mut self, value: f64) {
        if let Some(named) = self.named.as_ref() {
            named.set_value(value);
        }

        // Handles aircraft variables too
        if let Some(calculator) = self.calculator_partial.as_ref() {
            execute_calculator_code::<f64>(&format!("{} (>{})", value, calculator));
        }
    }
}

#[cfg(any(target_arch = "wasm32", doc))]
impl Syncable for GenericVariable {
    fn process_incoming(&mut self, value: f64) {
        if self.get() == value {
            return;
        }
        self.set(value)
    }

    fn get_var_value(&self) -> f64 {
        self.get()
    }
}

/// Provides multiple `set` implementations for an `event_name` and an `event_index`.
#[cfg(any(target_arch = "wasm32", doc))]
pub struct EventSet {
    event_name: String,
    event_index: Option<u32>,
    index_reversed: bool,
}

#[cfg(any(target_arch = "wasm32", doc))]
impl EventSet {
    /// The event will be executed with a value and an index.
    ///
    /// Format:
    /// `{value} {index} (>{event_name})`
    ///
    /// or with index_reversed:
    /// `{index} {value} (>{event_name})`
    fn set_with_value_and_index(&self, value: f64, index: u32) {
        if self.index_reversed {
            execute_calculator_code::<f64>(&format!(
                "{} {} (>K:{})",
                value, index, self.event_name
            ));
        } else {
            execute_calculator_code::<f64>(&format!(
                "{} {} (>K:{})",
                index, value, self.event_name
            ));
        }
    }

    /// The event will be executed with a value.
    ///
    /// Format:
    /// `{value} (>{event_name})`
    fn set_with_value_only(&self, value: f64) {
        execute_calculator_code::<f64>(&format!("{} (>K:{})", value, self.event_name));
    }
}

#[cfg(any(target_arch = "wasm32", doc))]
impl Settable for EventSet {
    fn set(&mut self) {
        execute_calculator_code::<f64>(&format!("(>K:{})", self.event_name));
    }

    fn set_with_value(&mut self, value: f64) {
        if let Some(index) = self.event_index {
            self.set_with_value_and_index(value, index);
        } else {
            self.set_with_value_only(value);
        }
    }
}

pub type RcVariable = Rc<RefCell<dyn Variable>>;
pub type RcSettable = Rc<RefCell<dyn Settable>>;

pub trait Syncable {
    fn process_incoming(&mut self, value: f64);
    fn get_var_value(&self) -> f64;
}

pub trait Variable {
    fn get(&self) -> f64;
    fn get_bool(&self) -> bool {
        self.get() == 1.0
    }
    fn set(&mut self, value: f64);
}

pub trait Settable {
    fn set(&mut self);
    fn set_with_value(&mut self, value: f64);
}
