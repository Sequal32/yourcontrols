pub type SimConnectorState = std::sync::Arc<std::sync::Mutex<SimConnectorWrapper>>;

#[derive(Default)]
pub struct SimConnectorWrapper(pub simconnect::SimConnector);

unsafe impl Send for SimConnectorWrapper {}
