use simconnect;

const GROUP_ID: u32 = 5;

pub trait Syncable<T> {
    fn set_current(&mut self, current: T);
    fn set_new(&mut self, new: T, conn: &simconnect::SimConnector);
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
        self.current = new;
        conn.transmit_client_event(1, self.event_id, 0, GROUP_ID, 0);
    }
}

pub struct ToggleSwitchSet {
    event_id: u32,
    current: bool
}

impl ToggleSwitchSet {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id,
            current: false
        }
    }
}

impl Syncable<bool> for ToggleSwitchSet {
    fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    fn set_new(&mut self, new: bool, conn: &simconnect::SimConnector) {
        if self.current == new {return}
        match new {
            true => conn.transmit_client_event(1, self.event_id, 1, GROUP_ID, 0),
            false => conn.transmit_client_event(1, self.event_id, 0, GROUP_ID, 0)
        };
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
}
