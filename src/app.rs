use leptos::task::spawn_local;
use leptos::*;
use leptos::prelude::*;
// use serde::{Deserialize, Serialize};
// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

// #[derive(Serialize, Deserialize)]
// struct GreetArgs<'a> {
//     name: &'a str,
// }
// 状态类型定义
#[derive(Clone, Debug, PartialEq)]
struct AppState {
    username: String,
    password: String,
    batch_idx: String,
    is_loop: bool,
    status_messages: Vec<String>,
    is_running: bool,
}

// 主组件
#[component]
pub fn App() -> impl IntoView {
    // 创建响应式状态
    let (state, set_state) = signal(AppState {
        username: String::new(),
        password: String::new(),
        batch_idx: String::new(),
        is_loop: false,
        status_messages: vec![],
        is_running: false,
    });

    // 添加状态消息的函数
    let add_message = move |message: String| {
        set_state.update(|s| {
            let mut new_state = s.clone();
            new_state.status_messages.push(message);
            *s = new_state;
        });
    };

    // 开始选课
    let start_enrollment = move |_| {
        set_state.update(|s| s.is_running = true);
        
        // 这里需要调用你的选课核心逻辑
        spawn_local(async move {
            // TODO: 实现选课逻辑
            add_message("开始登录...".to_string());
            // 模拟选课过程
            add_message("登录成功".to_string());
            add_message("获取课程列表...".to_string());
            add_message("开始选课...".to_string());
        });
    };

    // 停止选课
    let stop_enrollment = move |_| {
        set_state.update(|s| s.is_running = false);
        add_message("已停止选课".to_string());
    };

    view! {
        <div class="min-h-screen bg-gray-100 p-6">
            <div class="max-w-2xl mx-auto bg-white rounded-lg shadow-md p-6">
                <h1 class="text-2xl font-bold mb-6 text-gray-800">"自动选课系统"</h1>
                
                // 表单部分
                <div class="space-y-4 mb-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "用户名"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md"
                            prop:value=move || state.get().username
                            on:input=move |ev| {
                                set_state.update(|s| s.username = event_target_value(&ev));
                            }
                        />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "密码"
                        </label>
                        <input
                            type="password"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md"
                            prop:value=move || state.get().password
                            on:input=move |ev| {
                                set_state.update(|s| s.password = event_target_value(&ev));
                            }
                        />
                    </div>
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "选课批次(从0开始)"
                        </label>
                        <input
                            type="number"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md"
                            prop:value=move || state.get().batch_idx
                            on:input=move |ev| {
                                set_state.update(|s| s.batch_idx = event_target_value(&ev));
                            }
                        />
                    </div>
                    
                    <div class="flex items-center">
                        <input
                            type="checkbox"
                            class="h-4 w-4 text-blue-600"
                            prop:checked=move || state.get().is_loop
                            on:change=move |ev| {
                                set_state.update(|s| s.is_loop = event_target_checked(&ev));
                            }
                        />
                        <label class="ml-2 text-sm text-gray-700">
                            "循环模式"
                        </label>
                    </div>
                </div>
                
                // 操作按钮
                <div class="flex space-x-4 mb-6">
                    <button
                        class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:bg-gray-400"
                        prop:disabled=move || state.get().is_running
                        on:click=start_enrollment
                    >
                        "开始选课"
                    </button>
                    <button
                        class="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 disabled:bg-gray-400"
                        prop:disabled=move || !state.get().is_running
                        on:click=stop_enrollment
                    >
                        "停止选课"
                    </button>
                </div>
                
                // 状态输出
                <div class="border rounded-md p-4 bg-gray-50">
                    <h2 class="text-lg font-semibold mb-2">"运行状态"</h2>
                    <div class="space-y-1 h-64 overflow-y-auto">
                        <For
                            each=move || state.get().status_messages.clone()
                            key=|message| message.clone()
                            children=move |message| {
                                view! {
                                    <div class="text-sm text-gray-600">
                                        {message}
                                    </div>
                                }
                            }
                        />
                        // {move || state.get().status_messages.iter().map(|msg| {
                        //     view! {
                        //         <div class="text-sm text-gray-600">
                        //             {msg}
                        //         </div>
                        //     }
                        // }).collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}