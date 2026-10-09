mod commands;
mod crypto;
mod errors;
mod format;
mod session;
mod settings;

use session::Session;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Session::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_document,
            commands::open_document,
            commands::save_document,
            commands::change_password,
            commands::lock,
            commands::probe_file,
            commands::read_file,
            commands::load_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
