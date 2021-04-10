use simconnect::SimConnector;

use super::gaugecommunicator::GaugeCommunicator;

pub struct Control {
    has_control: bool,
}

impl Control {
    pub fn new() -> Self {
        Self { has_control: false }
    }

    pub fn do_transfer(&mut self, conn: &SimConnector) {
        conn.transmit_client_event(1, 1000, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1001, !self.has_control as u32, 5, 0);
        conn.transmit_client_event(1, 1002, !self.has_control as u32, 5, 0);
    }

    pub fn take_control(&mut self, conn: &SimConnector, gauge_communicator: &GaugeCommunicator) {
        self.has_control = true;
        self.do_transfer(conn);
        gauge_communicator.stop_interpolation(conn);
        // A32NX enable FBW
        gauge_communicator.set(conn, "L:A32NX_EXTERNAL_OVERRIDE", None, "0");
    }

    pub fn lose_control(&mut self, conn: &SimConnector, gauge_communicator: &GaugeCommunicator) {
        self.has_control = false;
        self.do_transfer(conn);
        // A32NX disable FBW
        gauge_communicator.set(conn, "L:A32NX_EXTERNAL_OVERRIDE", None, "1");
    }

    pub fn has_control(&self) -> bool {
        return self.has_control;
    }

    pub fn on_connected(&mut self, conn: &SimConnector) {
        conn.map_client_event_to_sim_event(1000, "FREEZE_LATITUDE_LONGITUDE_SET");
        conn.map_client_event_to_sim_event(1001, "FREEZE_ALTITUDE_SET");
        conn.map_client_event_to_sim_event(1002, "FREEZE_ATTITUDE_SET");
    }
}
