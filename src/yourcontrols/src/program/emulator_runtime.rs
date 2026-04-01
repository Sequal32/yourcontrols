use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::app::App;
use crate::definitions::{Definitions, SyncPermission};
use simconnect::SimConnector;
use yourcontrols_net::TransferClient;

const EMULATOR_TICK: Duration = Duration::from_millis(200);

pub struct EmulatorRuntimeState {
    pub enabled: bool,
    pub tracked: HashSet<String>,
    pub last_sent: HashMap<String, f64>,
    pub last_tick: Instant,
}

impl Default for EmulatorRuntimeState {
    fn default() -> Self {
        Self {
            enabled: false,
            tracked: HashSet::new(),
            last_sent: HashMap::new(),
            last_tick: Instant::now(),
        }
    }
}

pub struct EmulatorController;

pub struct EmulatorSetContext<'a> {
    pub definitions: &'a mut Definitions,
    pub conn: &'a SimConnector,
    pub client: Option<&'a dyn TransferClient>,
    pub app: &'a App,
}

impl EmulatorController {
    pub fn set_enabled(state: &mut EmulatorRuntimeState, app: &App, enabled: bool) {
        state.enabled = enabled;
        app.emulator_enabled(enabled);
    }

    pub fn send_vars_if_enabled(
        state: &EmulatorRuntimeState,
        definitions: &Definitions,
        app: &App,
    ) {
        if !state.enabled {
            return;
        }

        let vars = definitions.get_emulator_vars();
        if let Ok(payload) = serde_json::to_string(&vars) {
            app.send_emulator_vars(&payload);
        }
    }

    pub fn request_vars(state: &EmulatorRuntimeState, definitions: &Definitions, app: &App) {
        if !state.enabled {
            return;
        }

        let vars = definitions.get_emulator_vars();
        if vars.is_empty() {
            app.emulator_error("No variables available. Load an aircraft config first.");
        } else if let Ok(payload) = serde_json::to_string(&vars) {
            app.send_emulator_vars(&payload);
        }
    }

    pub fn add_var(
        state: &mut EmulatorRuntimeState,
        definitions: &Definitions,
        app: &App,
        var_id: &str,
    ) {
        if !state.enabled {
            return;
        }

        state.tracked.insert(var_id.to_string());
        match definitions.get_emulator_var_value(var_id) {
            Some(info) => {
                if let Ok(payload) = serde_json::to_string(&info) {
                    app.send_emulator_var_value(&payload);
                }
            }
            None => app.emulator_error("Unknown variable. Reload the definitions and try again."),
        }
    }

    pub fn remove_var(state: &mut EmulatorRuntimeState, var_id: &str) {
        if !state.enabled {
            return;
        }

        state.tracked.remove(var_id);
        state.last_sent.remove(var_id);
    }

    pub fn set_var(
        state: &EmulatorRuntimeState,
        ctx: &mut EmulatorSetContext<'_>,
        var_id: &str,
        value: f64,
    ) {
        if !state.enabled {
            return;
        }

        let mut apply_result = Ok(());
        if let Some(client) = ctx.client {
            if client.is_host() {
                let permission = SyncPermission {
                    is_server: true,
                    is_master: ctx.definitions.has_control(),
                    is_init: true,
                };
                apply_result = ctx.definitions.apply_emulator_value_to_sim(
                    ctx.conn,
                    var_id,
                    value,
                    &permission,
                );
            } else {
                apply_result = ctx.definitions.apply_emulator_value(var_id, value);
            }
        }

        if let Err(e) = apply_result {
            ctx.app
                .emulator_error(format!("Could not set variable: {}", e).as_str());
        } else if let Some(info) = ctx.definitions.get_emulator_var_value(var_id) {
            if let Ok(payload) = serde_json::to_string(&info) {
                ctx.app.send_emulator_var_value(&payload);
            }
        }
    }

    pub fn tick(
        state: &mut EmulatorRuntimeState,
        client: Option<&dyn TransferClient>,
        definitions: &Definitions,
        app: &App,
    ) {
        if !state.enabled || client.is_none() || state.tracked.is_empty() {
            return;
        }

        if state.last_tick.elapsed() < EMULATOR_TICK {
            return;
        }

        state.last_tick = Instant::now();

        for var_id in state.tracked.iter() {
            if let Some(info) = definitions.get_emulator_var_value(var_id) {
                if let Some(value) = info.value {
                    let should_send = state
                        .last_sent
                        .get(var_id)
                        .map(|last| (last - value).abs() > f64::EPSILON)
                        .unwrap_or(true);

                    if should_send {
                        if let Ok(payload) = serde_json::to_string(&info) {
                            app.send_emulator_var_value(&payload);
                        }
                        state.last_sent.insert(var_id.clone(), value);
                    }
                }
            }
        }
    }
}
