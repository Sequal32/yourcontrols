use simconnect;

use crate::util::NumberDigits;

const GROUP_ID: u32 = 5;

pub trait Syncable<T> where T: Default {
    fn set_current(&mut self, current: T);
    fn set_new(&mut self, new: T, conn: &simconnect::SimConnector);
    fn get_multiply_by(&self) -> T {return Default::default()}
}

pub struct ToggleSwitch {
    event_id: u32,
    current: bool
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
    event_id: u32,
    param: u32,
    current: bool
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
    off_event_id: u32,
    on_event_id: u32,
    current: bool
}

impl ToggleSwitchTwo {
    pub fn new(off_event_id: u32, on_event_id: u32) -> Self { 
        Self { 
            off_event_id, 
            on_event_id, 
            current: false
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
        conn.transmit_client_event(1, event_id, 0, GROUP_ID, 0);
    }
}

pub struct SwitchOn {
    event_id: u32,
    current: bool
}

impl SwitchOn {
    pub fn new(event_id: u32) -> Self { 
        Self { 
            event_id, 
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
        conn.transmit_client_event(1, self.event_id, 0, GROUP_ID, 0);
    }
}

pub struct NumSet {
    event_id: u32,
    current: i32
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
    event_id: u32,
    swap_event_id: u32,
    current: i32,
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
    event_id: u32,
    current: i32,
    multiply_by: i32
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
    up_event_id: u32,
    down_event_id: u32,
    increment_amount: T,
    current: T
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
    up_event_id: u32,
    down_event_id: u32,
    current: i32,
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
    inc_event_ids: Vec<u32>,
    dec_event_ids: Vec<u32>,
    current: NumberDigits
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


