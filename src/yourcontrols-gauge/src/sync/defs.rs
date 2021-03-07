use std::{cell::RefCell, rc::Rc};

use super::util::NumberDigits;
use crate::data::{Settable, Syncable, Variable};

type MultiVariable = Rc<RefCell<dyn Variable>>;
type MultiSettable = Rc<RefCell<dyn Settable>>;

pub struct ToggleSwitch {
    var: MultiVariable,
    // Either the toggle event or the off event
    event: MultiSettable,
    // To be used in the event where two events control this switch
    off_event: Option<MultiSettable>,
    // Only trigger if the new value is on
    switch_on: bool,
}

impl Syncable for ToggleSwitch {
    fn process_incoming(&mut self, value: f64) {
        let switch_should_be_on = value == 1.0;
        let switch_currently_on = self.var.borrow_mut().get_bool();

        if switch_should_be_on == switch_currently_on {
            return;
        }
        if !switch_should_be_on && self.switch_on {
            return;
        }

        let mut event = self.event.borrow_mut();

        if switch_should_be_on {
            event.set();
        } else {
            if let Some(mut off_event) = self.off_event.as_ref().map(|x| x.borrow_mut()) {
                off_event.set()
            } else {
                event.set()
            }
        }
    }

    fn get_var_value(&self) -> f64 {
        self.var.borrow_mut().get()
    }
}

pub struct NumSet {
    var: MultiVariable,
    event: MultiSettable,
    swap_event: Option<MultiSettable>,
    multiply_by: f64,
    add_by: f64,
}

impl Syncable for NumSet {
    fn process_incoming(&mut self, value: f64) {
        let value = value * self.multiply_by + self.add_by;
        let mut event = self.event.borrow_mut();

        event.set_with_value(value);

        if let Some(swap_event) = self.swap_event.as_ref() {
            swap_event.borrow_mut().set();
        }
    }
    fn get_var_value(&self) -> f64 {
        self.var.borrow().get()
    }
}

pub struct NumIncrement {
    var: MultiVariable,
    up_event: MultiSettable,
    down_event: MultiSettable,
    increment_amount: f64,
    pass_difference: bool,
}

impl Syncable for NumIncrement {
    fn process_incoming(&mut self, value: f64) {
        let current = self.var.borrow().get();

        let mut up_event = self.up_event.borrow_mut();
        let mut down_event = self.down_event.borrow_mut();

        if self.pass_difference {
            if value > current {
                up_event.set_with_value(value - current);
            } else {
                down_event.set_with_value(current - value);
            }
        } else {
            let mut working = current;

            while working > value {
                working -= self.increment_amount;
                down_event.set();
            }

            while working < value {
                working += self.increment_amount;
                up_event.set();
            }
        }
    }

    fn get_var_value(&self) -> f64 {
        self.var.borrow().get()
    }
}

pub struct NumDigitSet {
    var: MultiVariable,
    inc_events: Vec<MultiSettable>,
    dec_events: Vec<MultiSettable>,
}

impl Syncable for NumDigitSet {
    fn process_incoming(&mut self, value: f64) {
        let current = NumberDigits::new(self.var.borrow().get() as i32);
        let value = NumberDigits::new(value as i32);

        for index in 0..self.inc_events.len() {
            let mut working_value = current.get(index);
            let new_digit = value.get(index);

            let mut down_event = match self.inc_events.get(index) {
                Some(e) => e.borrow_mut(),
                None => return,
            };

            let mut up_event = match self.dec_events.get(index) {
                Some(e) => e.borrow_mut(),
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
        self.var.borrow().get()
    }
}

#[cfg(test)]
mod tests {
    use super::ToggleSwitch;
    use crate::data::{Settable, Syncable, Variable};
    use std::{cell::RefCell, rc::Rc};

    struct EventCallCounter {
        called_count: u32,
        last_set_value: f64,
    }

    impl EventCallCounter {
        pub fn new() -> Self {
            Self {
                called_count: 0,
                last_set_value: 0.0,
            }
        }
    }

    impl Settable for EventCallCounter {
        fn set(&mut self) {
            self.called_count += 1;
        }

        fn set_with_value(&mut self, value: f64) {
            self.called_count += 1;
            self.last_set_value = 0.0;
        }
    }

    struct TestVariable {
        value: f64,
    }

    impl TestVariable {
        pub fn new(value: f64) -> Self {
            Self { value }
        }

        pub fn set_new_value(&mut self, value: f64) {
            self.value = value;
        }
    }

    impl Variable for TestVariable {
        fn get(&self) -> f64 {
            self.value
        }

        fn set(&mut self, _value: f64) {}
    }

    fn get_test_variable(value: f64) -> Rc<RefCell<TestVariable>> {
        Rc::new(RefCell::new(TestVariable::new(value)))
    }

    fn get_call_counter() -> Rc<RefCell<EventCallCounter>> {
        Rc::new(RefCell::new(EventCallCounter::new()))
    }

    fn process_then_set(var: &RefCell<TestVariable>, event: &mut Syncable, value: f64) {
        event.process_incoming(value);
        var.borrow_mut().set_new_value(value);
    }

    #[test]
    fn test_toggle_switch() {
        let var = get_test_variable(0.0);
        let event_counter = get_call_counter();
        let off_event_counter = get_call_counter();

        let mut toggle_switch = ToggleSwitch {
            var: var.clone(),
            event: event_counter.clone(),
            off_event: Some(off_event_counter.clone()),
            switch_on: false,
        };

        // Switch changed remotely, but was already in same position for client
        toggle_switch.process_incoming(0.0);
        assert_eq!(event_counter.borrow().called_count, 0);
        assert_eq!(off_event_counter.borrow().called_count, 0);
        // Regular case
        process_then_set(&var, &mut toggle_switch, 1.0);

        assert_eq!(event_counter.borrow().called_count, 1);
        assert_eq!(off_event_counter.borrow().called_count, 0);
        // Off case
        process_then_set(&var, &mut toggle_switch, 0.0);

        assert_eq!(event_counter.borrow().called_count, 1);
        assert_eq!(off_event_counter.borrow().called_count, 1);
        // Test switch_on
        toggle_switch.switch_on = true;

        var.borrow_mut().set_new_value(0.0);
        toggle_switch.process_incoming(1.0);

        assert_eq!(event_counter.borrow().called_count, 2);
        assert_eq!(off_event_counter.borrow().called_count, 1);
    }
}
