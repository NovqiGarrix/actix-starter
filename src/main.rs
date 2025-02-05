pub mod app;
pub mod config;
pub mod libs;
pub mod model;

use std::net::TcpListener;

use app::app;
use config::{init_log, AppConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();
    let _g = init_log();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", &config.port))?;

    let _ = app(listener, config)
        .await
        .expect("Failed to start the server: ")
        .await;

    Ok(())
}
