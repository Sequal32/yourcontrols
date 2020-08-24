use simconnectsdk;

enum SyncStat {
    ToggleNeedsAdjustment
}

struct ToggleSwitch {
    event_id: u32,
}

impl ToggleSwitch {
    fn sync(&self, conn: &simconnectsdk::SimConnector, from: bool, to: bool) {
        if from != to {
            conn.transmit_client_event(1, self.event_id, 0, 5, 0);
        }
    }
}