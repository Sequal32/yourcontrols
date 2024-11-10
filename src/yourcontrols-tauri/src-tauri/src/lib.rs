use std::sync::Mutex;

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
}
mod sync;
mod syncdefs;
mod util;
mod varreader;

pub const AIRCRAFT_DEFINITIONS_PATH: &str = "F:/yourcontrols/definitions/aircraft/";

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
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri_specta_builder.invoke_handler())
        .manage(Mutex::new(states::SimConnectorWrapper::new()))
        .manage(Mutex::new(states::DefinitionsWrapper::new()))
        .manage(Mutex::new(states::Settings::new()))
        .setup(move |app| {
            tauri_specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
