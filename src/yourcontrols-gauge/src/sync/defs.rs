use super::util::NumberDigits;
use crate::data::{RcSettable, RcVariable, Settable, Syncable, Variable};
use crate::util::DatumValue;

/// A ToggleSwitch will execute `event` if the incoming value does not match its current value in `var`.
///
/// `off_event` will be used if the incoming value is 0.0 (false)
///
/// `switch_on` will only trigger `event` if the incoming value is 1.0 (true)
pub struct ToggleSwitch {
    pub var: RcVariable,
    // Either the toggle event or the off event
    pub event: RcSettable,
    // To be used in the event where two events control this switch
    pub off_event: Option<RcSettable>,
    // Only trigger if the new value is on
    pub switch_on: bool,
}

impl Syncable for ToggleSwitch {
    fn process_incoming(&mut self, value: DatumValue) {
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
}

/// NumSet will execute `event` with the parameter of the incoming value.
/// `swap_event` will be executed after `event` is executed.
///
/// The value will be multiplied by `multiply_by` if specified, or added if `add_by` is specified.
///
/// The implmentation of both multiply_by and add_by being set is undefined.
pub struct NumSet {
    pub var: RcVariable,
    pub event: RcSettable,
    pub swap_event: Option<RcSettable>,
    pub multiply_by: Option<DatumValue>,
    pub add_by: Option<DatumValue>,
}

impl Syncable for NumSet {
    fn process_incoming(&mut self, value: DatumValue) {
        let value = value * self.multiply_by.unwrap_or(1.0) + self.add_by.unwrap_or(0.0);
        let mut event = self.event.borrow_mut();

        event.set_with_value(value);

        if let Some(swap_event) = self.swap_event.as_ref() {
            swap_event.borrow_mut().set();
        }
    }
}

/// NumIncrement continuiously calls `up_event` and `down_event` the number of times it takes for the stored value in `var` to match the incoming value
/// where `increment_amount` specifies how much each call to `up_event` and `down_event` increments.
///
/// Alternatively, if `pass_difference` is set to true, the difference of the stored value in `var` and the incoming value will be
/// passed as a parameter to the relevent event instead.
pub struct NumIncrement {
    pub var: RcVariable,
    pub up_event: RcSettable,
    pub down_event: RcSettable,
    pub increment_amount: DatumValue,
    pub pass_difference: bool,
}

impl Syncable for NumIncrement {
    fn process_incoming(&mut self, value: DatumValue) {
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
}

/// NumDigitSet splits a float's left of the decimal digits into an array of u8,
/// and then calls the corresponding digit's events in `inc_events` and `dec_events` continuously
/// until the value stored in var matches the incoming value.
///
/// `inc_events` should be the same length as `dec_events`.
pub struct NumDigitSet {
    pub var: RcVariable,
    pub inc_events: Vec<RcSettable>,
    pub dec_events: Vec<RcSettable>,
}

impl Syncable for NumDigitSet {
    fn process_incoming(&mut self, value: DatumValue) {
        let current = NumberDigits::new(self.var.borrow().get() as i32);
        let value = NumberDigits::new(value as i32);

        for index in 0..self.inc_events.len() {
            let mut working_value = current.get(index);
            let new_digit = value.get(index);

            let mut down_event = match self.dec_events.get(index) {
                Some(e) => e.borrow_mut(),
                None => return,
            };

            let mut up_event = match self.inc_events.get(index) {
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
}

#[cfg(test)]
mod tests {
    use super::{NumDigitSet, NumIncrement, NumSet, RcSettable, ToggleSwitch};
    use crate::data::{Settable, Syncable, Variable};
    use crate::util::test::{get_call_counter, get_test_variable, process_then_set};

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

    #[test]
    fn test_num_set() {
        let var = get_test_variable(0.0);
        let event_counter = get_call_counter();
        let swap_event_counter = get_call_counter();

        let mut num_set = NumSet {
            var: var.clone(),
            event: event_counter.clone(),
            swap_event: None,
            multiply_by: None,
            add_by: None,
        };

        // Test regular set
        num_set.process_incoming(1.0);

        assert_eq!(event_counter.borrow().last_set_value, 1.0);
        assert_eq!(event_counter.borrow().called_count, 1);

        // Test swap after set
        num_set.swap_event = Some(swap_event_counter.clone());
        num_set.process_incoming(1.0);

        assert_eq!(event_counter.borrow().last_set_value, 1.0);
        assert_eq!(event_counter.borrow().called_count, 2);
        assert_eq!(swap_event_counter.borrow().called_count, 1);

        // Test multiply
        num_set.multiply_by = Some(2.0);
        num_set.process_incoming(1.0);

        assert_eq!(event_counter.borrow().last_set_value, 2.0);
        assert_eq!(event_counter.borrow().called_count, 3);

        // Test add (multiply + add together is undefined)
        num_set.multiply_by = None;
        num_set.add_by = Some(100.0);
        num_set.process_incoming(1.0);

        assert_eq!(event_counter.borrow().last_set_value, 101.0);
        assert_eq!(event_counter.borrow().called_count, 4);
    }

    #[test]
    fn test_num_increment() {
        let var = get_test_variable(250.0);
        let up_event_counter = get_call_counter();
        let down_event_counter = get_call_counter();

        let mut num_increment = NumIncrement {
            var: var.clone(),
            up_event: up_event_counter.clone(),
            down_event: down_event_counter.clone(),
            increment_amount: 1.0,
            pass_difference: false,
        };

        // Regular deincrement
        num_increment.process_incoming(150.0);
        assert_eq!(down_event_counter.borrow().called_count, 100);
        // Regular increment
        num_increment.process_incoming(350.0);
        assert_eq!(up_event_counter.borrow().called_count, 100);
        //
        down_event_counter.borrow_mut().reset();
        up_event_counter.borrow_mut().reset();
        // Change deincrement amount
        num_increment.increment_amount = 5.0;
        num_increment.process_incoming(150.0);
        assert_eq!(down_event_counter.borrow().called_count, 20);
        // Change increment amount
        num_increment.process_incoming(350.0);
        assert_eq!(up_event_counter.borrow().called_count, 20);
        //
        down_event_counter.borrow_mut().reset();
        up_event_counter.borrow_mut().reset();
        // Pass difference deincrmeent
        num_increment.pass_difference = true;
        num_increment.process_incoming(150.0);

        assert_eq!(down_event_counter.borrow().called_count, 1);
        assert_eq!(down_event_counter.borrow().last_set_value, 100.0);
        // Pass difference deincrmeent
        num_increment.process_incoming(350.0);

        assert_eq!(up_event_counter.borrow().called_count, 1);
        assert_eq!(up_event_counter.borrow().last_set_value, 100.0);
    }

    #[test]
    fn test_num_digit_set() {
        let var = get_test_variable(3829.5); // make sure floating is truncated
        let inc_events = vec![
            get_call_counter(),
            get_call_counter(),
            get_call_counter(),
            get_call_counter(),
        ];
        let dec_events = vec![
            get_call_counter(),
            get_call_counter(),
            get_call_counter(),
            get_call_counter(),
        ];

        let mut num_digit_set = NumDigitSet {
            var: var.clone(),
            inc_events: inc_events.iter().map(|x| x.clone() as RcSettable).collect(),
            dec_events: dec_events.iter().map(|x| x.clone() as RcSettable).collect(),
        };

        num_digit_set.process_incoming(1679.0);

        assert_eq!(dec_events[0].borrow().called_count, 2);
        assert_eq!(dec_events[1].borrow().called_count, 2);
        assert_eq!(inc_events[2].borrow().called_count, 5);
        assert_eq!(dec_events[3].borrow().called_count, 0);
        // Should not have been called
        assert_eq!(inc_events[0].borrow().called_count, 0);
        assert_eq!(inc_events[1].borrow().called_count, 0);
        assert_eq!(dec_events[2].borrow().called_count, 0);
        assert_eq!(inc_events[3].borrow().called_count, 0);

        num_digit_set.process_incoming(4908.0);

        assert_eq!(inc_events[0].borrow().called_count, 1);
        assert_eq!(inc_events[1].borrow().called_count, 1);
        assert_eq!(dec_events[2].borrow().called_count, 2);
        assert_eq!(dec_events[3].borrow().called_count, 1);
    }
}
