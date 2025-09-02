use leptos::web_sys;
use wasm_bindgen::prelude::*;

// 直接绑定到 window.Android 对象
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "Android"])]
    fn openInExternalBrowser(url: &str);

    #[wasm_bindgen(js_namespace = ["window", "Android"])]
    fn isExternalBrowserAvailable() -> bool;
}

// 检查是否在 Android WebView 环境中
pub fn is_android_webview() -> bool {
    js_sys::Reflect::has(
        &web_sys::window().unwrap().into(),
        &JsValue::from_str("Android"),
    )
    .unwrap_or(false)
}

pub fn open_external_browser(url: &str) -> Result<(), JsValue> {
    if is_android_webview() {
        // 直接调用 Android 接口
        openInExternalBrowser(url);
        Ok(())
    } else {
        // 在非 Android 环境中，使用标准的 window.open
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("No window object"))?
            .open_with_url_and_target(url, "_blank")?;
        Ok(())
    }
}
