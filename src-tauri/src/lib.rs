// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
use commands::*;
use funky_lesson_core::tokio;

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
// #[tokio::main]

#[cfg(not(mobile))]
#[tokio::main]
pub async fn run() {
    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(|| {
            let _ = funky_lesson_proxy::main();
        })
        .await;
    });
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

#[cfg(mobile)]
#[tauri::mobile_entry_point]
pub async fn run() {
    // Mobile implementation

    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(|| {
            let _ = funky_lesson_proxy::main();
        })
        .await;
    });
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

// 12-22 17:58:34.870  6486  6486 E Tauri/Console: File: http://tauri.localhost/ - Line 0 - Msg: Access to fetch at 'http://localhost:3030/api/proxy/elective/user' from origin 'http://tauri.localhost' has been blocked by CORS policy: Response to preflight request doesn't pass access control check: No 'Access-Control-Allow-Origin' header is present on the requested resource. If an opaque response serves your needs, set the request's mode to 'no-cors' to fetch the resource with CORS disabled.
// 12-22 17:58:41.176  6486  6486 E FrameTracker: force finish cuj, time out: J<IME_INSETS_ANIMATION::1@0@com.funkylesson.app>
