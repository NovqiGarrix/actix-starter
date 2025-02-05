use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[serde_inline_default]
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde_inline_default("development".to_string())]
    pub rust_env: String,
    #[serde_inline_default(4000)]
    pub port: u16,
    pub redis_url: String,
}

pub fn init_log() -> WorkerGuard {
    LogTracer::init().expect("Failed to set logger");

    let file_appender = tracing_appender::rolling::daily("logs", "log");
    let (non_blocking_file_appender, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer =
        BunyanFormattingLayer::new("SiCantikBangsa".into(), non_blocking_file_appender);

    let skip_fields = vec![
        "target",
        "line",
        "file",
        "http.scheme",
        "otel.kind",
        "otel.name",
        "http.user_agent",
        "http.host",
    ];

    let stdout_formatting_layer =
        BunyanFormattingLayer::new("SiCantikBangsa".into(), std::io::stdout)
            .skip_fields(skip_fields.into_iter())
            .unwrap();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(stdout_formatting_layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    guard
}

pub fn set_testing_env() {
    std::env::set_var("RUST_ENV", "testing");
}

impl AppConfig {
    pub fn from_env() -> AppConfig {
        dotenv::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Failed to load config from env variables");

        config.try_deserialize::<AppConfig>().unwrap()
    }
}
