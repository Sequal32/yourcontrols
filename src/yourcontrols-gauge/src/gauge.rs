#![cfg(any(target_arch = "wasm32"))]

use msfs::sim_connect::{
    client_data_definition, data_definition, ClientDataArea, Period, SimConnect, SimConnectRecv,
};
use std::{cell::RefCell, rc::Rc};

use crate::data::datum::{Datum, DatumManager};
use crate::data::watcher::VariableWatcher;
use crate::data::{EventSet, GenericVariable, KeyEvent, RcSettable, RcVariable, Syncable};
use crate::interpolation::Interpolation;
use crate::sync::{Condition, NumDigitSet, NumIncrement, NumSet, ToggleSwitch};
use crate::util::{GenericResult, DATA_SIZE};

use yourcontrols_types::{
    DatumMessage, MappingType, MessagePackFragmenter, Payloads, Result, SyncPermissionState,
    VarType,
};

/// A wrapper struct for an array of size DATA_SIZE bytes
#[client_data_definition]
struct ClientData {
    inner: [u8; DATA_SIZE],
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
    datum_manager: DatumManager,
    sync_permission_state: SyncPermissionState,
    send_data_area: Option<ClientDataArea<ClientData>>,
}

impl MainGauge {
    pub fn new() -> Self {
        Self {
            fragmenter: MessagePackFragmenter::new(DATA_SIZE - 16), // Leave 16 bytes free for header
            datum_manager: DatumManager::new(),
            sync_permission_state: SyncPermissionState::default(),
            send_data_area: None,
        }
    }

    /// Creates/Setup ClientDataAreas for communication
    pub fn setup(&mut self, simconnect: &mut SimConnect) -> GenericResult<()> {
        simconnect.create_client_data::<ClientData>("YourControlsSim")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsSim")?;

        self.send_data_area =
            Some(simconnect.create_client_data::<ClientData>("YourControlsExternal")?);

        // Request "fake" data to be sent every simulation frame. Solely for the purpose of getting a timer every simframe
        simconnect.request_data_on_sim_object::<AircraftData>(0, 0, Period::SimFrame, false);

        Ok(())
    }

    fn send_message(&self, simconnect: &mut SimConnect, payload: Payloads) -> Result<()> {
        let area = self.send_data_area.as_ref().unwrap();

        for fragment_bytes in self.fragmenter.into_fragmented_message_bytes(&payload)? {
            let mut client_data = [0; DATA_SIZE];

            for (place, element) in client_data.iter_mut().zip(fragment_bytes.iter()) {
                *place = *element;
            }

            simconnect.set_client_data(area, &ClientData { inner: client_data });
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
        let mut condition = None;
        let mut mapping = None;
        let mut interpolate = None;
        let mut var = None;

        if let Some(message_var) = message.var {
            let rc_var = Rc::new(RefCell::new(match &message_var {
                VarType::WithUnits { name, units, index } => {
                    GenericVariable::new_var(name, units, *index)
                }
                VarType::Named { name } => GenericVariable::new_named(name),
                VarType::Calculator { get, set } => {
                    GenericVariable::new_calculator(get.clone(), set.clone())
                }
            }?));

            watch_data = message
                .watch_period
                .map(|period| VariableWatcher::new(rc_var.clone(), period));

            condition = message.condition.map(|x| Condition {
                var: if x.use_var {
                    Some(rc_var.clone())
                } else {
                    None
                },
                equals: x.equals,
                less_than: x.less_than,
                greater_than: x.greater_than,
            });

            interpolate = message
                .interpolate
                .map(|x| Interpolation::new(x.interpolate_type, x.calculator));

            mapping = message.mapping.map(|x| match x {
                MappingType::ToggleSwitch {
                    event_name,
                    off_event_name,
                    switch_on,
                } => Box::new(ToggleSwitch {
                    var: rc_var.clone(),
                    event: EventSet::new(event_name).into_rc(),
                    off_event: off_event_name.map(|x| EventSet::new(x).into_rc()),
                    switch_on,
                }) as Box<dyn Syncable>,
                MappingType::NumSet {
                    event_name,
                    swap_event_name,
                    multiply_by,
                    add_by,
                } => Box::new(NumSet {
                    var: rc_var.clone(),
                    event: EventSet::new(event_name).into_rc(),
                    swap_event: swap_event_name.map(|x| EventSet::new(x).into_rc()),
                    multiply_by,
                    add_by,
                }) as Box<dyn Syncable>,
                MappingType::NumIncrement {
                    up_event_name,
                    down_event_name,
                    increment_amount,
                    pass_difference,
                } => Box::new(NumIncrement {
                    var: rc_var.clone(),
                    up_event: EventSet::new(up_event_name).into_rc(),
                    down_event: EventSet::new(down_event_name).into_rc(),
                    increment_amount,
                    pass_difference,
                }) as Box<dyn Syncable>,
                MappingType::NumDigitSet {
                    inc_events,
                    dec_events,
                } => Box::new(NumDigitSet {
                    var: rc_var.clone(),
                    inc_events: inc_events
                        .into_iter()
                        .map(|x| EventSet::new(x).into_rc())
                        .collect(),
                    dec_events: dec_events
                        .into_iter()
                        .map(|x| EventSet::new(x).into_rc())
                        .collect(),
                }) as Box<dyn Syncable>,
                MappingType::Var => Box::new(
                    match message_var {
                        VarType::WithUnits { name, units, index } => {
                            GenericVariable::new_var(&name, &units, index)
                        }
                        VarType::Named { name } => GenericVariable::new_named(&name),
                        VarType::Calculator { get, set } => {
                            GenericVariable::new_calculator(get, set)
                        }
                    }
                    .unwrap(),
                ) as Box<dyn Syncable>,
            });
        };

        let watch_event = message
            .watch_event
            .map(|event_name| KeyEvent::new(simconnect, event_name));

        let datum = Datum {
            var,
            watch_event,
            watch_data,
            condition,
            interpolate,
            mapping,
            sync_permission: message.sync_permission,
        };

        self.datum_manager.add_datum(datum_index, Box::new(datum));

        Ok(())
    }

    fn add_datums(&mut self, simconnect: &mut SimConnect, datums: Vec<DatumMessage>) {
        for (index, datum) in datums.into_iter().enumerate() {
            self.add_datum(simconnect, index as u32, datum);
        }
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

            Payloads::ResetInterpolation => self.datum_manager.reset_interpolate_time(),

            Payloads::ResetAll => self.datum_manager = DatumManager::new(),

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
                println!("Client data message! {:?}", e);
                self.process_client_data(simconnect, e.into::<ClientData>(simconnect).unwrap())?
            }
            // Triggered every simulation frame
            SimConnectRecv::SimObjectData(_) => {
                let changed = self.datum_manager.poll(&self.sync_permission_state);
                if changed.len() > 0 {
                    self.send_message(simconnect, Payloads::VariableChange { changed });
                }
            }
            _ => {}
        }

        Ok(())
    }
}
