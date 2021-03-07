mod data;
mod diff;
mod fragment;
mod gauge;
mod interpolation;
mod sync;
mod util;

use gauge::MainGauge;
use util::GenericResult;

const PROGRAM_NAME: &str = "YourControlsGauge";

#[msfs::standalone_module]
async fn module(mut module: msfs::StandaloneModule) -> GenericResult<()> {
    let mut simconnect = module.open_simconnect(PROGRAM_NAME)?;
    let mut program_gauge = MainGauge::new();

    program_gauge.setup(&mut simconnect);

    while let Some(message) = module.next_event().await {
        program_gauge.process_simconnect_message(&mut simconnect, message)
    }

    Ok(())
}
