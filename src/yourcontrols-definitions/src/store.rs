use std::sync::{Mutex, MutexGuard};

use lazy_static::lazy_static;
use owning_ref::{MutexGuardRef, OwningRef};
use yourcontrols_types::{EventMessage, VarType};

lazy_static! {
    pub static ref DATABASE: FactorDatabase = FactorDatabase::new();
}

pub type VarsRef<'a> = OwningRef<MutexGuard<'a, Factor<VarType>>, Vec<VarType>>;
pub type EventsRef<'a> = OwningRef<MutexGuard<'a, Factor<EventMessage>>, Vec<EventMessage>>;

pub struct Factor<T> {
    vec: Vec<T>,
}

impl<T> Factor<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn add(&mut self, var: T) -> usize {
        self.vec.push(var);
        self.vec.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }
}

pub struct FactorDatabase {
    pub vars: Mutex<Factor<VarType>>,
    pub events: Mutex<Factor<EventMessage>>,
}

impl FactorDatabase {
    pub fn new() -> Self {
        Self {
            vars: Mutex::new(Factor::new()),
            events: Mutex::new(Factor::new()),
        }
    }

    pub fn add_var(&self, var: VarType) -> usize {
        self.vars.lock().unwrap().add(var)
    }

    pub fn get_var(&self, index: usize) -> Option<VarType> {
        self.vars.lock().unwrap().get(index).cloned()
    }

    pub fn get_all_vars(&self) -> VarsRef {
        MutexGuardRef::new(self.vars.lock().unwrap()).map(|mg| &mg.vec)
    }

    pub fn add_event(&self, event: EventMessage) -> usize {
        self.events.lock().unwrap().add(event)
    }

    pub fn get_event(&self, index: usize) -> Option<EventMessage> {
        self.events.lock().unwrap().get(index).cloned()
    }

    pub fn get_all_events(&self) -> EventsRef {
        MutexGuardRef::new(self.events.lock().unwrap()).map(|mg| &mg.vec)
    }
}

pub fn map_vec_to_database<T, F>(vec: Vec<T>, f: F) -> Vec<usize>
where
    F: Fn(T) -> usize,
{
    vec.into_iter().map(|x| f(x)).collect()
}
