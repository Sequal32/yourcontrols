pub type SettingsState = std::sync::Mutex<Settings>;

#[derive(Default, Clone)]
pub struct Settings {
    pub username: String,
    pub aircraft: String,
    pub instructor_mode: bool,
    pub streamer_mode: bool,
}

unsafe impl Send for Settings {}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}
