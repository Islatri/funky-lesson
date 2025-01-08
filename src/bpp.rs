use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use funky_lesson_core::{
    crypto,
    request,
    error::{Result, ErrorKind}
};

// 数据模型保持不变
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatchInfo {
    pub code: String,
    pub name: String,
    #[serde(rename = "beginTime")]
    pub begin_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseInfo {
    pub SKJS: String,     // 教师名
    pub KCM: String,      // 课程名
    pub JXBID: String,    // 教学班ID
    #[serde(rename = "teachingClassType")]
    pub teaching_class_type: Option<String>,
    #[serde(default, rename = "secretVal")]
    pub secret_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnrollmentStatus {
    pub total_requests: u32,
    pub course_statuses: Vec<String>,
    pub is_running: bool,
}

// Leptos资源和信号
#[derive(Clone)]
pub struct AppState {
    pub token: RwSignal<Option<String>>,
    pub batch_id: RwSignal<Option<String>>,
    pub batch_list: RwSignal<Vec<BatchInfo>>,
    pub selected_courses: RwSignal<Vec<CourseInfo>>,
    pub favorite_courses: RwSignal<Vec<CourseInfo>>,
    pub enrollment_status: RwSignal<EnrollmentStatus>,
    pub should_continue: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            token: RwSignal::new(None),
            batch_id: RwSignal::new(None),
            batch_list: RwSignal::new(Vec::new()),
            selected_courses: RwSignal::new(Vec::new()),
            favorite_courses: RwSignal::new(Vec::new()),
            enrollment_status: RwSignal::new(EnrollmentStatus::default()),
            should_continue: RwSignal::new(false),
        }
    }
}

// 登录函数
pub async fn login(
    username: &str,
    password: &str,
    captcha: &str,
    uuid: &str,
    app_state: &AppState,
) -> Result<()> {
    // 初始化
    request::create_client().await?;
    
    // 获取AES密钥
    let aes_key = request::get_aes_key_proxy().await?;
    
    // 加密密码并登录
    let encrypted_password = crypto::encrypt_password(password, &aes_key)?;
    let login_resp = request::send_login_request_proxy(
        username,
        &encrypted_password,
        captcha,
        uuid
    ).await?;

    if login_resp["code"] == 200 && login_resp["msg"] == "登录成功" {
        let token = login_resp["data"]["token"]
            .as_str()
            .ok_or_else(|| ErrorKind::ParseError("Invalid token".to_string()))?
            .to_string();
            
        let batch_list: Vec<BatchInfo> = serde_json::from_value(
            login_resp["data"]["student"]["electiveBatchList"].clone()
        )?;

        // 更新状态
        app_state.token.set(Some(token));
        app_state.batch_list.set(batch_list);
        Ok(())
    } else {
        Err(ErrorKind::ParseError(login_resp["msg"].to_string()).into())
    }
}

// 获取验证码
pub async fn get_captcha() -> Result<(String, String)> {
    request::get_captcha_proxy().await
}

// 设置选课批次
pub async fn set_batch(
    batch_idx: usize,
    app_state: &AppState,
) -> Result<()> {
    let token = app_state.token.get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_list = app_state.batch_list.get();
    
    if batch_idx >= batch_list.len() {
        return Err(ErrorKind::ParseError("Invalid batch index".to_string()).into());
    }

    let batch_id = batch_list[batch_idx].code.clone();
    let resp = request::set_batch_proxy(&batch_id, &token).await?;

    if resp["code"] != 200 {
        return Err(ErrorKind::ParseError("Failed to set batch".to_string()).into());
    }

    app_state.batch_id.set(Some(batch_id));
    Ok(())
}

// 获取课程列表
pub async fn get_courses(app_state: &AppState) -> Result<()> {
    let token = app_state.token.get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_id = app_state.batch_id.get()
        .ok_or_else(|| ErrorKind::ParseError("No batch id selected".to_string()))?;

    let selected = request::get_selected_courses_proxy(&token, &batch_id).await?;
    let favorite = request::get_favorite_courses_proxy(&token, &batch_id).await?;

    let selected_courses: Vec<CourseInfo> = if selected["code"] == 200 {
        serde_json::from_value(selected["data"].clone())?
    } else {
        return Err(ErrorKind::CourseError(selected["msg"].to_string()).into());
    };

    let favorite_courses: Vec<CourseInfo> = if favorite["code"] == 200 {
        serde_json::from_value(favorite["data"].clone())?
    } else {
        return Err(ErrorKind::CourseError(favorite["msg"].to_string()).into());
    };

    app_state.selected_courses.set(selected_courses);
    app_state.favorite_courses.set(favorite_courses);
    Ok(())
}

