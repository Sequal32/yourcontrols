

use crate::util::{GenericResult, DATA_SIZE};
use msfs::sim_connect::{client_data_definition, ClientDataArea, SimConnect, SimConnectRecv};
use yourcontrols_types::{MessagePackFragmenter, Payloads, Result};

/// A wrapper struct for an array of size DATA_SIZE bytes
#[client_data_definition]
struct ClientData {
    inner: [u8; DATA_SIZE],
}

/// The main driver to process and send out messages through SimConnect.
pub struct MainGauge {
    fragmenter: MessagePackFragmenter,
    send_data_area: Option<ClientDataArea<ClientData>>,
}

impl MainGauge {
    pub fn new() -> Self {
        Self {
            fragmenter: MessagePackFragmenter::new(DATA_SIZE - 16), // Leave 16 bytes free for header
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

    fn process_client_data(
        &mut self,
        simconnect: &mut SimConnect,
        data: &ClientData,
    ) -> Result<()> {
        let payload = self.fragmenter.process_fragment_bytes(&data.inner)?;

        match payload {
            Payloads::Ping => self.send_message(simconnect, Payloads::Pong)?,
            _ => {}
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
