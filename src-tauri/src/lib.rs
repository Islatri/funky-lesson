// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            initialize_client,
            login_command,
            set_batch_command,
            get_courses_command,
            enroll_courses_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
