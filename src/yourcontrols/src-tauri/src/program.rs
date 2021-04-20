use std::time::Duration;

use crate::ui::cmd::{Cmd, UIEvents};
use crate::ui::{AircraftInstallData, Ui};
use crate::{aircraft::DefinitionsUpdater, ui::GameUiPayloads};

pub struct Program {
    ui: Ui,
    definitions_updater: DefinitionsUpdater,
}

impl Program {
    pub fn setup() -> Self {
        let ui = Ui::run();

        Self {
            ui,
            definitions_updater: DefinitionsUpdater::new(),
        }
    }

    pub fn process_ui_events(&mut self) {
        match self.ui.get_pending_events_app() {
            Some(Cmd::UiReady) => {
                self.ui.send_message_app(UIEvents::StartUpText {
                    text: "Starting...".to_string(),
                });

                self.definitions_updater.fetch_data().expect("OK");

                self.ui.send_message_app(UIEvents::InitData {
                    version: std::env::var("CARGO_PKG_VERSION").unwrap(),
                    aircraft: self.definitions_updater.get_all_aircraft_info(),
                });

                // SIMULATE TIME LAG
                self.ui.send_message_app(UIEvents::LoadingComplete);
            }
            Some(Cmd::InstallAircraft { names }) => {}
            Some(Cmd::TestNetwork { port }) => {}
            None => {}
        }

        match self.ui.get_pending_events_game_ui() {
            Some(GameUiPayloads::Host { port, username }) => {}
            _ => {}
        }
    }

    pub fn poll(&mut self) {
        self.process_ui_events();
    }
}
