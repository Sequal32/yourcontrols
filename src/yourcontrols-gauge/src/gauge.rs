use msfs::sim_connect::{client_data_definition, ClientDataArea, SimConnect, SimConnectRecv};

use crate::GenericResult;

#[client_data_definition]
struct ClientData {
    inner: [u8; 8192],
}

#[derive(serde::Deserialize, Debug)]
enum Payloads {
    Test,
    Hello {
        hi: bool,
        no: String,
        something: f64,
    },
}

pub struct MainGauge {}

impl MainGauge {
    pub fn new() -> Self {
        Self {}
    }

    pub fn setup(&mut self, simconnect: &mut SimConnect) -> GenericResult<()> {
        simconnect.create_client_data::<ClientData>("YourControlsSim")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsSim");
        simconnect.create_client_data::<ClientData>("YourControlsExternal")?;
        simconnect.request_client_data::<ClientData>(0, "YourControlsExternal");
        Ok(())
    }

    fn process_client_data(&mut self, data: &ClientData) {
        let payload: Payloads = match rmp_serde::decode::from_slice(&data.inner) {
            Ok(p) => p,
            Err(_) => return,
        };

        println!("{:?}", payload);
    }

    pub fn process_simconnect_message(
        &mut self,
        simconnect: &mut SimConnect,
        message: SimConnectRecv<'_>,
    ) {
        println!("Simconnect message! {:?}", message);
        match message {
            SimConnectRecv::Null => {}
            SimConnectRecv::Exception(_) => {}
            SimConnectRecv::Open(_) => {}
            SimConnectRecv::Quit(_) => {}
            SimConnectRecv::Event(_) => {}
            SimConnectRecv::SimObjectData(_) => {}
            SimConnectRecv::ClientData(e) => {
                self.process_client_data(e.into::<ClientData>(simconnect).unwrap())
            }
        }
    }
}
