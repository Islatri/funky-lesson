// commands.rs
use tauri::State;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use funky_lesson_core::app::{enroll_courses, get_courses, set_batch, login,BatchInfo};
use funky_lesson_core::request::create_client;
use funky_lesson_core::Client;

// 状态管理结构体
pub struct AppState {
    client: Mutex<Option<Client>>,
    token: Mutex<String>,
    batch_id: Mutex<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
    batch_list: Vec<BatchInfo>,
}

#[derive(Serialize)]
pub struct CourseInfo {
    selected_courses: Vec<String>,
    favorite_courses: Vec<String>,
}

// 初始化客户端
#[tauri::command]
pub async fn initialize_client(state: State<'_, AppState>) -> Result<(), String> {
    let client = create_client().await.map_err(|e| e.to_string())?;
    *state.client.lock().unwrap() = Some(client);
    Ok(())
}

// 登录命令
#[tauri::command]
pub async fn login_command(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = state.client.lock().unwrap();
    let client = client.as_ref().ok_or("Client not initialized")?;
    
    let (token, batch_list) = login(client, &username, &password)
        .await
        .map_err(|e| e.to_string())?;
    
    *state.token.lock().unwrap() = token.clone();
    
    Ok(LoginResponse {
        token,
        batch_list,
    })
}

// 设置批次
#[tauri::command]
pub async fn set_batch_command(
    state: State<'_, AppState>,
    batch_idx: usize,
    batch_list: Vec<BatchInfo>,
) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    let client = client.as_ref().ok_or("Client not initialized")?;
    let token = state.token.lock().unwrap();
    
    let batch_codes: Vec<String> = batch_list.iter().map(|b| b.code.clone()).collect();
    let batch_id = set_batch(client, &token, &batch_codes, batch_idx)
        .await
        .map_err(|e| e.to_string())?;
        
    *state.batch_id.lock().unwrap() = batch_id;
    Ok(())
}

// 获取课程信息
#[tauri::command]
pub async fn get_courses_command(
    state: State<'_, AppState>,
) -> Result<CourseInfo, String> {
    let client = state.client.lock().unwrap();
    let client = client.as_ref().ok_or("Client not initialized")?;
    let token = state.token.lock().unwrap();
    let batch_id = state.batch_id.lock().unwrap();
    
    let (selected_courses, favorite_courses) = get_courses(client, &token, &batch_id)
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(CourseInfo {
        selected_courses,
        favorite_courses,
    })
}

// 选课命令
#[tauri::command]
pub async fn enroll_courses_command(
    state: State<'_, AppState>,
    favorite_courses: Vec<String>,
) -> Result<(), String> {
    let client = state.client.lock().unwrap();
    let client = client.as_ref().ok_or("Client not initialized")?;
    let token = state.token.lock().unwrap();
    let batch_id = state.batch_id.lock().unwrap();
    
    enroll_courses(client, &token, &batch_id, &favorite_courses, true)
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(())
}
