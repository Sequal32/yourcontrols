use std::{rc::Rc, time::{Instant, Duration}};

use simconnect::SimConnector;

pub struct Control {
    has_control: bool,
    can_take_control: bool,

    relieving_control: bool,
    relieve_time: Instant,
    control_change_time: Instant
}

impl Control {
    pub fn new() -> Self{
        Self {
            has_control: false,
            can_take_control: true,
            relieving_control: false,

            relieve_time: Instant::now(),
            control_change_time: Instant::now()
        }
    }

    fn change_control(&self, conn: &SimConnector) {
        conn.transmit_client_event(1, 1000, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1001, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1002, !self.has_control as u32, 5, 0);
    }

    // Control was successful
    pub fn try_take_control(&mut self, conn: &SimConnector) -> bool {
        if !self.can_take_control {return false}

        self.take_control(conn);

        return true;
    }

    pub fn take_control(&mut self, conn: &SimConnector) {
        self.can_take_control = false;
        self.has_control = true;
        self.control_change_time = Instant::now();
        self.change_control(conn);
        self.stop_relieiving();
    }

    pub fn lose_control(&mut self, conn: &SimConnector) {
        self.has_control = false;
        self.relieving_control = false;
        self.change_control(conn);
    }

    pub fn has_control(&self) -> bool {
        return self.has_control;
    }

    pub fn relieve_control(&mut self) {
        self.relieving_control = true;
        self.relieve_time = Instant::now();
    }

    pub fn time_since_relieve(&self) -> Duration {
        return self.relieve_time.elapsed();
    }

    pub fn time_since_control_change(&self) -> Duration {
        return self.control_change_time.elapsed();
    }

    pub fn is_relieving_control(&self) -> bool {
        return self.relieving_control
    }

    pub fn stop_relieiving(&mut self) {
        self.relieving_control = false;
    }

    pub fn controls_available(&mut self) {
        self.can_take_control = true;
    }

    pub fn controls_unavailable(&mut self) {
        self.can_take_control = false;
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
        conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
        conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    }
}