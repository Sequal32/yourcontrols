use log::warn;
use simconnect::{DispatchResult, SimConnector};

use crate::definitions::Definitions;

pub struct SimState {
    pub(crate) conn: SimConnector,
    pub(crate) definitions: Definitions,
}

impl SimState {
    pub fn new() -> Self {
        Self {
            conn: SimConnector::new(),
            definitions: Definitions::new(),
        }
    }

    pub fn has_control(&self) -> bool {
        self.definitions.has_control()
    }

    pub fn take_control(&mut self) {
        self.definitions.on_control_change(&self.conn, true);
    }
    pub fn lose_control(&mut self) {
        self.definitions.on_control_change(&self.conn, false);
    }
}

pub enum SimAction {
    StopTransfer(String),
}

pub struct SimController;

impl SimController {
    pub fn poll(state: &mut SimState) -> Option<SimAction> {
        while let Ok(message) = state.conn.get_next_message() {
            if let Some(action) =
                SimHandler::handle_message(&state.conn, &mut state.definitions, message)
            {
                return Some(action);
            }
        }

        None
    }
}

pub struct SimHandler;

impl SimHandler {
    fn handle_message(
        conn: &SimConnector,
        definitions: &mut Definitions,
        message: DispatchResult,
    ) -> Option<SimAction> {
        match message {
            DispatchResult::SimObjectData(data) => {
                definitions.process_sim_object_data(conn, data);
                None
            }
            DispatchResult::Exception(data) => {
                let exception = unsafe { std::ptr::addr_of!(data.dwException).read_unaligned() };
                warn!("[SIM] SimConnect exception occurred: {}", exception);

                if exception == 31 {
                    return Some(SimAction::StopTransfer(
                        "Could not connect to the YourControls gauge. Do you have the community package installed correctly?"
                            .to_string(),
                    ));
                }

                None
            }
            DispatchResult::ClientData(data) => {
                definitions.process_client_data(data);
                None
            }
            DispatchResult::Event(data) => {
                definitions.process_event_data(data);
                None
            }
            DispatchResult::Quit(_) => Some(SimAction::StopTransfer("Sim closed.".to_string())),
            _ => None,
        }
    }
}
