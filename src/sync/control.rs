use std::{time::{Instant}};

use simconnect::SimConnector;

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

    pub fn finalize_transfer(&mut self, conn: &SimConnector) {
        conn.transmit_client_event(1, 1000, !self.has_control as u32, 5, 0);
        // conn.transmit_client_event(1, 1001, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1002, !self.has_control as u32, 5, 0);
        self.transferring_control = false;
    }

    pub fn take_control(&mut self, conn: &SimConnector) {
        self.has_control = true;
        self.control_change_time = Instant::now();
        self.finalize_transfer(conn);
    }

    pub fn lose_control(&mut self) {
        self.has_control = false;
        self.control_change_time = Instant::now();
        self.transferring_control = true;
    }

    pub fn has_pending_transfer(&self) -> bool {
        return self.transferring_control;
    }

    pub fn has_control(&self) -> bool {
        return self.has_control;
    }

    pub fn time_since_control_change(&self) -> u64 {
        return self.control_change_time.elapsed().as_secs();
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
        conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
        conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    }
}