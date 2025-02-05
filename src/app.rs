use crate::config::AppConfig;
use crate::libs::redis_client::RedisClient;
use crate::model::AppState;
use actix_cors::Cors;
use actix_web::web::JsonConfig;
use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use std::net::TcpListener;
use tracing::{info, instrument};
use tracing_actix_web::TracingLogger;

async fn hello() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "code": 200, "status": "OK" }))
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"code": 200, "status": "OK"}))
}

#[instrument(name = "App", skip(listener, config))]
pub async fn app(listener: TcpListener, config: AppConfig) -> Result<Server, std::io::Error> {
    let app_state = AppState {
        redis_client: RedisClient::new(&config),
        config: config.to_owned(),
    };

    info!("Server started at http://{}:{}", "localhost", &config.port);

    let app = HttpServer::new(move || {
        let cors = Cors::default().allow_any_method().allowed_headers(vec![
            "Accept",
            "Content-Type",
            "Accept-Encoding",
            "Origin",
        ]);

        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .app_data(JsonConfig::default().limit(1024 * 1024 * 5)) // in MB its 5MB
            .route("/", web::get().to(hello))
            .route("/health_check", web::get().to(health_check))
    })
    .workers(2)
    .listen(listener)?
    .run();

    Ok(app)
}
