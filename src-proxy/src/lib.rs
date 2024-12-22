use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
use actix_cors::Cors;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::json;
use serde::Deserialize;
use log::{info, error, debug, warn};
use std::collections::HashMap;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));
    
    info!("Starting universal proxy server at http://127.0.0.1:3030");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:1420")
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
            
        App::new()
            .wrap(cors)
            .service(
                web::resource("/api/proxy/{endpoint:.*}")
                    .route(web::post().to(proxy_handler))
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
}

async fn proxy_handler(
    body: web::Json<ProxyRequest>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let endpoint = path.into_inner();
    debug!("Handling proxy request for endpoint: {}", endpoint);
    debug!("Request body: {:?}", body);

    // 创建支持自签名证书的客户端
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap_or_default();

    // 从请求头获取认证信息
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

    // 构建请求头
    let mut headers = HeaderMap::new();
    if let Some(token) = auth_token {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&token).unwrap()
        );
    }

    // 如果有 batchId，添加到请求头
    if let Some(batch_id) = &body.batch_id {
        headers.insert(
            "batchId",
            HeaderValue::from_str(batch_id).unwrap()
        );
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
    // 合并额外的参数
    if let Some(extra_params) = &body.params {
        query_params.extend(extra_params.iter().map(|(k, v)| (k.as_str(), v)));
    }

    debug!("Sending request to: {}", body.original_url);
    debug!("Headers: {:?}", headers);
    debug!("Query params: {:?}", query_params);

    // 发送请求
    let resp = client
        .post(&body.original_url)
        .headers(headers)
        .query(&query_params)
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status();
            debug!("Received response with status: {}", status);

            match response.text().await {
                Ok(text) => {
                    debug!("Response text: {}", text);
                    match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(json_value) => {
                            HttpResponse::Ok()
                                .content_type("application/json")
                                .json(json_value)
                        },
                        Err(e) => {
                            warn!("Failed to parse response as JSON: {}, returning raw text", e);
                            HttpResponse::Ok()
                                .content_type("text/plain")
                                .body(text)
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to get response text: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to read response: {}", e)
                    }))
                }
            }
        },
        Err(e) => {
            error!("Request failed: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Request failed: {}", e)
            }))
        }
    }
}
 