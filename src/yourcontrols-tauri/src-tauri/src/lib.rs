use tauri_plugin_log::fern;

mod commands;
mod corrector;
mod definitions;
mod states {
    pub mod definitions;
    pub use definitions::*;

    pub mod sim_connector;
    pub use sim_connector::*;

    pub mod settings;
    pub use settings::*;

    pub mod transfer_client;
    pub use transfer_client::*;
}
mod sync;
mod syncdefs;
mod util;
mod varreader;

pub const AIRCRAFT_DEFINITIONS_PATH: &str = "F:/yourcontrols/definitions/aircraft/";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    String(std::string::String),
    #[error("IO error occurred, check the logs for more information!")]
    Io(#[from] std::io::Error),
    #[error("An error occurred, check the logs for more information!")]
    Yourcontrols(#[from] yourcontrols_types::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let log_error = match self {
            Error::String(x) => x.clone(),
            Error::Io(x) => x.to_string(),
            Error::Yourcontrols(x) => x.to_string(),
            Error::Anyhow(x) => x.to_string(),
        };
        log::error!("{log_error}");

        serializer.serialize_str(&self.to_string())
    }
}

impl specta::Type for Error {
    fn inline(_: &mut specta::TypeMap, _: specta::Generics) -> specta::datatype::DataType {
        specta::datatype::DataType::Primitive(specta::datatype::PrimitiveType::String)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tauri_specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        .commands(tauri_specta::collect_commands![
            commands::get_aircraft_configs,
            commands::save_settings,
            commands::start_server,
        ]);

    #[cfg(debug_assertions)]
    {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("src")
            .join("types")
            .join("bindings.ts");

        tauri_specta_builder
            .export(specta_typescript::Typescript::default(), path)
            .expect("Failed to export typescript bindings");
    }

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .with_colors(fern::colors::ColoredLevelConfig {
                    error: fern::colors::Color::Red,
                    warn: fern::colors::Color::Yellow,
                    info: fern::colors::Color::Green,
                    debug: fern::colors::Color::Magenta,
                    trace: fern::colors::Color::Blue,
                })
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .build(),
        )
        .invoke_handler(tauri_specta_builder.invoke_handler())
        .manage(states::SimConnectorState::default())
        .manage(states::DefinitionsState::default())
        .manage(states::SettingsState::default())
        .manage(states::TransferClientState::default())
        .setup(move |app| {
            tauri_specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
