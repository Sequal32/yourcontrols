#![cfg(any(target_arch = "wasm32", doc))]
mod gauge;
use gauge::MainGauge;

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

const PROGRAM_NAME: &str = "YourControlsGauge";

// Used for quick reloading via aircraft selector
#[msfs::gauge(name=YourControlsGauge)]
async fn callback(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let mut simconnect = gauge.open_simconnect(PROGRAM_NAME)?;
    let mut program_gauge = MainGauge::new();

    while let Some(message) = gauge.next_event().await {
        match message {
            msfs::MSFSEvent::PostInstall => program_gauge.setup(&mut simconnect)?,
            msfs::MSFSEvent::SimConnect(m) => {
                program_gauge.process_simconnect_message(&mut simconnect, m)
            }
            _ => {}
        }
    }

    Ok(())
}

#[msfs::standalone_module]
async fn module(mut module: msfs::StandaloneModule) -> Result<(), Box<dyn std::error::Error>> {
    let mut simconnect = module.open_simconnect(PROGRAM_NAME)?;
    let mut program_gauge = MainGauge::new();

    while let Some(message) = module.next_event().await {
        program_gauge.process_simconnect_message(&mut simconnect, message)
    }

    Ok(())
}
