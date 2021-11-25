mod error;

pub use error::Error;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Event {
    JSEvent {
        name: String,
    },
    JSInput {
        id: String,
        value: String,
        instrument: String,
    },
    KeyEvent {
        name: String,
        value: u32,
    },
    Time {
        hour: u32,
        minute: u32,
        day: u32,
        year: u32,
    },
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd)]
pub enum VarReaderTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
}

impl VarReaderTypes {
    pub fn get_as_f64(&self) -> f64 {
        match self {
            VarReaderTypes::Bool(v) => *v as i32 as f64,
            VarReaderTypes::I32(v) => *v as f64,
            VarReaderTypes::I64(v) => *v as f64,
            VarReaderTypes::F64(v) => *v,
        }
    }
}

// Name of aircraft variable and the value of it
pub type AVarMap = HashMap<String, VarReaderTypes>;
// Name of local variable and the value of it
pub type LVarMap = HashMap<String, f64>;
// Name of the event the DWORD data associated with it with how many times it got triggered (not a map as the event could've got triggered multiple times before the data could get send)
pub type EventData = Vec<Event>;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AllNeedSync {
    pub avars: AVarMap,
    pub lvars: LVarMap,
    pub events: EventData,
}

impl AllNeedSync {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_empty(&self) -> bool {
        self.avars.len() == 0 && self.lvars.len() == 0 && self.events.len() == 0
    }

    pub fn clear(&mut self) {
        self.avars.clear();
        self.lvars.clear();
        self.events.clear();
    }

    // Filter the variables
    pub fn filter<F>(&mut self, filter_fn: F)
    where
        F: Fn(&str) -> bool,
    {
        self.filter_keep(filter_fn);
    }

    // Keeps variables that matches the filter, returns the variables that don't
    pub fn filter_keep<F>(&mut self, filter_fn: F) -> AllNeedSync
    where
        F: Fn(&str) -> bool,
    {
        let mut filtered = AllNeedSync::new();

        macro_rules! filter_or_op {
            ($name: expr, $op: block) => {
                if filter_fn($name) {
                    return true;
                } else {
                    $op
                    return false;
                }
            };
        }

        macro_rules! filter_or_push {
            ($name: expr, $map: ident, $value: expr) => {
                filter_or_op!($name, {
                    filtered.$map.push($value.clone());
                })
            };
        }

        macro_rules! filter_or_insert {
            ($name: expr, $map: ident, $value: expr) => {
                filter_or_op!($name, {
                    filtered.$map.insert($name.clone(), $value.clone());
                })
            };
        }

        self.avars
            .retain(|name, var| filter_or_insert!(name, avars, var));

        self.lvars
            .retain(|name, var| filter_or_insert!(name, lvars, var));

        self.events.retain(|event| match event {
            Event::JSEvent { name } => filter_or_push!(name, events, event),
            Event::JSInput { id, .. } => filter_or_push!(id, events, event),
            Event::KeyEvent { name, .. } => filter_or_push!(name, events, event),
            Event::Time { .. } => filter_or_push!("", events, event),
        });

        filtered
    }
}
