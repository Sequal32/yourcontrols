use std::collections::HashMap;

type ClientId = u32;

pub struct ClientInfo {
    name: String,
    in_control: bool,
    is_observer: bool,
}

pub struct Clients {
    client_map: HashMap<ClientId, ClientInfo>,
}

impl Clients {}
