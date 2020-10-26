use simconnect;

use crate::{sync::LVarSyncer, util::NumberDigits};

const GROUP_ID: u32 = 5;

pub trait Syncable<T> where T: Default {
    fn set_current(&mut self, current: T);
    fn set_new(&mut self, new: T, conn: &simconnect::SimConnector);
    fn get_multiply_by(&self) -> T {return Default::default()}
}

pub struct ToggleSwitch {
    pub event_id: u32,
    pub current: bool
}

impl ToggleSwitch {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id,
            current: false
        }
    }
}

impl Syncable<bool> for ToggleSwitch {
    fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    fn set_new(&mut self, new: bool, conn: &simconnect::SimConnector) {
        if self.current == new {return}
        conn.transmit_client_event(1, self.event_id, 0, GROUP_ID, 0);
    }
}

pub struct ToggleSwitchParam {
    pub event_id: u32,
    pub param: u32,
    pub current: bool
}

impl ToggleSwitchParam {
    pub fn new(event_id: u32, param: u32) -> Self {
        return Self {
            event_id: event_id,
            param,
            current: false
        }
    }
}

impl Syncable<bool> for ToggleSwitchParam {
    fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    fn set_new(&mut self, new: bool, conn: &simconnect::SimConnector) {
        if self.current == new {return}
        conn.transmit_client_event(1, self.event_id, self.param, GROUP_ID, 0);
    }
}

pub struct ToggleSwitchTwo {
    pub off_event_id: u32,
    pub on_event_id: u32,
    pub current: bool,
    pub param: Option<u32>
}

impl ToggleSwitchTwo {
    pub fn new(off_event_id: u32, on_event_id: u32, param: Option<u32>) -> Self { 
        Self { 
            off_event_id, 
            on_event_id, 
            current: false,
            param
        } 
    }
}

impl Syncable<bool> for ToggleSwitchTwo {
    fn set_current(&mut self, current: bool) {
        self.current = current
    }

    fn set_new(&mut self, new: bool, conn: &simconnect::SimConnector) {
        if self.current == new {return}
        let event_id = if new {self.on_event_id} else {self.off_event_id};
        conn.transmit_client_event(1, event_id, self.param.unwrap_or(0), GROUP_ID, 0);
    }
}

pub struct SwitchOn {
    pub event_id: u32,
    pub event_param: Option<u32>,
    pub current: bool,
}

impl SwitchOn {
    pub fn new(event_id: u32, event_param: Option<u32>) -> Self { 
        Self { 
            event_id, 
            event_param,
            current: false
        } 
    }
}

impl Syncable<bool> for SwitchOn {
    fn set_current(&mut self, current: bool) {
        self.current = current
    }

    fn set_new(&mut self, new: bool, conn: &simconnect::SimConnector) {
        if new == false {return}
        conn.transmit_client_event(1, self.event_id, self.event_param.unwrap_or(0), GROUP_ID, 0);
    }
}

pub struct NumSet {
    pub event_id: u32,
    pub current: i32
}

impl NumSet {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id,
            current: 0
        }
    }
}

impl Syncable<i32> for NumSet {
    fn set_current(&mut self, current: i32) {
        self.current = current
    }

    fn set_new(&mut self, new: i32, conn: &simconnect::SimConnector) {
        if new == self.current {return}
        conn.transmit_client_event(1, self.event_id, new as u32, GROUP_ID, 0);
    }
}

pub struct NumSetSwap {
    pub event_id: u32,
    pub swap_event_id: u32,
    pub current: i32,
}

impl NumSetSwap {
    pub fn new(event_id: u32, swap_event_id: u32) -> Self {
        return Self {
            event_id,
            swap_event_id,
            current: 0
        }
    }
}

impl Syncable<i32> for NumSetSwap {
    fn set_current(&mut self, current: i32) {
        self.current = current
    }

    fn set_new(&mut self, new: i32, conn: &simconnect::SimConnector) {
        if new == self.current {return}
        conn.transmit_client_event(1, self.event_id, new as u32, GROUP_ID, 0);
        conn.transmit_client_event(1, self.swap_event_id, 0, GROUP_ID, 0);
    }
}

pub struct NumSetMultiply {
    pub event_id: u32,
    pub current: i32,
    pub multiply_by: i32
}

impl NumSetMultiply {
    pub fn new(event_id: u32, multiply_by: i32) -> Self {
        Self {
            event_id,
            current: 0,
            multiply_by
        }
    }
}

impl Syncable<i32> for NumSetMultiply {
    fn set_current(&mut self, current: i32) {
        self.current = current
    }

    fn set_new(&mut self, new: i32, conn: &simconnect::SimConnector) {
        if new == self.current {return}
        conn.transmit_client_event(1, self.event_id, (new * self.multiply_by) as u32, GROUP_ID, 0);    
    }

    fn get_multiply_by(&self) -> i32 {return self.multiply_by}
}

pub struct NumIncrement<T> {
    pub up_event_id: u32,
    pub down_event_id: u32,
    pub increment_amount: T,
    pub current: T
}

impl<T> NumIncrement<T>  where T: Default {
    pub fn new(up_event_id: u32, down_event_id: u32, increment_amount: T) -> Self { 
        Self { 
            up_event_id, 
            down_event_id, 
            increment_amount,
            current: Default::default()
        } 
    }
}

impl<T> Syncable<T> for NumIncrement<T> where T: Default + std::ops::SubAssign + std::ops::AddAssign + std::cmp::PartialOrd + Copy {
    fn set_current(&mut self, current: T) {
        self.current = current
    }

    fn set_new(&mut self, new: T, conn: &simconnect::SimConnector) {
        let mut working = self.current;

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

pub struct NumIncrementSet {
    pub up_event_id: u32,
    pub down_event_id: u32,
    pub current: i32,
}

impl NumIncrementSet {
    pub fn new(up_event_id: u32, down_event_id: u32) -> Self {
        return Self {
            up_event_id,
            down_event_id,
            current: 0
        }
    }
}

impl Syncable<i32> for NumIncrementSet {
    fn set_current(&mut self, current: i32) {
        self.current = current
    }

    fn set_new(&mut self, new: i32, conn: &simconnect::SimConnector) {
        if new == self.current {return}
        if new > self.current {
            conn.transmit_client_event(1, self.up_event_id, (new - self.current) as u32, GROUP_ID, 0);
        } else if new < self.current {
            conn.transmit_client_event(1, self.down_event_id,  (self.current - new) as u32, GROUP_ID, 0);
        }
    }
}

pub struct NumDigitSet {
    pub inc_event_ids: Vec<u32>,
    pub dec_event_ids: Vec<u32>,
    pub current: NumberDigits
}

impl NumDigitSet {
    pub fn new(inc_event_ids: Vec<u32>, dec_event_ids: Vec<u32>) -> Self { 
        Self { 
            inc_event_ids, 
            dec_event_ids, 
            current: NumberDigits::new(0)
        } 
    }
}

impl Syncable<i32> for NumDigitSet {
    fn set_current(&mut self, current: i32) {
        self.current = NumberDigits::new(current)
    }

    fn set_new(&mut self, new: i32, conn: &simconnect::SimConnector) {
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
    current: f64
}

impl CustomCalculator {
    pub fn new(set_string: String) -> Self { 
        Self {
            set_string, current: 0.0
        }
    }

    pub fn set_current(&mut self, new: f64) {
        self.current = new
    }

    pub fn set_new(&self, new: f64, conn: &simconnect::SimConnector, transfer: &mut LVarSyncer) {
        if self.current == new {return}
        transfer.send_raw(conn, &self.set_string);
    }
}