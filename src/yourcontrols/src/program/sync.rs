use std::time::Instant;

use log::info;
use yourcontrols_net::TransferClient;

use crate::definitions::{ProgramAction, SyncPermission};

use super::network::NetworkState;
use super::simconnect::SimState;

pub struct SyncState {
    pub(crate) ready_to_process_data: bool,
    pub(crate) connection_time: Option<Instant>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            ready_to_process_data: false,
            connection_time: None,
        }
    }
}

pub struct SyncController;

impl SyncController {
    pub fn tick(state: &mut SyncState, network: &mut NetworkState, sim: &mut SimState) {
        let Some(mut client) = network.transfer_client.take() else {
            return;
        };

        if let Err(e) = sim.definitions.step(&sim.conn) {
            client.stop(e.to_string());
        }

        // Handle specific program triggered actions
        if let Some(pending_action) = sim.definitions.get_next_pending_action() {
            SyncHandler::handle_program_action(pending_action, &mut client, network, sim);
        }

        // Handle initial 3 second connection delay, allows lvars to be processed
        if let Some(true) = state.connection_time.map(|t| t.elapsed().as_secs() >= 3) {
            // Do not let server send initial data - wait for data to get cleared on the previous loop
            if !network.observing && state.ready_to_process_data {
                let permission = SyncPermission {
                    is_server: client.is_host(),
                    is_master: sim.definitions.has_control(),
                    is_init: false,
                };

                SyncHandler::write_update_data(
                    sim.definitions.get_sync(&permission),
                    &mut client,
                    true,
                );
            }

            // Tell server we're ready to receive data after 3 seconds
            if !state.ready_to_process_data {
                state.ready_to_process_data = true;
                sim.definitions.reset_sync();

                if !client.is_host() {
                    client.send_ready();
                }
            }
        }

        network.transfer_client = Some(client);
    }
}

pub struct SyncHandler;

impl SyncHandler {
    fn handle_program_action(
        pending_action: ProgramAction,
        client: &mut Box<dyn TransferClient>,
        network: &mut NetworkState,
        sim: &mut SimState,
    ) {
        match pending_action {
            ProgramAction::TakeControls => {
                if !sim.has_control() && !network.observing {
                    if let Some(in_control) = network.clients.get_client_in_control() {
                        sim.take_control();
                        client.take_control(in_control.clone());
                    }
                }
            }
            ProgramAction::TransferControls => {
                if sim.has_control() {
                    if let Some(next_control) = network.clients.get_next_client_for_control() {
                        client.transfer_control(next_control.clone())
                    }
                } else if let Some(in_control) = network.clients.get_client_in_control() {
                    sim.lose_control();
                    client.take_control(in_control.clone());
                }
            }
        }
    }

    fn write_update_data(
        data: (
            Option<yourcontrols_types::AllNeedSync>,
            Option<yourcontrols_types::AllNeedSync>,
        ),
        client: &mut Box<dyn TransferClient>,
        log_sent: bool,
    ) {
        let (unreliable, reliable) = data;

        if let Some(data) = unreliable {
            client.update(data, true);
        }

        if let Some(data) = reliable {
            if log_sent {
                info!("[PACKET] SENT {:?}", data);
            }

            client.update(data, false);
        }
    }
}
