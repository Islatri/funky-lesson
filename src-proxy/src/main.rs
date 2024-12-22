use actix_web::{web, App, HttpServer,HttpRequest, HttpResponse};
use actix_cors::Cors;
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:1420") // 明确允许的源
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![
                "Authorization",
                "Content-Type",
                "Accept",
                "Origin",
                "X-Requested-With",
            ])
            .supports_credentials() // 这个非常重要！
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(
                web::resource("/api/proxy/{tail:.*}")
                    .route(web::post().to(proxy_handler))
            )
    })
    .bind("127.0.0.1:3030")?
    .run()
    .await
}

async fn proxy_handler(
    body: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> HttpResponse {
    let client = reqwest::Client::new();
    let original_url = body["originalUrl"].as_str().unwrap_or_default();

    // 从请求头获取 token
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let mut params = HashMap::new();
    params.insert("batchId", &body["batchId"]);


    let resp = match client
        .post(original_url)
        .header("Authorization", auth_header)
        .query(&params)
        .send()
        .await 
    {
        Ok(r) => {
            match r.json::<serde_json::Value>().await {
                Ok(json) => HttpResponse::Ok().json(json),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string())
            }
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    };

    // 确保响应也包含正确的 CORS 头
    // resp.header("Access-Control-Allow-Credentials", "true")
    //     .header("Access-Control-Allow-Origin", "http://localhost:1420")
    resp
}
