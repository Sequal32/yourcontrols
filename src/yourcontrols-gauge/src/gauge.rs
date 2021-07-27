#![cfg(any(target_arch = "wasm32"))]

use anyhow::Result;
use msfs::legacy;
use msfs::sim_connect::{
    client_data_definition, data_definition, ClientDataArea, Period, SimConnect, SimConnectRecv,
};

use std::rc::Rc;

use crate::data::datum::{Condition, Datum, DatumManager, MappingArgs};
use crate::data::watcher::VariableWatcher;
use crate::data::{EventSet, GenericVariable, KeyEvent, RcSettable, RcVariable};
use crate::sync::SCRIPTING_ENGINE;
use crate::util::map_ids;
use crate::util::{GenericResult, DATA_SIZE};

use yourcontrols_types::{
    DatumKey, DatumMessage, EventMessage, MappingType, MessagePackFragmenter, Payloads,
    ScriptMessage, VarType,
};

/// A wrapper struct for an array of size DATA_SIZE bytes
#[client_data_definition]
struct ClientData {
    inner: [u8; DATA_SIZE],
}

impl ClientData {
    fn new() -> Self {
        Self {
            inner: [0; DATA_SIZE],
        }
    }
}

#[data_definition]
struct AircraftData {
    #[name = "BRAKE PARKING POSITION"]
    #[unit = "Bool"]
    parking_brake: bool,
}

/// The main driver to process and send out messages through SimConnect.
pub struct MainGauge {
    fragmenter: MessagePackFragmenter,
    datum_manager: DatumManager<Datum>,
    send_data_area: Option<ClientDataArea<ClientData>>,

    // Vars/Events
    vars: Vec<RcVariable>,
    events: Vec<RcSettable>,
}

