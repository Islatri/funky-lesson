use funky_lesson_core::{
    crypto,
    error::{ ErrorKind, Result },
    client::gloo,
    model::structs::{ BatchInfo, CourseInfo, EnrollmentStatus },
};
use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

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
    app_state: &AppState
) -> Result<()> {
    // 初始化
    gloo::create_client().await?;

    // 获取AES密钥
    let aes_key = gloo::get_aes_key_proxy().await?;

    // 加密密码并登录
    let encrypted_password = crypto::encrypt_password(password, &aes_key)?;
    let login_resp = gloo::send_login_request_proxy(
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
    gloo::get_captcha_proxy().await
}

// 设置选课批次
pub async fn set_batch(batch_idx: usize, app_state: &AppState) -> Result<()> {
    let token = app_state.token
        .get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_list = app_state.batch_list.get();

    if batch_idx >= batch_list.len() {
        return Err(ErrorKind::ParseError("Invalid batch index".to_string()).into());
    }

    let batch_id = batch_list[batch_idx].code.clone();
    let resp = gloo::set_batch_proxy(&batch_id, &token).await?;

    if resp["code"] != 200 {
        return Err(ErrorKind::ParseError("Failed to set batch".to_string()).into());
    }

    app_state.batch_id.set(Some(batch_id));
    Ok(())
}

// 获取课程列表
pub async fn get_courses(app_state: &AppState) -> Result<()> {
    let token = app_state.token
        .get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_id = app_state.batch_id
        .get()
        .ok_or_else(|| ErrorKind::ParseError("No batch id selected".to_string()))?;

    let selected = gloo::get_selected_courses_proxy(&token, &batch_id).await?;
    let favorite = gloo::get_favorite_courses_proxy(&token, &batch_id).await?;

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
    app_state: &AppState
) -> Result<()> {
    if courses.is_empty() {
        return Ok(());
    }

    let token = app_state.token
        .get()
        .ok_or_else(|| ErrorKind::ParseError("No token available".to_string()))?;
    let batch_id = app_state.batch_id
        .get()
        .ok_or_else(|| ErrorKind::ParseError("No batch id selected".to_string()))?;

    app_state.should_continue.set(true);
    app_state.enrollment_status.update(|status| {
        status.is_running = true;
        status.course_statuses = courses
            .iter()
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
                let result = gloo::select_course_proxy(
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
                            }
                            (500, "该课程已在选课结果中") => {
                                app_state.should_continue.set(false);
                                "已选"
                            }
                            (500, "本轮次选课暂未开始") => "未开始",
                            (500, "课容量已满") if !try_if_capacity_full => {
                                app_state.should_continue.set(false);
                                "已满"
                            }
                            (500, "课容量已满") => "等待中",
                            (500, "参数校验不通过") => "参数错误",
                            (401, _) => {
                                app_state.should_continue.set(false);
                                "未登录"
                            }
                            _ => "失败",
                        };

                        app_state.enrollment_status.update(|s| {
                            s.course_statuses[course_idx] = format!("[{}]{}", course.KCM, status);
                        });
                    }
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
                set_timeout(200).await;
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

// Utility functions
async fn set_timeout(ms: i32) {
    use wasm_bindgen_futures::JsFuture;
    let promise = js_sys::Promise::new(
        &mut (|resolve, _| {
            web_sys
                ::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                .unwrap();
        })
    );
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
    let (status_message, set_status_message) = signal("请登录".to_string());
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
            set_status_message.set("请输入学号".to_string());
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
            match
                login(
                    &username.get(),
                    &password.get(),
                    &captcha.get(),
                    &captcha_uuid.get(),
                    &current_state
                ).await
            {
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
<div 
    id="app" 
    class="min-h-screen mx-auto w-full relative flex flex-col items-center justify-end p-4 sm:p-6"
    style="background: linear-gradient(135deg, rgba(0,0,0,0.1), rgba(0,0,0,0.05)), url('./public/91403676_p0_z2.jpg'); background-size: cover; background-position: center 20%; background-attachment: fixed;"
>
    {/* Logo和标题 - 独立的小卡片 */}
    <div class="text-center mb-6 bg-black/30 backdrop-blur-sm rounded-2xl p-3 border border-white/20"
        class:hidden={move || step.get() != 1}
        >
        <h1 class="text-lg sm:text-xl font-bold text-white drop-shadow-lg">
            "FunkyLesson自动抢课！(๑˃ᴗ˂)ﻭ"
        </h1>
        <div class="w-12 h-0.5 bg-gradient-to-r from-blue-400 to-purple-400 rounded-full mx-auto mt-2"></div>
    </div>

    // 登录表单
    <div class="w-full max-w-sm sm:max-w-md mx-auto" class:hidden={move || step.get() != 1}>
        <form class="mb-4 space-y-3" on:submit=handle_login>
            <div class="bg-black/30 backdrop-blur-sm rounded-xl p-4 border border-white/20 space-y-3">
                <div>
                    <label class="block text-xs font-medium text-white/80 mb-2">
                        学号 <span class="text-red-400">*</span>
                    </label>
                    <input
                        id="username"
                        class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all duration-300"
                        type="text"
                        placeholder="请输入学号"
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="block text-xs font-medium text-white/80 mb-2">
                        密码 <span class="text-red-400">*</span>
                    </label>
                    <input
                        id="password"
                        class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all duration-300"
                        type="password"
                        placeholder="请输入密码(默认是身份证后6位)"
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </div>

                // 验证码部分
                <div>
                    <label class="block text-xs font-medium text-white/80 mb-2">验证码</label>
                    <div class="flex items-center gap-2 mb-3">
                    
                        <input
                            type="text"
                            class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-sm placeholder-white/50 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all duration-300"
                            placeholder="请输入验证码"
                            on:input=move |ev| set_captcha.set(event_target_value(&ev))
                        />
                        <img
                            src={move || captcha_image_src.get()}
                            alt="验证码"
                            class="h-8 border border-white/20 rounded flex-shrink-0"
                        />
                        <button
                            type="button"
                            class="bg-green-500/80 hover:bg-green-600/80 text-white text-xs font-medium py-1.5 px-3 rounded-lg transition-all duration-300 whitespace-nowrap"
                            on:click=move |_| handle_get_captcha(())
                        >
                            "刷新"
                        </button>
                    </div>
                </div>

                <div class="flex flex-row items-center justify-between gap-3">
                    <p class={move || {
                        let base = "text-xs sm:text-sm font-mono break-words leading-relaxed flex-1";
                        if status_message.get().contains("成功") {
                            format!("{} text-green-300", base)
                        } else if status_message.get().contains("失败") {
                            format!("{} text-red-300", base)
                        } else if status_message.get().contains("请输入") {
                            format!("{} text-orange-300", base)
                        } else {
                            format!("{} text-white/70", base)
                        }
                    }}>
                        {move || status_message.get()}
                    </p>
        
                    <button
                        class="bg-blue-500/80 hover:bg-blue-600/80 text-white font-medium py-2 px-4 rounded-lg transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-blue-400"
                        type="submit"
                    >
                        "登录"
                    </button>
                </div>
            </div>
        </form>
    </div>

    // 批次选择
    <div class="w-full max-w-sm sm:max-w-md mx-auto" class:hidden={move || step.get() != 2}>
        <div class="bg-black/30 backdrop-blur-sm rounded-xl p-4 border border-white/20 space-y-4">
            <div class="text-center">
                <h2 class="text-lg sm:text-xl font-bold text-white drop-shadow-lg">"选择批次"</h2>
                <div class="w-8 h-0.5 bg-gradient-to-r from-blue-400 to-purple-400 rounded-full mx-auto mt-2"></div>
            </div>
            <div class="space-y-2">
                <For
                    each=move || batch_list().into_iter().enumerate()
                    key=|(_idx, batch)| batch.code.clone()
                    children=move |(idx, batch)| {
                        let handle_select = handle_batch_select.clone();
                        view! {
                            <button
                                class="w-full text-left px-4 py-3 bg-white/10 hover:bg-white/20 border border-white/20 rounded-lg text-white text-sm transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed"
                                on:click=move |_| handle_select(idx)
                                disabled=move || is_enrolling.get()
                            >
                                <div class="font-medium">{batch.name}</div>
                                <div class="text-xs text-white/70 mt-1">{format!("批次代码: {} | 批次 {}", batch.code, idx + 1)}</div>
                            </button>
                        }
                    }
                />
            </div>
        </div>
    </div>

    // 课程选择和抢课
    <div class="w-full max-w-4xl mx-auto" class:hidden={move || step.get() != 3}>
        <div class="space-y-4">
            
            // 标题和请求统计
            <div class="text-center mb-4">
                <h2 class="text-lg sm:text-xl font-bold text-white drop-shadow-lg mb-2">"抢课控制台"</h2>
                <div class="w-12 h-0.5 bg-gradient-to-r from-blue-400 to-purple-400 rounded-full mx-auto mb-3"></div>
                <div class="bg-black/30 backdrop-blur-sm rounded-lg p-3 border border-white/20">
                    <span class="text-white/80 text-sm">"总请求次数: "</span>
                    <span class="text-blue-300 font-bold text-lg">
                        {move || app_state.get().enrollment_status.get().total_requests}
                    </span>
                </div>
            </div>

            // 课程状态输出
            <div class="bg-black/80 backdrop-blur-sm text-green-400 p-4 rounded-xl h-48 sm:h-64 overflow-y-auto font-mono border border-white/20">
                <div class="text-xs text-white/60 mb-2 uppercase tracking-wide">"实时状态"</div>
                <For
                    each=move || app_state.get().enrollment_status.get().course_statuses
                    key=|status| status.clone()
                    children=move |status| {
                        view! {
                            <div class="whitespace-pre-wrap text-sm leading-relaxed">{status}</div>
                        }
                    }
                />
            </div>

            // 控制按钮
            <div class="flex flex-row justify-center gap-3 sm:gap-4">
                <button
                    class="bg-green-500/80 hover:bg-green-600/80 text-white font-medium py-3 px-6 rounded-lg transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-green-400 disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=handle_enroll
                    disabled=move || is_enrolling.get()
                >
                    "🚀 开始抢课"
                </button>
                <button
                    class="bg-red-500/80 hover:bg-red-600/80 text-white font-medium py-3 px-6 rounded-lg transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-red-400 disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=handle_stop_enroll
                    disabled=move || !is_enrolling.get()
                >
                    "⏹️ 停止抢课"
                </button>
            </div>

            // 课程列表
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-6">
                <div class="bg-black/30 backdrop-blur-sm rounded-xl p-4 border border-white/20">
                    <div class="flex items-center gap-2 mb-3">
                        <div class="w-3 h-3 bg-green-400 rounded-full"></div>
                        <h3 class="text-lg font-bold text-white">"已选课程"</h3>
                        <span class="text-white/70 text-sm">
                            {format!("共 {} 门", app_state.get().selected_courses.get().len())}
                        </span>
                    </div>
                    <div class="space-y-2 max-h-40 overflow-y-auto">
                        <For
                            each=move || app_state.get().selected_courses.get()
                            key=|course| course.JXBID.clone()
                            children=move |course| {
                                view! {
                                    <div class="p-3 bg-green-500/20 border border-green-400/30 rounded-lg">
                                        <div class="font-medium text-white text-sm">{course.KCM}</div>
                                        <div class="text-xs text-white/70 mt-1">
                                            {format!("教师: {} | ID: {}", course.SKJS, course.JXBID)}
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
                
                <div class="bg-black/30 backdrop-blur-sm rounded-xl p-4 border border-white/20">
                    <div class="flex items-center gap-2 mb-3">
                        <div class="w-3 h-3 bg-orange-400 rounded-full"></div>
                        <h3 class="text-lg font-bold text-white">"待选课程"</h3>
                        <span class="text-white/70 text-sm">
                            {format!("共 {} 门", app_state.get().favorite_courses.get().len())}
                        </span>
                    </div>
                    <div class="space-y-2 max-h-40 overflow-y-auto">
                        <For
                            each=move || app_state.get().favorite_courses.get()
                            key=|course| course.JXBID.clone()
                            children=move |course| {
                                view! {
                                    <div class="p-3 bg-orange-500/20 border border-orange-400/30 rounded-lg">
                                        <div class="font-medium text-white text-sm">{course.KCM}</div>
                                        <div class="text-xs text-white/70 mt-1">
                                            {format!("教师: {} | ID: {}", course.SKJS, course.JXBID)}
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>    

}
}
