use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use log::{debug, error, info, warn};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    info!("Starting universal proxy server at http://127.0.0.1:3030");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://tauri.localhost")
            .allowed_origin("http://localhost:1420")
            .allowed_origin("http://127.0.0.1:1420")
            // .allowed_origin("file://")
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![
                "Authorization",
                "Content-Type",
                "Accept",
                "Origin",
                "X-Requested-With",
                "batchId",
            ])
            .supports_credentials()
            .max_age(3600);

        App::new().wrap(cors).service(
            web::resource("/api/proxy/{endpoint:.*}")
                .route(web::post().to(proxy_handler))
                .route(web::get().to(proxy_handler_get)),
        )
    })
    .bind("127.0.0.1:3030")?
    .run()
    .await
}

#[derive(Debug, Deserialize)]
struct ProxyRequest {
    original_url: String,
    #[serde(default)]
    batch_id: Option<String>,
    #[serde(default)]
    class_type: Option<String>,
    #[serde(default)]
    class_id: Option<String>,
    #[serde(default)]
    secret_val: Option<String>,
    #[serde(default)]
    params: Option<HashMap<String, String>>,
    // 新增登录相关字段
    #[serde(default)]
    loginname: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    captcha: Option<String>,
    #[serde(default)]
    uuid: Option<String>,
}

async fn proxy_handler_get(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let endpoint = path.into_inner();
    debug!("Handling GET proxy request for endpoint: {}", endpoint);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default();

    // 获取原始URL
    let original_url = match endpoint.as_str() {
        "profile/index.html" => "https://icourses.jlu.edu.cn/xsxk/profile/index.html",
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid endpoint for GET request"
            }));
        }
    };

    let auth_token = match req.headers().get(actix_web::http::header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
            Ok(t) => Some(t.to_string()),
            Err(e) => {
                error!("Invalid authorization token: {}", e);
                return HttpResponse::BadRequest().json(json!({
                    "error": "Invalid authorization token"
                }));
            }
        },
        None => None,
    };

    let mut headers = HeaderMap::new();
    if let Some(token) = auth_token {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    }

    debug!("Sending GET request to: {}", original_url);

    match client.get(original_url).headers(headers).send().await {
        Ok(response) => {
            let status = response.status();
            debug!("Received response with status: {}", status);

            match response.text().await {
                Ok(text) => HttpResponse::Ok().content_type("text/html").body(text),
                Err(e) => {
                    error!("Failed to get response text: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to read response: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            error!("Request failed: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Request failed: {}", e)
            }))
        }
    }
}

async fn proxy_handler(
    body: web::Json<ProxyRequest>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let endpoint = path.into_inner();
    debug!("Handling proxy request for endpoint: {}", endpoint);
    debug!("Request body: {:?}", body);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default();

    let auth_token = match req.headers().get(actix_web::http::header::AUTHORIZATION) {
        Some(token) => match token.to_str() {
            Ok(t) => Some(t.to_string()),
            Err(e) => {
                error!("Invalid authorization token: {}", e);
                return HttpResponse::BadRequest().json(json!({
                    "error": "Invalid authorization token"
                }));
            }
        },
        None => None,
    };

    let mut headers = HeaderMap::new();
    if let Some(token) = auth_token {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&token).unwrap());
    }

    // 构建请求体
    let mut request_body = HashMap::new();
    if let Some(loginname) = &body.loginname {
        request_body.insert("loginname", loginname);
    }
    if let Some(password) = &body.password {
        request_body.insert("password", password);
    }
    if let Some(captcha) = &body.captcha {
        request_body.insert("captcha", captcha);
    }
    if let Some(uuid) = &body.uuid {
        request_body.insert("uuid", uuid);
    }

    // 构建查询参数
    let mut query_params = HashMap::new();
    if let Some(batch_id) = &body.batch_id {
        query_params.insert("batchId", batch_id);
    }
    if let Some(class_type) = &body.class_type {
        query_params.insert("clazzType", class_type);
    }
    if let Some(class_id) = &body.class_id {
        query_params.insert("clazzId", class_id);
    }
    if let Some(secret_val) = &body.secret_val {
        query_params.insert("secretVal", secret_val);
    }
    if let Some(extra_params) = &body.params {
        query_params.extend(extra_params.iter().map(|(k, v)| (k.as_str(), v)));
    }

    debug!("Sending request to: {}", body.original_url);
    debug!("Headers: {:?}", headers);
    debug!("Query params: {:?}", query_params);
    debug!("Request body: {:?}", request_body);

    let mut request = client.post(&body.original_url).headers(headers);

    // 添加请求体或查询参数
    if !request_body.is_empty() {
        request = request.query(&request_body);
    }
    if !query_params.is_empty() {
        request = request.query(&query_params);
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            debug!("Received response with status: {}", status);

            match response.text().await {
                Ok(text) => {
                    debug!("Response text: {}", text);
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(json_value) => HttpResponse::Ok()
                            .content_type("application/json")
                            .json(json_value),
                        Err(e) => {
                            warn!(
                                "Failed to parse response as JSON: {}, returning raw text",
                                e
                            );
                            HttpResponse::Ok().content_type("text/plain").body(text)
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get response text: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to read response: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            error!("Request failed: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Request failed: {}", e)
            }))
        }
    }
}
// use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
// use actix_cors::Cors;
// use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
// use serde_json::json;
// use serde::Deserialize;
// use log::{info, error, debug, warn};
// use std::collections::HashMap;

