use simconnectsdk;

pub trait Syncable<T> {
    fn sync(&self, conn: &simconnectsdk::SimConnector, from: T, to: T);
}

pub struct ToggleSwitch {
    event_id: u32,
}

impl ToggleSwitch {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id
        }
    }
}

impl Syncable<bool> for ToggleSwitch {
    fn sync(&self, conn: &simconnectsdk::SimConnector, from: bool, to: bool) {
        if from != to {
            conn.transmit_client_event(1, self.event_id, 0, 5, 0);
        }
    }
}

pub struct ToggleSwitchSet {
    event_id: u32
}

impl ToggleSwitchSet {
    pub fn new(event_id: u32) -> Self {
        return Self {
            event_id: event_id
        }
    }
}

impl Syncable<bool> for ToggleSwitchSet {
    fn sync(&self, conn: &simconnectsdk::SimConnector, _: bool, to: bool) {
        match to {
            true => conn.transmit_client_event(1, self.event_id, 1, 5, 0),
            false => conn.transmit_client_event(1, self.event_id, 0, 5, 0)
        };
    }
}