impl MainGauge {
    pub fn new() -> Self {
        Self {
            fragmenter: MessagePackFragmenter::new(DATA_SIZE - 16), // Leave 16 bytes free for header
            datum_manager: DatumManager::new(),
            send_data_area: None,

            vars: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Creates/Setup ClientDataAreas for communication
    pub fn setup(&mut self, simconnect: &mut SimConnect) -> GenericResult<()> {
        simconnect.create_client_data::<ClientData>("YourControlsSim")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsSim")?;

        self.send_data_area =
            Some(simconnect.create_client_data::<ClientData>("YourControlsExternal")?);

        // Request "fake" data to be sent every simulation frame. Solely for the purpose of getting a timer every simframe
        simconnect.request_data_on_sim_object::<AircraftData>(0, 0, Period::SimFrame, false)?;

        Ok(())
    }

    fn send_message(&self, simconnect: &mut SimConnect, payload: Payloads) -> Result<()> {
        let area = self.send_data_area.as_ref().unwrap();

        for fragment_bytes in self.fragmenter.into_fragmented_message_bytes(&payload)? {
            let mut client_data = ClientData::new();
            for (index, byte) in fragment_bytes.into_iter().enumerate() {
                client_data.inner[index] = byte;
            }
            simconnect
                .set_client_data(area, &client_data)
                .map_err(|_| anyhow::anyhow!("Could not set client data!"))?;
        }

        Ok(())
    }

    fn add_datum(
        &mut self,
        simconnect: &mut SimConnect,
        datum_index: DatumKey,
        message: DatumMessage,
    ) -> Result<()> {
        let mut datum = Datum::new();

        if let Some(var_id) = message.var {
            let rc_var = self
                .vars
                .get(var_id)
                .ok_or(anyhow::anyhow!("Variable not set prior."))?
                .clone();

            datum.with_var(rc_var.clone());

            if let Some(interpolate_type) = message.interpolate {
                datum.with_interpolate(interpolate_type);
            }

            if let Some(watch_period) = message.watch_period {
                datum.with_watch_data(VariableWatcher::new(rc_var.clone(), watch_period))
            }
        }

        if let Some(mapping_message) = message.mapping {
            let mapping = match mapping_message {
                MappingType::Event => MappingType::Event,
                MappingType::Var => MappingType::Var,
                MappingType::Script(message) => MappingType::Script(MappingArgs {
                    script_id: message.script_id,
                    vars: map_ids(&self.vars, message.vars)?,
                    sets: map_ids(&self.events, message.sets)?,
                    params: message.params,
                }),
            };

            datum.with_mapping(mapping);
        }

        if let Some(condition_message) = message.conditions {
            let mut conditions = Vec::new();

            for condition_message in condition_message {
                conditions.push(Condition {
                    script_id: condition_message.script_id,
                    params: condition_message.params,
                    vars: map_ids(&self.vars, condition_message.vars)?,
                });
            }

            datum.with_conditions(conditions);
        }

        if let Some(watch_event) = message.watch_event {
            datum.with_watch_event(KeyEvent::new(simconnect, watch_event));
        }

        self.datum_manager.add_datum(datum_index, datum);

        Ok(())
    }

    fn add_datums(&mut self, simconnect: &mut SimConnect, datums: Vec<DatumMessage>) {
        println!("Adding {} datums!", datums.len());
        for (index, datum) in datums.into_iter().enumerate() {
            print!("Added success: {:?}", datum,);
            println!("{:?}", self.add_datum(simconnect, index, datum));
        }
    }

    fn add_vars(&mut self, vars: Vec<VarType>) -> Result<()> {
        println!("Adding {} vars!", vars.len());
        self.vars.clear();
        self.vars.reserve(vars.len());

        for var in vars {
            println!("{:?}", var);
            // Create generic vars from message data
            let var = match &var {
                VarType::WithUnits { name, units, index } => {
                    GenericVariable::new_var(name, units, *index)
                }
                VarType::Named { name } => GenericVariable::new_named(name),
                VarType::Calculator { get, set } => {
                    GenericVariable::new_calculator(get.clone(), set.clone())
                }
            }?;

            self.vars.push(Rc::new(var));
        }

        Ok(())
    }

    fn add_events(&mut self, events: Vec<EventMessage>) {
        println!("Adding {} events!", events.len());
        self.events.clear();
        self.events.reserve(events.len());

        for event in events {
            // Create events from message data
            let event = match event.param {
                Some(index) => EventSet::new_with_index(event.name, index, event.param_reversed),
                None => EventSet::new(event.name),
            };

            self.events.push(event.into_rc());
        }
    }

    fn set_scripts(&mut self, scripts: Vec<ScriptMessage>) -> Result<()> {
        println!("Added scripts! {:?}", scripts);
        SCRIPTING_ENGINE.with(|x| x.borrow_mut().reset());

        for script in scripts {
            let lines: Vec<&str> = script.lines.iter().map(String::as_str).collect();
            let mut add_result = None;

            SCRIPTING_ENGINE.with(|x| {
                add_result = Some(x.borrow_mut().add_script(&lines));
            });

            add_result.unwrap()?;
        }

        Ok(())
    }

    fn send_lvars(&self, simconnect: &mut SimConnect) -> Result<()> {
        let mut lvar_names = Vec::new();
        let mut id = 0;

        while let Some(lvar) = legacy::get_name_of_named_variable(id) {
            lvar_names.push(lvar);
            id += 1;
        }

        self.send_message(simconnect, Payloads::LVars { data: lvar_names })
    }

    fn reset(&mut self) {
        self.datum_manager = DatumManager::new();
        self.vars.clear();
        self.events.clear();

        SCRIPTING_ENGINE.with(|x| x.borrow_mut().reset());
    }

    fn process_client_data(
        &mut self,
        simconnect: &mut SimConnect,
        data: &ClientData,
    ) -> Result<()> {
        let payload = self.fragmenter.process_fragment_bytes(&data.inner)?;

        match payload {
            // Unused
            Payloads::VariableChange { .. }
            | Payloads::EventTriggered {}
            | Payloads::Pong
            | Payloads::LVars { .. } => {}
            // Receiving
            Payloads::Ping => self.send_message(simconnect, Payloads::Pong)?,

            Payloads::SetMappings {
                vars,
                events,
                datums,
                scripts,
            } => {
                self.add_vars(vars)?;
                self.add_events(events);
                self.set_scripts(scripts)?;
                self.add_datums(simconnect, datums);
            }

            Payloads::RequestLvarNames => self.send_lvars(simconnect)?,

            Payloads::ResetInterpolation => self.datum_manager.reset_interpolate_time(),

            Payloads::ResetAll => self.reset(),

            Payloads::SendIncomingValues { data, time } => {
                self.datum_manager.process_incoming_data(data, time)
            }
            // TODO:
            Payloads::WatchVariable {} => {}
            Payloads::WatchEvent {} => {}
            Payloads::MultiWatchVariable {} => {}
            Payloads::MultiWatchEvent {} => {}
            Payloads::ExecuteCalculator {} => {}
            Payloads::AddMapping {} => {}
        }

        Ok(())
    }

    pub fn process_simconnect_message(
        &mut self,
        simconnect: &mut SimConnect,
        message: SimConnectRecv<'_>,
    ) -> Result<()> {
        match message {
            SimConnectRecv::Null => {}
            SimConnectRecv::ClientData(e) => {
                self.process_client_data(simconnect, e.into::<ClientData>(simconnect).unwrap())?
            }
            // Triggered every simulation frame
            SimConnectRecv::SimObjectData(_) => {
                let changed = self.datum_manager.poll();
                if changed.len() > 0 {
                    self.send_message(simconnect, Payloads::VariableChange { changed })?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
