pub mod data;
pub mod interpolation;
pub mod sync;
pub mod util;

use util::GenericResult;
#[cfg(any(target_arch = "wasm32", doc))]
pub mod gauge;
#[cfg(any(target_arch = "wasm32", doc))]
use gauge::MainGauge;

const PROGRAM_NAME: &str = "YourControlsGauge";

/// The entry point of the gauge. This is called when the module is loaded in.
#[msfs::standalone_module]
#[cfg(any(target_arch = "wasm32", doc))]
async fn module(mut module: msfs::StandaloneModule) -> GenericResult<()> {
    let mut simconnect = module.open_simconnect(PROGRAM_NAME)?;
    let mut program_gauge = MainGauge::new();

    program_gauge.setup(&mut simconnect);

    while let Some(message) = module.next_event().await {
        program_gauge.process_simconnect_message(&mut simconnect, message);
    }

    Ok(())
}