// 选课函数
pub async fn enroll_courses(
    courses: Vec<CourseInfo>,
    try_if_capacity_full: bool,
    app_state: &AppState,
) -> Result<()> {
    if courses.is_empty() {
        return Ok(());
    }

    let token = app_state.token.get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_id = app_state.batch_id.get()
        .ok_or_else(|| ErrorKind::ParseError("No batch id selected".to_string()))?;

    app_state.should_continue.set(true);
    app_state.enrollment_status.update(|status| {
        status.is_running = true;
        status.course_statuses = courses.iter()
            .map(|c| format!("[{}]等待中", c.KCM))
            .collect();
    });

    let courses_count = courses.len();
    
    // 创建工作任务
    for thread_id in 0..12 {
        let token = token.clone();
        let batch_id = batch_id.clone();
        let courses = courses.clone();
        let app_state = app_state.clone();

        spawn_local(async move {
            let mut course_idx = thread_id % courses_count;
            
            while app_state.should_continue.get() {
                let course = &courses[course_idx];
                
                // 更新状态
                app_state.enrollment_status.update(|status| {
                    status.total_requests += 1;
                });

                // 尝试选课
                let result = request::select_course_proxy(
                    &token,
                    &batch_id,
                    &course.teaching_class_type.clone().unwrap_or_default(),
                    &course.JXBID,
                    &course.secret_val.clone().unwrap_or_default()
                ).await;

                match result {
                    Ok(json) => {
                        let code = json["code"].as_i64().unwrap_or(0);
                        let msg = json["msg"].as_str().unwrap_or("");
                        
                        let status = match (code, msg) {
                            (200, _) => {
                                app_state.should_continue.set(false);
                                "选课成功"
                            },
                            (500, "该课程已在选课结果中") => {
                                app_state.should_continue.set(false);
                                "已选"
                            },
                            (500, "本轮次选课暂未开始") => "未开始",
                            (500, "课容量已满") if !try_if_capacity_full => {
                                app_state.should_continue.set(false);
                                "已满"
                            },
                            (500, "课容量已满") => "等待中",
                            (500, "参数校验不通过") => "参数错误",
                            (401, _) => {
                                app_state.should_continue.set(false);
                                "未登录"
                            },
                            _ => "失败"
                        };

                        app_state.enrollment_status.update(|s| {
                            s.course_statuses[course_idx] = format!("[{}]{}", course.KCM, status);
                        });
                    },
                    Err(e) => {
                        app_state.enrollment_status.update(|s| {
                            s.course_statuses[course_idx] = format!("[{}]请求错误", course.KCM);
                        });
                        log::error!("请求错误: {:?}", e);
                    }
                }

                if !app_state.should_continue.get() {
                    break;
                }

                course_idx = (course_idx + 1) % courses_count;
                
                // 短暂延迟避免请求过快
                set_timeout(|| {}, 200).await;
            }
        });
    }

    Ok(())
}

// 停止选课
pub fn stop_enrollment(app_state: &AppState) {
    app_state.should_continue.set(false);
    app_state.enrollment_status.update(|status| {
        status.is_running = false;
    });
}

// Leptos组件示例
#[component]
pub fn EnrollmentPanel( app_state: AppState) -> impl IntoView {
    let enrollment_status = app_state.enrollment_status;
    
    view! { 
        <div class="p-4 border rounded shadow-sm mt-6">
            <h2 class="text-xl font-medium mb-4">"选课状态"</h2>
            <div class="text-gray-600 mb-4">
                {move || format!("总请求次数: {}", enrollment_status.get().total_requests)}
            </div>
            <div class="space-y-2">
                <For
                    each=move || enrollment_status.get().course_statuses.clone()
                    key=|status| status.clone()
                    children=move | status| view! { 
                        <div class="p-2 bg-gray-50 rounded">
                            {status}
                        </div>
                    }
                />
            </div>
        </div>
    }
}

