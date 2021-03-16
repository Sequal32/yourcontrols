use std::{cell::RefCell, rc::Rc};

use crate::{
    data::{
        datum::{Datum, DatumManager},
        watcher::VariableWatcher,
        EventSet, GenericVariable, KeyEvent, RcSettable, RcVariable, Syncable,
    },
    interpolation::Interpolation,
    sync::{Condition, NumDigitSet, NumIncrement, NumSet, ToggleSwitch},
    util::{GenericResult, DATA_SIZE},
};
use msfs::sim_connect::{client_data_definition, ClientDataArea, SimConnect, SimConnectRecv};
use yourcontrols_types::{
    DatumMessage, MappingType, MessagePackFragmenter, Payloads, Result, VarType,
};

/// A wrapper struct for an array of size DATA_SIZE bytes
#[client_data_definition]
struct ClientData {
    inner: [u8; DATA_SIZE],
}

/// The main driver to process and send out messages through SimConnect.
pub struct MainGauge {
    fragmenter: MessagePackFragmenter,
    datum_manager: DatumManager,
    send_data_area: Option<ClientDataArea<ClientData>>,
}

impl MainGauge {
    pub fn new() -> Self {
        Self {
            fragmenter: MessagePackFragmenter::new(DATA_SIZE - 16), // Leave 16 bytes free for header
            datum_manager: DatumManager::new(),
            send_data_area: None,
        }
    }

    /// Creates/Setup ClientDataAreas for communication
    pub fn setup(&mut self, simconnect: &mut SimConnect) -> GenericResult<()> {
        simconnect.create_client_data::<ClientData>("YourControlsSim")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsSim")?;
        simconnect.subscribe_to_system_event("Frame")?;

        self.send_data_area =
            Some(simconnect.create_client_data::<ClientData>("YourControlsExternal")?);
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

    fn add_datums(&mut self, simconnect: &mut SimConnect, message: DatumMessage) -> Result<()> {
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

        self.datum_manager.add_datum(
            0,
            Box::new(Datum {
                var,
                watch_event,
                watch_data,
                condition,
                interpolate,
                mapping,
                sync_permission: message.sync_permission,
            }),
        );

        Ok(())
    }

    fn process_client_data(
        &mut self,
        simconnect: &mut SimConnect,
        data: &ClientData,
    ) -> Result<()> {
        let payload = self.fragmenter.process_fragment_bytes(&data.inner)?;

        match payload {
            Payloads::Ping => self.send_message(simconnect, Payloads::Pong)?,

            Payloads::SetDatums { datums } => {}
            Payloads::WatchVariable {} => {}
            Payloads::WatchEvent {} => {}
            Payloads::MultiWatchVariable {} => {}
            Payloads::MultiWatchEvent {} => {}
            Payloads::ExecuteCalculator {} => {}
            Payloads::AddMapping {} => {}
            Payloads::SendIncomingValues {} => {}
            Payloads::QueueInterpolationData {} => {}
            Payloads::SetInterpolationData {} => {}
            Payloads::StopInterpolation {} => {}
            Payloads::ResetInterpolation {} => {}
            Payloads::ResetAll => {}
            Payloads::VariableChange {} => {}
            Payloads::EventTriggered {} => {}
            Payloads::Pong => {}
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
            SimConnectRecv::EventFrame(_) => {}
            _ => {}
        }

        Ok(())
    }
}
