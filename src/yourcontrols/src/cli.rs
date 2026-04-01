use clap::{value_parser, Parser, ValueEnum};

use crate::app::ConnectionMethod;
use crate::simconfig::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[cfg(debug_assertions)]
    #[arg(long, help = "Skip SimConnect connection (debug only).")]
    skip_sim_connect: bool,

    #[arg(long, help = "Definition file name to load at startup.")]
    definition_file: Option<String>,

    #[arg(long, help = "Start a server immediately after startup.")]
    start_server: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = CliConnectionMethod::CloudServer,
        help = "Connection method for auto-start server."
    )]
    connection_method: CliConnectionMethod,

    #[arg(long, help = "Override connection timeout (seconds).")]
    conn_timeout: Option<u64>,

    #[arg(long, help = "Override network port.")]
    port: Option<u16>,

    #[arg(long, help = "Override IP address.")]
    ip: Option<String>,

    #[arg(long, help = "Override user name.")]
    name: Option<String>,

    #[arg(
        long,
        value_parser = value_parser!(bool),
        num_args = 1,
        help = "Override UI dark theme (true/false)."
    )]
    ui_dark_theme: Option<bool>,

    #[arg(
        long,
        value_parser = value_parser!(bool),
        num_args = 1,
        help = "Override streamer mode (true/false)."
    )]
    streamer_mode: Option<bool>,

    #[arg(
        long,
        value_parser = value_parser!(bool),
        num_args = 1,
        help = "Override instructor mode (true/false)."
    )]
    instructor_mode: Option<bool>,

    #[arg(long, help = "Enable emulator UI and controls.")]
    emulator: bool,

    #[arg(long, help = "Log to the terminal in addition to log.txt.")]
    log_console: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum CliConnectionMethod {
    Direct,
    Relay,
    CloudServer,
}

pub struct CliWrapper {
    cli: Cli,
}

impl CliWrapper {
    pub fn new() -> Self {
        Self { cli: Cli::parse() }
    }

    pub fn skip_sim_connect(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            self.cli.skip_sim_connect
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }

    pub fn definition_file(&self) -> Option<&str> {
        self.cli.definition_file.as_deref()
    }

    pub fn start_server(&self) -> bool {
        self.cli.start_server
    }

    pub fn emulator_enabled(&self) -> bool {
        self.cli.emulator
    }

    pub fn log_to_console(&self) -> bool {
        self.cli.log_console
    }

    pub fn connection_method(&self) -> ConnectionMethod {
        match self.cli.connection_method {
            CliConnectionMethod::Direct => ConnectionMethod::Direct,
            CliConnectionMethod::Relay => ConnectionMethod::Relay,
            CliConnectionMethod::CloudServer => ConnectionMethod::CloudServer,
        }
    }

    pub fn apply_config_overrides(&self, config: &mut Config) {
        if let Some(conn_timeout) = self.cli.conn_timeout {
            config.conn_timeout = conn_timeout;
        }
        if let Some(port) = self.cli.port {
            config.port = port;
        }
        if let Some(ip) = self.cli.ip.as_ref() {
            config.ip = ip.clone();
        }
        if let Some(name) = self.cli.name.as_ref() {
            config.name = name.clone();
        }
        if let Some(ui_dark_theme) = self.cli.ui_dark_theme {
            config.ui_dark_theme = ui_dark_theme;
        }
        if let Some(streamer_mode) = self.cli.streamer_mode {
            config.streamer_mode = streamer_mode;
        }
        if let Some(instructor_mode) = self.cli.instructor_mode {
            config.instructor_mode = instructor_mode;
        }
    }
}
