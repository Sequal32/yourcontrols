use crate::data::{Settable, Syncable, Variable};

use super::util::NumberDigits;

pub struct ToggleSwitch {
    var: Box<dyn Variable>,
    // Either the toggle event or the off event
    event: Box<dyn Settable>,
    // To be used in the event where two events control this switch
    off_event: Option<Box<dyn Settable>>,
    // Only trigger if the new value is on
    switch_on: bool,
}

impl Syncable for ToggleSwitch {
    fn process_incoming(&self, value: f64) {
        let switch_should_be_on = value == 0.0;
        let switch_currently_on = self.var.get_bool();

        if switch_should_be_on && switch_currently_on {
            return;
        }
        if !switch_should_be_on && self.switch_on {
            return;
        }

        if switch_should_be_on {
            self.event.set();
        } else {
            if let Some(off_event) = self.off_event.as_ref() {
                off_event.set()
            } else {
                self.event.set()
            }
        }
    }

    fn get_var_value(&self) -> f64 {
        self.var.get()
    }
}

pub struct NumSet {
    var: Box<dyn Variable>,
    event: Box<dyn Settable>,
    swap_event: Option<Box<dyn Settable>>,
    multiply_by: f64,
    add_by: f64,
}

impl Syncable for NumSet {
    fn process_incoming(&self, value: f64) {
        let value = value * self.multiply_by + self.add_by;

        self.event.set_with_value(value);

        if let Some(swap_event) = self.swap_event.as_ref() {
            swap_event.set();
        }
    }
    fn get_var_value(&self) -> f64 {
        self.var.get()
    }
}

pub struct NumIncrement {
    var: Box<dyn Variable>,
    up_event: Box<dyn Settable>,
    down_event: Box<dyn Settable>,
    increment_amount: f64,
    pass_difference: bool,
}

impl Syncable for NumIncrement {
    fn process_incoming(&self, value: f64) {
        let current = self.var.get();

        if self.pass_difference {
            if value > current {
                self.up_event.set_with_value(value - current);
            } else {
                self.down_event.set_with_value(current - value);
            }
        } else {
            let mut working = current;

            while working > value {
                working -= self.increment_amount;
                self.down_event.set();
            }

            while working < value {
                working += self.increment_amount;
                self.up_event.set();
            }
        }
    }

    fn get_var_value(&self) -> f64 {
        self.var.get()
    }
}

pub struct NumDigitSet {
    var: Box<dyn Variable>,
    inc_events: Vec<Box<dyn Settable>>,
    dec_events: Vec<Box<dyn Settable>>,
}

impl Syncable for NumDigitSet {
    fn process_incoming(&self, value: f64) {
        let current = NumberDigits::new(self.var.get() as i32);
        let value = NumberDigits::new(value as i32);

        for index in 0..self.inc_events.len() {
            let mut working_value = current.get(index);
            let new_digit = value.get(index);

            let down_event = match self.inc_events.get(index) {
                Some(e) => e,
                None => return,
            };

            let up_event = match self.dec_events.get(index) {
                Some(e) => e,
                None => return,
            };

            while working_value > new_digit {
                working_value -= 1;
                down_event.set();
            }

            while working_value < new_digit {
                working_value += 1;
                up_event.set();
            }
        }
    }

    fn get_var_value(&self) -> f64 {
        self.var.get()
    }
}
