// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            client: TokioMutex::new(None),
            token: TokioMutex::new(String::new()),
            batch_id: TokioMutex::new(String::new()),
            is_running: TokioMutex::new(false),
            status: TokioMutex::new(EnrollmentStatus::default()),
        })
        .invoke_handler(tauri::generate_handler![
            get_captcha,
            initialize_client,
            login_command,
            set_batch_command,
            get_courses_command,
            stop_enrollment,
            enroll_courses_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
