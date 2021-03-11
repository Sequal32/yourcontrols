use std::convert::TryInto;

use crate::util::{GenericResult, DATA_SIZE};
use msfs::sim_connect::{client_data_definition, ClientDataArea, SimConnect, SimConnectRecv};
use yourcontrols_types::{MessageFragmenter, Payloads};

/// A wrapper struct for an array of size DATA_SIZE bytes
#[client_data_definition]
struct ClientData {
    inner: [u8; DATA_SIZE],
}

/// The main driver to process and send out messages through SimConnect.
pub struct MainGauge {
    fragmenter: MessageFragmenter,
    send_data_area: Option<ClientDataArea<ClientData>>,
}

impl MainGauge {
    pub fn new() -> Self {
        Self {
            fragmenter: MessageFragmenter::new(DATA_SIZE - 16), // Leave 16 bytes free for header
            send_data_area: None,
        }
    }

    /// Creates/Setup ClientDataAreas for communication
    pub fn setup(&mut self, simconnect: &mut SimConnect) -> GenericResult<()> {
        simconnect.create_client_data::<ClientData>("YourControlsSim")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsSim");
        simconnect.subscribe_to_system_event("Frame")?;

        self.send_data_area =
            Some(simconnect.create_client_data::<ClientData>("YourControlsExternal")?);
        Ok(())
    }

    fn send_message(&self, simconnect: &mut SimConnect, payload: Payloads) {
        let bytes = rmp_serde::encode::to_vec(&payload).unwrap();
        let fragmented_messages = self.fragmenter.fragment_bytes(bytes);
        let area = self.send_data_area.as_ref().unwrap();

        for fragment in fragmented_messages {
            let fragment_bytes = rmp_serde::encode::to_vec(&fragment).unwrap();
            simconnect.set_client_data(
                area,
                &ClientData {
                    inner: fragment_bytes.try_into().unwrap(),
                },
            );
        }
    }

    fn process_client_data(&mut self, simconnect: &mut SimConnect, data: &ClientData) {
        let payload: Payloads = match rmp_serde::decode::from_slice(&data.inner) {
            Ok(p) => p,
            Err(_) => return,
        };

        println!("{:?}", payload);

        match payload {
            Payloads::SetDatums { datums } => {}
            Payloads::Ping => self.send_message(simconnect, Payloads::Pong),
            _ => {}
        }
    }

    pub fn process_simconnect_message(
        &mut self,
        simconnect: &mut SimConnect,
        message: SimConnectRecv<'_>,
    ) {
        println!("Simconnect message! {:?}", message);
        match message {
            SimConnectRecv::Null => {}
            SimConnectRecv::ClientData(e) => {
                self.process_client_data(simconnect, e.into::<ClientData>(simconnect).unwrap())
            }
            SimConnectRecv::EventFrame(_) => {}
            _ => {}
        }
    }
}
