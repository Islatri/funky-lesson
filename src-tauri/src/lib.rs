// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

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
        .invoke_handler(tauri::generate_handler![])
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
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
