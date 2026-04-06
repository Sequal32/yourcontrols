use std::collections::HashMap;

use crate::{sync::gaugecommunicator::GaugeCommunicator, util::InDataTypes};

use super::transfer::AircraftVars;
use log::{debug, info};
use simconnect::SimConnector;
use yourcontrols_types::VarReaderTypes;

const CHECK_FREEZE_VARS: [&str; 3] = [
    "IS ALTITUDE FREEZE ON",
    "IS ATTITUDE FREEZE ON",
    "IS LATITUDE LONGITUDE FREEZE ON",
];

const FREEZE_EVENTS: [&str; 3] = [
    "FREEZE_ALTITUDE_SET",
    "FREEZE_ATTITUDE_SET",
    "FREEZE_LATITUDE_LONGITUDE_SET",
];

// Listens for updates on control and freezes the controls when the player doesn't have control
// Ensures the sim stays consistent with the player's control state as well
pub struct Freezer {
    has_control: bool,
    is_client: bool,
}

impl Freezer {
    pub fn new() -> Self {
        Self {
            has_control: false,
            is_client: false,
        }
    }

    // Start subscribing for sim events
    pub fn register_vars(&self, avars: &mut AircraftVars) {
        for var in CHECK_FREEZE_VARS {
            debug!("[Freezer] Subscribing to {}", var);
            avars.add_var(var, "Bool", InDataTypes::Bool);
        }
    }

    fn perform_transfer(&self, conn: &SimConnector, gauge: &GaugeCommunicator, has_control: bool) {
        let should_freeze = if has_control { "0" } else { "1" };
        info!(
            "[Freezer] Performing transfer, has_control: {}, freezing: {}",
            has_control, should_freeze
        );

        for event in FREEZE_EVENTS {
            debug!(
                "[Freezer] Sending raw message for {}: {}",
                event, should_freeze
            );
            gauge.send_raw(conn, &format!("{} (>K:{})", should_freeze, event));
        }
    }

    pub fn on_control_change(
        &mut self,
        conn: &SimConnector,
        gauge: &GaugeCommunicator,
        has_control: bool,
    ) {
        self.has_control = has_control;
        self.perform_transfer(conn, gauge, has_control);
    }

    pub fn on_vars_change(
        &self,
        vars: &HashMap<String, VarReaderTypes>,
        conn: &SimConnector,
        gauge: &GaugeCommunicator,
    ) {
        if !self.is_client {
            return;
        }

        for freeze_var in CHECK_FREEZE_VARS {
            let Some(VarReaderTypes::Bool(is_freeze_on)) = vars.get(freeze_var) else {
                continue;
            };

            let should_be_freezing = !self.has_control;

            debug!(
                "[Freezer] Checking var {}: {}, should be freezing: {}",
                freeze_var, is_freeze_on, should_be_freezing
            );

            if *is_freeze_on != should_be_freezing {
                info!("[Freezer] Detected mismatch for {}, correcting", freeze_var);
                self.perform_transfer(conn, gauge, self.has_control); // Blanket freeze and return early
                return;
            }
        }
    }

    pub fn has_control(&self) -> bool {
        self.has_control
    }

    pub fn set_is_client(&mut self, is_client: bool) {
        self.is_client = is_client;
    }
}
