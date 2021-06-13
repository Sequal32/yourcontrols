pub mod cmd;
mod tauriui;
mod util;
mod cliui;

pub use tauriui::TauriUI;

use self::cmd::*;
use anyhow::Result;

pub trait Ui {
    fn run() -> Self;
    fn send_message(&mut self, event: UiEvents) -> Result<()>;
    fn next_event(&mut self) -> Option<UiEvents>;
}
