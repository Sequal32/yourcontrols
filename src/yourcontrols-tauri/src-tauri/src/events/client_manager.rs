// todo: should replace with an event that always sends the whole client list, this allows the frontend to simply update the list instead of having to keep track of the list itself
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]
pub struct SetObservingEvent(pub String, pub bool);
