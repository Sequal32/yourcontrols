use simconnect::SimConnector;
use std::ffi::c_void;
use yourcontrols_types::{Error, MessagePackFragmenter, Payloads, Result};

const SEND_AREA_ID: u32 = 0;
const RECEIVE_AREA_ID: u32 = 1;
const DATA_SIZE: usize = 8192;

pub struct Simulator {
    connection: SimConnector,
    fragmenter: MessagePackFragmenter,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            connection: SimConnector::new(),
            fragmenter: MessagePackFragmenter::new(DATA_SIZE - 16),
        }
    }

    /// Attempts to open a connection with the simulator, returns if it was successful
    pub fn connect(&mut self, name: &str) -> bool {
        let mut ok = self.connection.connect(name);

        ok &= self
            .connection
            .map_client_data_name_to_id("YourControlsSim", SEND_AREA_ID);
        ok &= self
            .connection
            .map_client_data_name_to_id("YourControlsExternal", RECEIVE_AREA_ID);
        ok &= self
            .connection
            .add_to_client_data_definition(SEND_AREA_ID, 0, 8192, 0.0, 0);
        ok &= self
            .connection
            .add_to_client_data_definition(RECEIVE_AREA_ID, 0, 8192, 0.0, 0);
        ok &= self.connection.request_client_data(
            RECEIVE_AREA_ID,
            0,
            RECEIVE_AREA_ID,
            simconnect::SIMCONNECT_CLIENT_DATA_PERIOD_SIMCONNECT_CLIENT_DATA_PERIOD_ON_SET,
            0,
            0,
            0,
            0,
        );

        ok
    }

    pub fn send_message(&self, message: Payloads) {
        let fragments = self
            .fragmenter
            .into_fragmented_message_bytes(&message)
            .unwrap();

        for mut fragment in fragments {
            println!("{}", fragment.len());
            self.connection.set_client_data(
                SEND_AREA_ID,
                SEND_AREA_ID,
                0,
                0,
                8192,
                fragment.as_mut_slice().as_mut_ptr() as *mut c_void,
            );
        }
    }

    pub fn poll(&mut self) -> Result<Payloads> {
        match self.connection.get_next_message() {
            Ok(simconnect::DispatchResult::ClientData(data)) => {
                let data_pointer = &data._base.dwData as *const u32 as *const u8;
                let data_bytes = unsafe { std::slice::from_raw_parts(data_pointer, DATA_SIZE) };

                return Ok(self.fragmenter.process_fragment_bytes(data_bytes)?);
            }
            _ => return Err(Error::None),
        }
    }
}
