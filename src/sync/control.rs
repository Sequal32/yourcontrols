use std::{time::{Instant, Duration}};

use simconnect::SimConnector;

pub struct Control {
    has_control: bool,
    control_change_time: Instant
}

impl Control {
    pub fn new() -> Self{
        Self {
            has_control: false,
            control_change_time: Instant::now()
        }
    }

    fn change_control(&self, conn: &SimConnector) {
        conn.transmit_client_event(1, 1000, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1001, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1002, !self.has_control as u32, 5, 0);
    }

    pub fn take_control(&mut self, conn: &SimConnector) {
        self.has_control = true;
        self.control_change_time = Instant::now();
        self.change_control(conn);
    }

    pub fn lose_control(&mut self, conn: &SimConnector) {
        self.has_control = false;
        self.change_control(conn);
    }

    pub fn has_control(&self) -> bool {
        return self.has_control;
    }

    pub fn time_since_control_change(&self) -> Duration {
        return self.control_change_time.elapsed();
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
        conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
        conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    }
}