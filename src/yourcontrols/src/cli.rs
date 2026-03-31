use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[cfg(debug_assertions)]
    #[arg(long, help = "Skip SimConnect connection (debug only).")]
    skip_sim_connect: bool,
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
}
