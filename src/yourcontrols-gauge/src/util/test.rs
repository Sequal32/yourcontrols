use std::{cell::RefCell, rc::Rc};

use crate::data::{Settable, Syncable, Variable};

use super::DatumValue;

pub struct EventCallCounter {
    pub called_count: u32,
    pub last_set_value: DatumValue,
}

impl EventCallCounter {
    pub fn new() -> Self {
        Self {
            called_count: 0,
            last_set_value: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.called_count = 0;
        self.last_set_value = 0.0;
    }
}

impl Settable for EventCallCounter {
    fn set(&mut self) {
        self.called_count += 1;
    }

    fn set_with_value(&mut self, value: DatumValue) {
        self.called_count += 1;
        self.last_set_value = value;
    }
}

pub struct TestVariable {
    value: DatumValue,
}

impl TestVariable {
    pub fn new(value: DatumValue) -> Self {
        Self { value }
    }

    pub fn set_new_value(&mut self, value: DatumValue) {
        self.value = value;
    }
}

impl Variable for TestVariable {
    fn get(&self) -> DatumValue {
        self.value
    }

    fn set(&mut self, _value: DatumValue) {}
}

pub fn get_test_variable(value: DatumValue) -> Rc<RefCell<TestVariable>> {
    Rc::new(RefCell::new(TestVariable::new(value)))
}

pub fn get_call_counter() -> Rc<RefCell<EventCallCounter>> {
    Rc::new(RefCell::new(EventCallCounter::new()))
}

pub fn process_then_set(var: &RefCell<TestVariable>, event: &mut dyn Syncable, value: DatumValue) {
    event.process_incoming(value);
    var.borrow_mut().set_new_value(value);
}
