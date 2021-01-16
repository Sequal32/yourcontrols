use std::{time::{Instant}};

pub struct Control {
    has_control: bool,
    // Used to seemlessly transfer control
    transferring_control: bool,
    control_change_time: Instant
}

impl Control {
    pub fn new() -> Self{
        Self {
            has_control: false,
            transferring_control: false,
            control_change_time: Instant::now()
        }
    }

    pub fn take_control(&mut self) {
        self.has_control = true;
        self.control_change_time = Instant::now();
    }

    pub fn lose_control(&mut self) {
        self.has_control = false;
        self.control_change_time = Instant::now();
        self.transferring_control = true;
    }

    pub fn has_control(&self) -> bool {
        return self.has_control;
    }
}