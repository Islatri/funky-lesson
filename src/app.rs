use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace=["console"], js_name="log")]
    pub fn console_log(s: &str, args: JsValue);
}

// Add new structs for captcha and enrollment status
#[derive(Serialize, Deserialize)]
struct CaptchaResponse {
    uuid: String,
    image_base64: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EnrollmentStatus {
    total_requests: u32,
    course_statuses: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct LoginArgs {
    username: String,
    password: String,
    captcha: String,
    uuid: String,
}

#[derive(Serialize, Deserialize)]
struct BatchArgs {
    #[serde(rename = "batchIdx")]
    batch_idx: usize,
    #[serde(rename = "batchList")]
    batch_list: Vec<BatchInfo>,
}

#[derive(Serialize, Deserialize)]
struct EnrollArgs {
    #[serde(rename = "favoriteCourses")]
    favorite_courses: Vec<CourseInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BatchInfo {
    pub code: String,
    pub name: String,
    #[serde(rename = "beginTime")]
    pub begin_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseInfo {
    #[serde(rename = "SKJS")]
    pub skjs: String, // 教师名
    #[serde(rename = "KCM")]
    pub kcm: String, // 课程名
    #[serde(rename = "JXBID")]
    pub jxbid: String, // 教学班ID
    #[serde(rename = "teachingClassType")]
    pub teaching_class_type: Option<String>,
    #[serde(default, rename = "secretVal")]
    pub secret_val: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
    batch_list: Vec<BatchInfo>,
}

#[component]
pub fn App() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (batch_list, set_batch_list) = signal(Vec::<BatchInfo>::new());
    let (selected_courses, set_selected_courses) = signal(Vec::<CourseInfo>::new());
    let (favorite_courses, set_favorite_courses) = signal(Vec::<CourseInfo>::new());
    let (status_message, set_status_message) = signal(String::new());
    let (step, set_step) = signal(1); // 1: login, 2: batch selection, 3: course selection

    // New signals for captcha and enrollment status
    let (captcha, set_captcha) = signal(String::new());
    let (captcha_uuid, set_captcha_uuid) = signal(String::new());
    let (captcha_image_src, set_captcha_image_src) = signal(String::new());
    let (total_requests, set_total_requests) = signal(0u32);
    let (course_statuses, set_course_statuses) = signal(Vec::<String>::new());
    let (is_enrolling, set_is_enrolling) = signal(false);

    // 初始化客户端
    spawn_local(async move {
        invoke("initialize_client", JsValue::NULL).await;
    });

    
    // 获取验证码
    let handle_get_captcha = move |_| {
        spawn_local(async move {
            let result = invoke("get_captcha", JsValue::NULL).await;
            let captcha_response: Result<CaptchaResponse, _> = serde_wasm_bindgen::from_value(result);
            
            match captcha_response {
                Ok(response) => {
                // 先设置 UUID
                set_captcha_uuid.set(response.uuid);
                
                // 确保 base64 数据没有前缀
                let image_data = response.image_base64.trim();
                
                // 设置完整的 data URL
                let image_src = format!("data:image/png;base64,{}", image_data);
                console_log("image_src set to:", serde_wasm_bindgen::to_value(&image_src).unwrap());

                set_captcha_image_src.set(image_src);
                
    
                }
                Err(e) => {
                    set_status_message.set(format!("获取验证码失败：{}", e));
                }
            }
        });
    };

    // 登录处理
    let handle_login = move |ev: SubmitEvent| {
        ev.prevent_default();
            // 表单验证
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

        spawn_local(async move {
            let args = LoginArgs {
                username: username.get(),
                password: password.get(),
                captcha: captcha.get(),
                uuid: captcha_uuid.get(),
            };
            
            let result = invoke("login_command", serde_wasm_bindgen::to_value(&args).unwrap()).await;
            let login_response: Result<LoginResponse, _> = serde_wasm_bindgen::from_value(result);
            
            match login_response {
                Ok(response) => {
                    set_batch_list.set(response.batch_list);
                    set_step.set(2);
                    set_status_message.set("登录成功！".to_string());
                }
                Err(e) => {
                    set_status_message.set(format!("登录失败：{}", e));
                    // 登录失败时自动刷新验证码
                    handle_get_captcha(());
                }
            }
        });
    };

    // 开始抢课
    let handle_enroll = move |_| {
        set_is_enrolling.set(true);
        spawn_local(async move {
            let args = EnrollArgs {
                favorite_courses: favorite_courses.get(),
            };
            
            while is_enrolling.get() {
                let result = invoke(
                    "enroll_courses_command",
                    serde_wasm_bindgen::to_value(&args).unwrap()
                ).await;
                let status: Result<EnrollmentStatus, _> = serde_wasm_bindgen::from_value(result);
                
                match status {
                    Ok(status) => {
                        set_total_requests.set(status.total_requests);
                        set_course_statuses.set(status.course_statuses);
                        
                    }
                    Err(e) => {
                        set_status_message.set(format!("抢课出错：{}", e));
                        set_is_enrolling.set(false);
                        break;
                    }
                }
                
                // 短暂延迟，避免请求过快
                
                TimeoutFuture::new(100).await;
            }
        });
    };

    // 停止抢课
    let handle_stop_enroll = move |_| {
        spawn_local(async move {
            let _ = invoke("stop_enrollment", JsValue::NULL).await;
            set_is_enrolling.set(false);
        });
    };

    // 选择批次
    let handle_batch_select = move |idx: usize| {
        spawn_local(async move {
            let args = BatchArgs {
                batch_idx: idx,
                batch_list: batch_list.get(),
            };

            let batch_result = invoke(
                "set_batch_command",
                serde_wasm_bindgen::to_value(&args).unwrap(),
            )
            .await;
            let batch_response: Result<(), _> = serde_wasm_bindgen::from_value(batch_result);

            match batch_response {
                Ok(_) => {
                    set_step.set(3);
                    // 获取课程列表
                    let courses_result = invoke("get_courses_command", JsValue::NULL).await;
                    let courses: Result<(Vec<CourseInfo>, Vec<CourseInfo>), _> =
                        serde_wasm_bindgen::from_value(courses_result);

                    match courses {
                        Ok((selected, favorite)) => {
                            set_selected_courses.set(selected);
                            set_favorite_courses.set(favorite);
                        }
                        Err(e) => {
                            set_status_message.set(format!("获取课程失败：{}", e));
                        }
                    }
                }
                Err(e) => {
                    set_status_message.set(format!("选择批次失败：{}", e));
                }
            }
        });
    };

    Effect::new(move|_| handle_get_captcha(()));


    view! {
        <main class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold mb-8 text-center">"自动抢课系统"</h1>

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
                    { move || status_message.get() }
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
                        each=move || batch_list.get().into_iter().enumerate()
                        key=|(_idx, batch)| batch.code.clone()
                        children=move |(idx, batch)| {
                            view! {
                                <button
                                    class="w-full text-left px-4 py-2 bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-50"
                                    on:click=move |_| handle_batch_select(idx)
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
                <span class="text-blue-600">{move || total_requests.get()}</span>
            </div>

            // 课程状态输出
            <div class="bg-black text-green-400 p-4 rounded h-64 overflow-y-auto font-mono mb-6">
                <For
                    each=move || course_statuses.get()
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
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <h3 class="text-lg font-bold mb-2">"已选课程"</h3>
                            <div class="space-y-2">
                                <For
                                    each=move || selected_courses.get().into_iter()
                                    key=|course| course.jxbid.clone()
                                    children=move |course| {
                                        view! {
                                            <div class="p-2 bg-gray-100 rounded">
                                                {format!("{} - {} - {}", course.jxbid, course.kcm, course.skjs)}
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
                                    each=move || favorite_courses.get().into_iter()
                                    key=|course| course.jxbid.clone()
                                    children=move |course| {
                                        view! {
                                            <div class="p-2 bg-gray-100 rounded">
                                            {format!("{} - {} - {}", course.jxbid, course.kcm, course.skjs)}
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