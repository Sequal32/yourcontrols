use std::{
    cmp::PartialOrd,
    fmt::Display,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use num::{FromPrimitive, ToPrimitive};
use simconnect;

use crate::{sync::transfer::LVarSyncer, util::NumberDigits};

const GROUP_ID: u32 = 5;

pub trait Syncable<T>
where
    T: Default,
{
    fn set_current(&mut self, current: T);
    fn set_new(&self, new: T, conn: &simconnect::SimConnector, lvar_transfer: &mut LVarSyncer);
}

pub struct ToggleSwitch {
    event_id: u32,
    // To be used in the event where two events control this switch
    off_event_id: Option<u32>,
    // To be used to select an index to control
    event_param: Option<u32>,
    // The presence of event_name determines whether to use the calculator
    event_name: Option<String>,
    // Only trigger if the new value is on
    switch_on: bool,
    // Current value of the switch
    current: bool,
}

impl ToggleSwitch {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id,
            off_event_id: None,
            event_param: None,
            event_name: None,
            switch_on: false,
            current: false,
        };
    }

    pub fn set_off_event(&mut self, off_event_id: u32) {
        self.off_event_id = Some(off_event_id);
    }

    pub fn set_calculator_event_name(&mut self, event_name: Option<&str>) {
        match event_name {
            Some(event_name) => self.event_name = Some(format!("K:{}", event_name)),
            None => self.event_name = None,
        }
    }

    pub fn set_param(&mut self, param: u32) {
        self.event_param = Some(param);
    }

    pub fn set_switch_on(&mut self, switch_on: bool) {
        self.switch_on = switch_on
    }
}

impl<'a> Syncable<bool> for ToggleSwitch {
    fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    fn set_new(&self, new: bool, conn: &simconnect::SimConnector, lvar_transfer: &mut LVarSyncer) {
        if self.current == new {
            return;
        }
        if !new && self.switch_on {
            return;
        }

        if let Some(event_name) = self.event_name.as_ref() {
            let value_string = match self.event_param {
                Some(value) => value.to_string(),
                None => String::new(),
            };

            lvar_transfer.set_unchecked(conn, event_name, None, &value_string);
        } else {
            let event_id = match self.off_event_id {
                Some(off_event_id) => match new {
                    true => self.event_id,
                    false => off_event_id,
                },
                None => self.event_id,
            };

            conn.transmit_client_event(1, event_id, self.event_param.unwrap_or(0), GROUP_ID, 0);
        }
    }
}

#[derive(Default)]
pub struct NumSet<T> {
    event_id: u32,
    event_name: Option<String>,
    event_param: Option<u32>,
    swap_event_id: Option<u32>,
    multiply_by: Option<T>,
    add_by: Option<T>,
    index_reversed: bool,

    current: T,
}

impl<T> NumSet<T>
where
    T: Default,
{
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id,
            ..Default::default()
        };
    }

    pub fn set_calculator_event_name(&mut self, event_name: Option<&str>, with_param: bool) {
        match event_name {
            Some(event_name) => match with_param {
                true => self.event_name = Some(format!("K:2:{}", event_name)),
                false => self.event_name = Some(format!("K:{}", event_name)),
            },
            None => self.event_name = None,
        }
    }

    pub fn set_swap_event(&mut self, event_id: u32) {
        self.swap_event_id = Some(event_id)
    }

    pub fn set_multiply_by(&mut self, multiply_by: T) {
        self.multiply_by = Some(multiply_by)
    }

    pub fn set_add_by(&mut self, add_by: T) {
        self.add_by = Some(add_by)
    }

    pub fn set_param(&mut self, event_param: u32, index_reversed: bool) {
        self.event_param = Some(event_param);
        self.index_reversed = index_reversed;
    }
}

