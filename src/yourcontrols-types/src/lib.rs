mod error;

pub use error::Error;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventTriggered {
    pub event_name: String,
    pub data: u32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd)]
pub enum VarReaderTypes {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64)
}

impl VarReaderTypes {
    pub fn get_as_f64(&self) -> f64 {
        match self {
            VarReaderTypes::Bool(v) => *v as i32 as f64,
            VarReaderTypes::I32(v) => *v as f64,
            VarReaderTypes::I64(v) => *v as f64,
            VarReaderTypes::F64(v) => *v
        }
    }
}

// Name of aircraft variable and the value of it
pub type AVarMap = HashMap<String, VarReaderTypes>;
// Name of local variable and the value of it
pub type LVarMap = HashMap<String, f64>;
// Name of the event the DWORD data associated with it with how many times it got triggered (not a map as the event could've got triggered multiple times before the data could get send)
pub type EventData = Vec<EventTriggered>;
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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