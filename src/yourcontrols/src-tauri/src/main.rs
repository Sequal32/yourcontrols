use std::time::Duration;

use clap::Arg;
use program::Program;

use crate::ui::{CliUi, TauriUI};

mod aircraft;
mod clients;
mod network;
mod program;
mod simulator;
mod ui;

fn main() {
    let matches = clap::App::new("YourControls").get_matches();

    let mut program: Program<CliUi> = Program::setup();
    program
        .load_definitions("aircraft/Asobo_C172.yaml")
        .expect("did not load");

    println!("{}", program.connect_to_simulator());

    loop {
        program.poll().expect("Error occured...");
        std::thread::sleep(Duration::from_millis(10));
    }
}