// #[actix_web::main]
// pub async fn main() -> std::io::Result<()> {
//     env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

//     info!("Starting universal proxy server at http://127.0.0.1:3030");

//     HttpServer::new(|| {
//         let cors = Cors::default()
// .allowed_origin("http://tauri.localhost")
//             .allowed_origin("http://localhost:1420")
//             .allowed_methods(vec!["GET", "POST", "OPTIONS"])
//             .allowed_headers(vec![
//                 "Authorization",
//                 "Content-Type",
//                 "Accept",
//                 "Origin",
//                 "X-Requested-With",
//                 "batchId",
//             ])
//             .supports_credentials()
//             .max_age(3600);

//         App::new()
//             .wrap(cors)
//             .service(
//                 web::resource("/api/proxy/{endpoint:.*}")
//                     .route(web::post().to(proxy_handler))
//             )
//     })
//     .bind("127.0.0.1:3030")?
//     .run()
//     .await
// }

// #[derive(Debug, Deserialize)]
// struct ProxyRequest {
//     original_url: String,
//     #[serde(default)]
//     batch_id: Option<String>,
//     #[serde(default)]
//     class_type: Option<String>,
//     #[serde(default)]
//     class_id: Option<String>,
//     #[serde(default)]
//     secret_val: Option<String>,
//     #[serde(default)]
//     params: Option<HashMap<String, String>>,
// }

// async fn proxy_handler(
//     body: web::Json<ProxyRequest>,
//     req: HttpRequest,
//     path: web::Path<String>,
// ) -> HttpResponse {
//     let endpoint = path.into_inner();
//     debug!("Handling proxy request for endpoint: {}", endpoint);
//     debug!("Request body: {:?}", body);

//     // 创建支持自签名证书的客户端
//     let client = reqwest::Client::builder()
//         .danger_accept_invalid_certs(true)
//         .build()
//         .unwrap_or_default();

//     // 从请求头获取认证信息
//     let auth_token = match req.headers().get(actix_web::http::header::AUTHORIZATION) {
//         Some(token) => match token.to_str() {
//             Ok(t) => Some(t.to_string()),
//             Err(e) => {
//                 error!("Invalid authorization token: {}", e);
//                 return HttpResponse::BadRequest().json(json!({
//                     "error": "Invalid authorization token"
//                 }));
//             }
//         },
//         None => None,
//     };

//     // 构建请求头
//     let mut headers = HeaderMap::new();
//     if let Some(token) = auth_token {
//         headers.insert(
//             AUTHORIZATION,
//             HeaderValue::from_str(&token).unwrap()
//         );
//     }

//     // 如果有 batchId，添加到请求头
//     if let Some(batch_id) = &body.batch_id {
//         headers.insert(
//             "batchId",
//             HeaderValue::from_str(batch_id).unwrap()
//         );
//     }

//     // 构建查询参数
//     let mut query_params = HashMap::new();
//     if let Some(batch_id) = &body.batch_id {
//         query_params.insert("batchId", batch_id);
//     }
//     if let Some(class_type) = &body.class_type {
//         query_params.insert("clazzType", class_type);
//     }
//     if let Some(class_id) = &body.class_id {
//         query_params.insert("clazzId", class_id);
//     }
//     if let Some(secret_val) = &body.secret_val {
//         query_params.insert("secretVal", secret_val);
//     }
//     // 合并额外的参数
//     if let Some(extra_params) = &body.params {
//         query_params.extend(extra_params.iter().map(|(k, v)| (k.as_str(), v)));
//     }

//     debug!("Sending request to: {}", body.original_url);
//     debug!("Headers: {:?}", headers);
//     debug!("Query params: {:?}", query_params);

//     // 发送请求
//     let resp = client
//         .post(&body.original_url)
//         .headers(headers)
//         .query(&query_params)
//         .send()
//         .await;

//     match resp {
//         Ok(response) => {
//             let status = response.status();
//             debug!("Received response with status: {}", status);

//             match response.text().await {
//                 Ok(text) => {
//                     debug!("Response text: {}", text);
//                     match serde_json::from_str::<serde_json::Value>(&text) {
//                         Ok(json_value) => {
//                             HttpResponse::Ok()
//                                 .content_type("application/json")
//                                 .json(json_value)
//                         },
//                         Err(e) => {
//                             warn!("Failed to parse response as JSON: {}, returning raw text", e);
//                             HttpResponse::Ok()
//                                 .content_type("text/plain")
//                                 .body(text)
//                         }
//                     }
//                 },
//                 Err(e) => {
//                     error!("Failed to get response text: {}", e);
//                     HttpResponse::InternalServerError().json(json!({
//                         "error": format!("Failed to read response: {}", e)
//                     }))
//                 }
//             }
//         },
//         Err(e) => {
//             error!("Request failed: {}", e);
//             HttpResponse::InternalServerError().json(json!({
//                 "error": format!("Request failed: {}", e)
//             }))
//         }
//     }
// }
