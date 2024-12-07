use client_manager::ClientManager;
use tauri::Manager;
use tauri_plugin_log::fern;

mod commands;
mod corrector;
mod definitions;
mod states {
    pub mod client_manager;
    pub use client_manager::*;

    pub mod definitions;
    pub use definitions::*;

    pub mod settings;
    pub use settings::*;

    pub mod sim_connector;
    pub use sim_connector::*;

    pub mod transfer_client;
    pub use transfer_client::*;
}
mod events {
    pub mod client_fail;
    pub use client_fail::*;

    pub mod client_manager;

    pub mod control;

    pub mod metrics;
    pub use metrics::*;

    pub mod server;
}
mod client_manager;
mod simconfig;
mod sync;
mod syncdefs;
mod update;
mod util;
mod varreader;

// TODO: move to a config file
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
    #[error(transparent)]
    LocalIpAddress(#[from] local_ip_address::Error),
    #[error(transparent)]
    IgdNextSearch(#[from] igd_next::SearchError),
    #[error(transparent)]
    IgdNextExternalIp(#[from] igd_next::GetExternalIpError),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // automatically log errors that get send to the frontend
        let log_error = match self {
            Error::String(x) => x.clone(),
            Error::Io(x) => x.to_string(),
            Error::Yourcontrols(x) => x.to_string(),
            Error::Anyhow(x) => x.to_string(),
            Error::LocalIpAddress(x) => x.to_string(),
            Error::IgdNextSearch(x) => x.to_string(),
            Error::IgdNextExternalIp(x) => x.to_string(),
        };
        log::error!(target: "tauri_backend", "{log_error}");

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
            commands::disconnect,
            commands::transfer_control,
            commands::set_observer,
            commands::go_observer,
            commands::force_take_control,
            commands::get_public_ip,
        ])
        .events(tauri_specta::collect_events![
            events::ClientFailEvent,
            events::server::ServerFailEvent,
            events::server::ServerAttemptEvent,
            events::server::ServerStartedEvent,
            events::control::GainControlEvent,
            events::control::LoseControlEvent,
            events::control::SetInControlEvent,
            events::MetricsEvent,
            events::client_manager::SetObservingEvent,
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
        .plugin(tauri_plugin_clipboard_manager::init())
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
            let client_manager = ClientManager::new(app.handle().clone());
            let state = states::ClientManagerState::new(std::sync::Mutex::new(client_manager));
            app.manage(state);

            tauri_specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running application");
}
