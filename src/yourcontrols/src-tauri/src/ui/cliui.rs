use super::Ui;

pub struct CliUi {}

impl CliUi {
    pub fn new() -> Self {
        Self {}
    }
}

impl Ui for CliUi {
    fn run() -> Self {
        todo!()
    }

    fn send_message(&mut self, event: super::cmd::UiEvents) -> anyhow::Result<()> {
        todo!()
    }

    fn next_event(&mut self) -> Option<super::cmd::UiEvents> {
        todo!()
    }
}
