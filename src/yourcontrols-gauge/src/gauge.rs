#![cfg(any(target_arch = "wasm32"))]

use msfs::sim_connect::{
    client_data_definition, data_definition, ClientDataArea, Period, SimConnect, SimConnectRecv,
};

use std::rc::Rc;

use crate::data::datum::{Datum, DatumManager, MappingArgs};
use crate::data::watcher::VariableWatcher;
use crate::data::{EventSet, GenericVariable, KeyEvent, RcSettable, RcVariable};
use crate::interpolation::Interpolation;
use crate::sync::SCRIPTING_ENGINE;
use crate::util::map_ids;
use crate::util::{GenericResult, DATA_SIZE};

use yourcontrols_types::{
    DatumMessage, Error, EventMessage, MappingType, MessagePackFragmenter, Payloads, Result,
    ScriptMessage, SyncPermissionState, VarType,
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
    sync_permission_state: SyncPermissionState,
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
            sync_permission_state: SyncPermissionState::default(),
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
                .map_err(|_| Error::ClientDataSendError)?;
        }

        Ok(())
    }

    fn add_datum(
        &mut self,
        simconnect: &mut SimConnect,
        datum_index: u32,
        message: DatumMessage,
    ) -> Result<()> {
        let mut watch_data = None;
        let mut mapping = None;
        let mut interpolate = None;
        let mut var = None;
        let mut condition = None; // TODO: implement

        if let Some(var_id) = message.var {
            let rc_var = self.vars.get(var_id).ok_or(Error::None)?.clone();

            interpolate = message
                .interpolate
                .map(|x| Interpolation::new(x.interpolate_type, x.calculator));
            watch_data = message
                .watch_period
                .map(|period| VariableWatcher::new(rc_var.clone(), period));

            var = Some(rc_var)
        }

        if let Some(mapping_message) = message.mapping {
            mapping = Some(match mapping_message {
                MappingType::Event => MappingType::Event,
                MappingType::Var => MappingType::Var,
                MappingType::Script(message) => MappingType::Script(MappingArgs {
                    script_id: message.script_id,
                    vars: map_ids(&self.vars, message.vars)?,
                    sets: map_ids(&self.events, message.sets)?,
                    params: message.params,
                }),
            });
        }

        let watch_event = message.watch_event.map(|x| KeyEvent::new(simconnect, x));

        let datum = Datum {
            var,
            watch_event,
            watch_data,
            condition,
            interpolate,
            mapping,
            sync_permission: message.sync_permission,
        };

        self.datum_manager.add_datum(datum_index, datum);

        Ok(())
    }

    fn add_datums(&mut self, simconnect: &mut SimConnect, datums: Vec<DatumMessage>) {
        println!("Adding {} datums!", datums.len());
        for (index, datum) in datums.into_iter().enumerate() {
            print!("Added success: {:?}", datum,);
            println!("{:?}", self.add_datum(simconnect, index as u32, datum));
        }
    }

    fn add_vars(&mut self, datums: Vec<VarType>) -> Result<()> {
        println!("Adding {} vars!", datums.len());
        self.vars.clear();
        self.vars.reserve(datums.len());

        for var in datums {
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

    fn add_events(&mut self, datums: Vec<EventMessage>) {
        println!("Adding {} events!", datums.len());
        self.events.clear();
        self.events.reserve(datums.len());

        for event in datums {
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
            Payloads::VariableChange { .. } | Payloads::EventTriggered {} | Payloads::Pong => {}
            // Receiving
            Payloads::Ping => self.send_message(simconnect, Payloads::Pong)?,

            Payloads::SetDatums { datums } => self.add_datums(simconnect, datums),

            Payloads::SetVars { vars } => self.add_vars(vars)?,

            Payloads::SetEvents { events } => self.add_events(events),

            Payloads::SetScripts { scripts } => self.set_scripts(scripts)?,

            Payloads::ResetInterpolation => self.datum_manager.reset_interpolate_time(),

            Payloads::ResetAll => self.reset(),

            Payloads::UpdateSyncPermission { new } => self.sync_permission_state = new,

            Payloads::SendIncomingValues { data, time } => self
                .datum_manager
                .process_incoming_data(data, time, &self.sync_permission_state),
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
                let changed = self.datum_manager.poll(&self.sync_permission_state);
                if changed.len() > 0 {
                    println!("CHANGED {}", changed.len());
                    self.send_message(simconnect, Payloads::VariableChange { changed });
                }
            }
            _ => {}
        }

        Ok(())
    }
}
