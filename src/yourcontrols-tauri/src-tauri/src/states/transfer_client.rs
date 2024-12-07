pub type TransferClientState = std::sync::Arc<std::sync::Mutex<Option<TransferClientWrapper>>>;

pub struct TransferClientWrapper(pub Box<dyn yourcontrols_net::TransferClient>);

unsafe impl Send for TransferClientWrapper {}
