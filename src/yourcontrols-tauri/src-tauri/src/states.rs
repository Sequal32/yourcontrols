use std::sync::Mutex;

pub type SimConnectorState = Mutex<SimConnectorWrapper>;

pub struct SimConnectorWrapper(pub simconnect::SimConnector);

unsafe impl Send for SimConnectorWrapper {}

impl SimConnectorWrapper {
    pub fn new() -> Self {
        Self(simconnect::SimConnector::new())
    }
}
