// commands.rs
use funky_lesson_core::app::request::gui::{enroll_courses, login};
use funky_lesson_core::app::request::{get_courses,set_batch,get_captcha_inner};
use funky_lesson_core::model::structs::{BatchInfo, CourseInfo, EnrollmentStatus};
use funky_lesson_core::client::request::create_client;
use funky_lesson_core::tokio;
use funky_lesson_core::Client;
use funky_lesson_core::TokioMutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub client: TokioMutex<Option<Client>>,
    pub token: TokioMutex<String>,
    pub batch_id: TokioMutex<String>,
    pub is_running: TokioMutex<bool>,
    pub status: TokioMutex<EnrollmentStatus>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
    batch_list: Vec<BatchInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct CaptchaResponse {
    uuid: String,
    image_base64: String,
}

#[tauri::command]
pub async fn initialize_client(state: State<'_, AppState>) -> Result<(), String> {
    let client = create_client().await.map_err(|e| e.to_string())?;
    let mut client_lock = state.client.lock().await;
    *client_lock = Some(client);
    Ok(())
}

#[tauri::command]
pub async fn get_captcha(state: State<'_, AppState>) -> Result<CaptchaResponse, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Client not initialized")?;

    let (uuid, image_base64) = get_captcha_inner(client).await.map_err(|e| e.to_string())?;
    // println!("uuid: {}", uuid);
    // println!("image_base64: {}", image_base64);
    Ok(CaptchaResponse { uuid, image_base64 })
}

#[tauri::command]
pub async fn login_command(
    state: State<'_, AppState>,
    username: String,
    password: String,
    captcha: String,
    uuid: String,
) -> Result<LoginResponse, String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Client not initialized")?;

    let (token, batch_list) = login(client, &username, &password, &captcha, &uuid)
        .await
        .map_err(|e| e.to_string())?;

    let mut token_lock = state.token.lock().await;
    *token_lock = token.clone();

    Ok(LoginResponse { token, batch_list })
}

#[tauri::command]
pub async fn enroll_courses_command(
    state: State<'_, AppState>,
    favorite_courses: Vec<CourseInfo>,
) -> Result<EnrollmentStatus, String> {
    // 先检查是否在运行
    {
        let is_running = state.is_running.lock().await;
        if *is_running {
            return Ok(state.status.lock().await.clone());
        }
    }

    // 获取client
    let client = {
        let client_lock = state.client.lock().await;
        client_lock
            .as_ref()
            .ok_or("Client not initialized")?
            .clone()
    };

    let token_lock = state.token.lock().await.clone();

    let batch_id_lock = state.batch_id.lock().await.clone();
    let mut is_running = state.is_running.lock().await;

    // 如果已经在运行，返回当前状态
    if *is_running {
        let status = state.status.lock().await;
        return Ok(status.clone());
    }

    // 设置运行状态
    *is_running = true;

    // 创建状态追踪器
    let status = Arc::new(TokioMutex::new(EnrollmentStatus::default()));
    let should_continue = Arc::new(TokioMutex::new(true));

    let status_clone = Arc::clone(&status);

    // 更新state中的status
    *state.status.lock().await = status.lock().await.clone();

    // 启动选课进程
    tokio::spawn(async move {
        let _ = enroll_courses(
            &client,
            &token_lock,
            &batch_id_lock,
            &favorite_courses,
            true,
            status_clone,
            should_continue,
        )
        .await;
    });

    // 返回初始状态
    Ok(state.status.lock().await.clone())
}

#[tauri::command]
pub async fn stop_enrollment(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_running = state.is_running.lock().await;
    *is_running = false;
    Ok(())
}

// ... 其他现有的命令保持不变 ...
#[tauri::command]
pub async fn set_batch_command(
    state: State<'_, AppState>,
    batch_idx: usize,
    batch_list: Vec<BatchInfo>,
) -> Result<(), String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Client not initialized")?;
    let token_lock = state.token.lock().await;

    let batch_id = set_batch(client, &token_lock, &batch_list, batch_idx)
        .await
        .map_err(|e| e.to_string())?;

    let mut batch_id_lock = state.batch_id.lock().await;
    *batch_id_lock = batch_id;
    Ok(())
}

#[tauri::command]
pub async fn get_courses_command(
    state: State<'_, AppState>,
) -> Result<(Vec<CourseInfo>, Vec<CourseInfo>), String> {
    let client_lock = state.client.lock().await;
    let client = client_lock.as_ref().ok_or("Client not initialized")?;
    let token_lock = state.token.lock().await;
    let batch_id_lock = state.batch_id.lock().await;

    let (selected_courses, favorite_courses) = get_courses(client, &token_lock, &batch_id_lock)
        .await
        .map_err(|e| e.to_string())?;

    Ok((selected_courses, favorite_courses))
}
