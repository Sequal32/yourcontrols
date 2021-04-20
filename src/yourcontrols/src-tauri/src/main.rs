use std::time::Duration;

use program::Program;

mod aircraft;
mod program;
mod ui;

fn main() {
    let mut program = Program::setup();

    loop {
        program.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
}