// Utility functions
async fn set_timeout(f: impl FnOnce() + 'static, ms: i32) {
    use wasm_bindgen_futures::JsFuture;
    use wasm_bindgen::prelude::*;
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms,
            )
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}

#[component]
pub fn App() -> impl IntoView {
// 把 app_state 转换为 Resource
let app_state = RwSignal::new(AppState::new());
    
let (username, set_username) = signal(String::new());
let (password, set_password) = signal(String::new());
let (captcha, set_captcha) = signal(String::new());
let (captcha_image_src, set_captcha_image_src) = signal(String::new());
let (captcha_uuid, set_captcha_uuid) = signal(String::new());
let (status_message, set_status_message) = signal(String::new());
let (step, set_step) = signal(1);
let (is_enrolling, set_is_enrolling) = signal(false);

// 获取验证码
let handle_get_captcha = move |_| {
    spawn_local(async move {
        match get_captcha().await {
            Ok((uuid, captcha_b64)) => {
                set_captcha_uuid.set(uuid);
                // let image_src = format!("data:image/png;base64,{}", captcha_b64);
                let image_src = format!("{}", captcha_b64);
                set_captcha_image_src.set(image_src);
            }
            Err(e) => {
                set_status_message.set(format!("获取验证码失败：{:?}", e));
            }
        }
    });
};

// 登录处理
let handle_login = move |ev: web_sys::SubmitEvent| {
    ev.prevent_default();
    
    if username.get().is_empty() {
        set_status_message.set("请输入用户名".to_string());
        return;
    }
    if password.get().is_empty() {
        set_status_message.set("请输入密码".to_string());
        return;
    }
    if captcha.get().is_empty() {
        set_status_message.set("请输入验证码".to_string());
        return;
    }

    let current_state = app_state.get();
    spawn_local(async move {
        match login(
            &username.get(),
            &password.get(),
            &captcha.get(),
            &captcha_uuid.get(),
            &current_state
        ).await {
            Ok(()) => {
                set_step.set(2);
                set_status_message.set("登录成功！".to_string());
            }
            Err(e) => {
                set_status_message.set(format!("登录失败：{:?}", e));
                handle_get_captcha(());
            }
        }
    });
};

// 选择批次
// 修改批次选择的处理逻辑
let handle_batch_select = move |idx: usize| {
    let current_state = app_state.get();
    set_status_message.set("正在设置批次...".to_string());
    
    spawn_local(async move {
        match set_batch(idx, &current_state).await {
            Ok(()) => {
                set_step.set(3);
                match get_courses(&current_state).await {
                    Ok(()) => set_status_message.set("获取课程成功".to_string()),
                    Err(e) => set_status_message.set(format!("获取课程失败：{:?}", e)),
                }
            }
            Err(e) => set_status_message.set(format!("选择批次失败：{:?}", e)),
        }
    });
};
// 开始抢课
let handle_enroll = move |_| {
    set_is_enrolling.set(true);
    let current_state = app_state.get();
    spawn_local(async move {
        let courses = current_state.favorite_courses.get();
        if let Err(e) = enroll_courses(courses, true, &current_state).await {
            set_status_message.set(format!("抢课出错：{:?}", e));
            set_is_enrolling.set(false);
        }
    });
};

// 停止抢课
let handle_stop_enroll = move |_| {
    set_is_enrolling.set(false);
    let current_state = app_state.get();
    stop_enrollment(&current_state);
};

// 初始化时获取验证码
Effect::new(move |_| handle_get_captcha(()));

// 在使用 batch_list 时使用 app_state
let batch_list = move || app_state.get().batch_list.get();

    view! {
        <main class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold mb-8 text-center">"FunkyLesson自动抢课！(๑˃ᴗ˂)ﻭ"</h1>

            // 状态消息
            <div class="mb-4 text-center">
                <p class={move || {
                    let base = "px-4 py-2 rounded";
                    if status_message.get().contains("成功") {
                        format!("{} bg-green-100 text-green-700", base)
                    } else if status_message.get().contains("失败") {
                        format!("{} bg-red-100 text-red-700", base)
                    } else {
                        format!("{} bg-blue-100 text-blue-700", base)
                    }
                }}>
                    {move || status_message.get()}
                </p>
            </div>

            // 登录表单
            <div class="max-w-md mx-auto" class:hidden={move || step.get() != 1}>
                <form class="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4" on:submit=handle_login>
                    <div class="mb-4">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="username">
                            "用户名"
                        </label>
                        <input
                            id="username"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            type="text"
                            placeholder="请输入用户名"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="mb-6">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="password">
                            "密码"
                        </label>
                        <input
                            id="password"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 mb-3 leading-tight focus:outline-none focus:shadow-outline"
                            type="password"
                            placeholder="请输入密码"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    // 验证码部分
                    <div class="mb-4">
                        <div class="flex items-center justify-between mb-2">
                            <img
                                src={move || captcha_image_src.get()}
                                alt="验证码"
                                class="h-10 border rounded"
                            />
                            <button
                                type="button"
                                class="bg-gray-500 hover:bg-gray-700 text-white font-bold py-1 px-2 rounded"
                                on:click=move |_| handle_get_captcha(())
                            >
                                "刷新验证码"
                            </button>
                        </div>
                        <input
                            type="text"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            placeholder="请输入验证码"
                            on:input=move |ev| set_captcha.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="flex items-center justify-center">
                        <button
                            class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                            type="submit"
                        >
                            "登录"
                        </button>
                    </div>
                </form>
            </div>

            // 批次选择
            <div class="max-w-md mx-auto" class:hidden={move || step.get() != 2}>
                <div class="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4">
                    <h2 class="text-xl font-bold mb-4">"选择批次"</h2>
                    <div class="space-y-2">
                        <For
                            each=move || batch_list().into_iter().enumerate()
                            key=|(_idx, batch)| batch.code.clone()
                            children=move |(idx, batch)| {
                                let handle_select = handle_batch_select.clone();
                                view! {
                                    <button
                                        class="w-full text-left px-4 py-2 bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-50"
                                        on:click=move |_| handle_select(idx)
                                        disabled=move || is_enrolling.get()
                                    >
                                        {format!("{} - {} (批次 {})", batch.code, batch.name, idx + 1)}
                                    </button>
                                }
                            }
                        />
                    </div>
                </div>
            </div>

            // 课程选择和抢课
            <div class="max-w-4xl mx-auto" class:hidden={move || step.get() != 3}>
                <div class="bg-white shadow-md rounded px-8 pt-6 pb-8 mb-4">
                    // 请求统计
                    <div class="mb-4 text-center">
                        <span class="font-bold">"总请求次数: "</span>
                        <span class="text-blue-600">
                            {move || app_state.get().enrollment_status.get().total_requests}
                        </span>
                    </div>

                    // 课程状态输出
                    <div class="bg-black text-green-400 p-4 rounded h-64 overflow-y-auto font-mono mb-6">
                        <For
                            each=move || app_state.get().enrollment_status.get().course_statuses
                            key=|status| status.clone()
                            children=move |status| {
                                view! {
                                    <div class="whitespace-pre-wrap">{status}</div>
                                }
                            }
                        />
                    </div>

                    <div class="mt-6 flex justify-center space-x-4">
                        <button
                            class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                            on:click=handle_enroll
                            disabled=move || is_enrolling.get()
                        >
                            "开始抢课"
                        </button>
                        <button
                            class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                            on:click=handle_stop_enroll
                            disabled=move || !is_enrolling.get()
                        >
                            "停止抢课"
                        </button>
                    </div>

                    <div class="grid grid-cols-2 gap-4 mt-6">
                        <div>
                            <h3 class="text-lg font-bold mb-2">"已选课程"</h3>
                            <div class="space-y-2">
                                <For
                                    each=move || app_state.get().selected_courses.get()
                                    key=|course| course.JXBID.clone()
                                    children=move |course| {
                                        view! {
                                            <div class="p-2 bg-gray-100 rounded">
                                                {format!("{} - {} - {}", course.JXBID, course.KCM, course.SKJS)}
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                        <div>
                            <h3 class="text-lg font-bold mb-2">"待选课程"</h3>
                            <div class="space-y-2">
                                <For
                                    each=move || app_state.get().favorite_courses.get()
                                    key=|course| course.JXBID.clone()
                                    children=move |course| {
                                        view! {
                                            <div class="p-2 bg-gray-100 rounded">
                                                {format!("{} - {} - {}", course.JXBID, course.KCM, course.SKJS)}
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    }
}