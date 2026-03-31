#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod app;
mod cli;
mod clientmanager;
mod corrector;
mod definitions;
mod paths;
mod program;
mod simconfig;
mod sync;
mod syncdefs;
mod update;
mod util;
mod varreader;

use cli::CliWrapper;
use program::Program;
use std::{env, fs::File};

const LOG_FILENAME: &str = "log.txt";

fn main() {
    let cli: CliWrapper = CliWrapper::new();
    let is_dev_build = cfg!(debug_assertions);

    if !is_dev_build {
        // Set CWD to application directory
        let exe_path = env::current_exe();
        env::set_current_dir(exe_path.unwrap().parent().unwrap()).ok();
    }
    // Initialize logging
    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        File::create(LOG_FILENAME).unwrap(),
    )
    .ok();
    let mut program = Program::new(cli);
    program.run();
}
