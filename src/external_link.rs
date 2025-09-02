use crate::external_browser::{is_android_webview, open_external_browser};
use leptos::children::Children;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn ExternalLink(
    href: String,
    children: Children,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let href_for_click = href.clone();
    let href_for_view = href.clone();
    let handle_click = move |ev: web_sys::MouseEvent| {
        // 阻止默认的链接行为
        ev.prevent_default();

        let url = href_for_click.clone();
        spawn_local(async move {
            if let Err(e) = open_external_browser(&url) {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "Failed to open external browser: {e:?}"
                )));
            }
        });
    };

    view! {
        <a
            href=href_for_view
            class=class.unwrap_or_default()
            on:click=handle_click
            // 在非 Android 环境中保留这些属性
            target=move || if is_android_webview() { None } else { Some("_blank") }
            rel=move || if is_android_webview() { None } else { Some("noopener noreferrer") }
        >
            {children()}
        </a>
    }
}
