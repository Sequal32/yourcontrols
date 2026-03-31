use std::time::{Duration, Instant};

use spin_sleep::sleep;

use crate::app::App;
use crate::cli::CliWrapper;
use crate::simconfig::Config;
use crate::sync::control::Control;
use crate::update::Updater;

mod app_loop;
mod network;
mod simconnect;
mod state;
mod sync;

use app_loop::{AppContext, AppController, AppState};
use network::{NetworkContext, NetworkController, NetworkState};
use simconnect::SimController;
use state::ProgramState;
use sync::{SyncController, SyncState};

const LOOP_SLEEP_TIME: Duration = Duration::from_millis(10);

/// Parameters for starting the server
pub struct StartServerParameters {
    pub method: crate::app::ConnectionMethod,
    pub is_ipv6: bool,
    pub use_upnp: bool,
}

// Temporary implementation of the whole program handler. All mutable references so this can be worked on incrementally.
pub struct Program {
    state: ProgramState,
    cli: CliWrapper,
    config: Config,
    control: Control,
    updater: Updater,
    sim: simconnect::SimState,
    network: NetworkState,
    sync: SyncState,
    app: AppState,
}

impl Program {
    pub fn new(cli: CliWrapper) -> Self {
        let mut config = Config::read_or_default();
        cli.apply_config_overrides(&mut config);

        let updater = Updater::new();
        let app_interface = App::setup(format!("YourControls v{}", updater.get_version()));

        Self {
            state: ProgramState::default(),
            cli,
            config,
            control: Control::new(),
            updater,
            sim: simconnect::SimState::new(),
            network: NetworkState::new(),
            sync: SyncState::new(),
            app: AppState::new(app_interface),
        }
    }

    pub fn run(&mut self) {
        loop {
            let timer = Instant::now();

            if self.network.has_client() {
                if let Some(action) = SimController::poll(&mut self.sim) {
                    NetworkController::apply_sim_action(&mut self.network, action);
                }

                if self.network.has_client() {
                    {
                        let mut net_ctx = NetworkContext {
                            sim: &mut self.sim,
                            control: &mut self.control,
                            config: &self.config,
                            cli: &self.cli,
                            updater: &self.updater,
                            app: &mut self.app.app_interface,
                            sync: &mut self.sync,
                        };

                        NetworkController::poll(&mut self.network, &mut net_ctx);
                    }

                    SyncController::tick(
                        &mut self.sync,
                        &mut self.network,
                        &mut self.sim,
                        &mut self.control,
                    );
                }
            }

            {
                let mut app_ctx = AppContext {
                    program_state: &mut self.state,
                    sim: &mut self.sim,
                    network: &mut self.network,
                    control: &mut self.control,
                    config: &mut self.config,
                    cli: &self.cli,
                    updater: &mut self.updater,
                };

                AppController::poll(&mut self.app, &mut app_ctx);
            }

            NetworkController::cleanup_if_needed(&mut self.network, &mut self.sync, &mut self.sim);

            if timer.elapsed().as_millis() < 10 {
                sleep(LOOP_SLEEP_TIME)
            };
            // Attempt Simconnect connection
            if self.app.app_interface.exited() || self.app.installer_spawned {
                break;
            }
        }
    }
}
