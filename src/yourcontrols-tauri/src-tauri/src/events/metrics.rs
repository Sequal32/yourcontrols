#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]
pub struct MetricsEvent {
    pub sent_packets: f32,
    pub received_packets: f32,
    pub sent_bandwidth: f32,
    pub received_bandwidth: f32,
    pub packet_loss: f32,
    pub ping: f32,
}

impl From<laminar::Metrics> for MetricsEvent {
    fn from(metrics: laminar::Metrics) -> Self {
        Self {
            sent_packets: metrics.sent_packets,
            received_packets: metrics.received_packets,
            sent_bandwidth: metrics.sent_kbps,
            received_bandwidth: metrics.receive_kbps,
            packet_loss: metrics.packet_loss,
            ping: metrics.rtt / 2.0,
        }
    }
}