impl<T> Syncable<T> for NumSet<T>
where
    T: Default
        + PartialEq
        + Mul<T, Output = T>
        + Add<T, Output = T>
        + FromPrimitive
        + ToPrimitive
        + Display
        + Copy,
{
    fn set_current(&mut self, current: T) {
        self.current = current
    }

    fn set_new(&self, new: T, conn: &simconnect::SimConnector, lvar_transfer: &mut LVarSyncer) {
        if new == self.current {
            return;
        }

        let value = match self.multiply_by.as_ref() {
            Some(multiply_by) => new * multiply_by.clone(),
            None => new,
        };

        let value = match self.add_by.as_ref() {
            Some(add_by) => value + add_by.clone(),
            None => value,
        };

        if let Some(event_name) = self.event_name.as_ref() {
            let value_string = match self.event_param {
                Some(event_param) => {
                    if self.index_reversed {
                        format!("{} {}", value, event_param)
                    } else {
                        format!("{} {}", event_param, value)
                    }
                }
                None => value.to_string(),
            };

            lvar_transfer.set_unchecked(conn, event_name, None, &value_string);
        } else {
            conn.transmit_client_event(
                1,
                self.event_id,
                value.to_i32().unwrap() as u32,
                GROUP_ID,
                0,
            );
        }

        if let Some(swap_event_id) = self.swap_event_id {
            conn.transmit_client_event(1, swap_event_id, 0, GROUP_ID, 0);
        }
    }
}

pub struct NumIncrement<T> {
    pub up_event_id: u32,
    pub down_event_id: u32,
    pub increment_amount: T,
    pub current: T,
    pub pass_difference: bool,
}

impl<T> NumIncrement<T>
where
    T: Default,
{
    pub fn new(up_event_id: u32, down_event_id: u32, increment_amount: T) -> Self {
        Self {
            up_event_id,
            down_event_id,
            increment_amount,
            current: Default::default(),
            pass_difference: false,
        }
    }

    pub fn set_pass_difference(&mut self, pass_difference: bool) {
        self.pass_difference = pass_difference
    }
}

impl<T> Syncable<T> for NumIncrement<T>
where
    T: Default + Sub<T, Output = T> + AddAssign + SubAssign + PartialOrd + Copy + ToPrimitive,
{
    fn set_current(&mut self, current: T) {
        self.current = current
    }

    fn set_new(&self, new: T, conn: &simconnect::SimConnector, _: &mut LVarSyncer) {
        let mut working = self.current;

        if self.pass_difference {
            if new > self.current {
                conn.transmit_client_event(
                    1,
                    self.up_event_id,
                    (new - self.current).to_i32().unwrap() as u32,
                    GROUP_ID,
                    0,
                );
            } else if new < self.current {
                conn.transmit_client_event(
                    1,
                    self.down_event_id,
                    (self.current - new).to_i32().unwrap() as u32,
                    GROUP_ID,
                    0,
                );
            }
        } else {
            while working > new {
                working -= self.increment_amount;
                conn.transmit_client_event(1, self.down_event_id, 0, GROUP_ID, 0);
            }

            while working < new {
                working += self.increment_amount;
                conn.transmit_client_event(1, self.up_event_id, 0, GROUP_ID, 0);
            }
        }
    }
}

pub struct NumDigitSet {
    pub inc_event_ids: Vec<u32>,
    pub dec_event_ids: Vec<u32>,
    pub current: NumberDigits,
}

impl NumDigitSet {
    pub fn new(inc_event_ids: Vec<u32>, dec_event_ids: Vec<u32>) -> Self {
        Self {
            inc_event_ids,
            dec_event_ids,
            current: NumberDigits::new(0),
        }
    }
}

impl Syncable<i32> for NumDigitSet {
    fn set_current(&mut self, current: i32) {
        self.current = NumberDigits::new(current)
    }

    fn set_new(&self, new: i32, conn: &simconnect::SimConnector, _: &mut LVarSyncer) {
        let new = NumberDigits::new(new);

        for index in 0..self.inc_event_ids.len() {
            let new_value = new.get(index);
            let mut working_value = self.current.get(index);

            while working_value > new_value {
                working_value -= 1;
                conn.transmit_client_event(1, self.dec_event_ids[index], 0, GROUP_ID, 0);
            }

            while working_value < new_value {
                working_value += 1;
                conn.transmit_client_event(1, self.inc_event_ids[index], 0, GROUP_ID, 0);
            }
        }
    }
}
pub struct CustomCalculator {
    set_string: String,
    current: f64,
}

impl CustomCalculator {
    pub fn new(set_string: String) -> Self {
        Self {
            set_string,
            current: 0.0,
        }
    }
}

impl Syncable<f64> for CustomCalculator {
    fn set_current(&mut self, new: f64) {
        self.current = new
    }

    fn set_new(&self, new: f64, conn: &simconnect::SimConnector, transfer: &mut LVarSyncer) {
        if self.current == new {
            return;
        }
        transfer.send_raw(conn, &self.set_string);
    }
}
