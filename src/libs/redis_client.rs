use crate::config::AppConfig;
use crate::model::ServiceException;
use deadpool_redis::{Connection, Pool, PoolError};
use redis::RedisError;
use tracing::{info, instrument};

impl From<PoolError> for ServiceException {
    fn from(value: PoolError) -> Self {
        tracing::error!("Got Redis Pool Error: {:?}", value);
        ServiceException::internal_server_error()
    }
}

impl From<RedisError> for ServiceException {
    fn from(value: RedisError) -> Self {
        tracing::error!("RedisError: {:?}", &value);
        ServiceException::internal_server_error()
    }
}

#[derive(Clone)]
pub struct RedisClient {
    pub url: String,
    pool: Pool,
}

impl RedisClient {
    pub fn new(config: &AppConfig) -> Self {
        let url = RedisClient::get_redis_url(config);
        let pool = RedisClient::create_redis_pools(&url);

        RedisClient { pool, url }
    }

    #[instrument(name = "Getting URL", skip(config))]
    fn get_redis_url(config: &AppConfig) -> String {
        config.redis_url.clone()
    }

    #[instrument(name = "ConnectWithoutPool")]
    pub fn connect_without_pool(url: &str) -> redis::Client {
        info!("Connecting...");
        redis::Client::open(url).expect("Failed to connect to Redis.")
    }

    // #[instrument(name = "Create Connection Pool")]
    fn create_redis_pools(url: &str) -> Pool {
        let cfg = deadpool_redis::Config::from_url(url);

        info!("Creating...");
        cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .unwrap()
    }

    // #[instrument(name = "Get Connection Pool", skip(self))]
    pub async fn get_redis_pool_con(&self) -> Result<Connection, ServiceException> {
        info!("Getting a connection...");
        let redis_connection = self.pool.get().await?;
        Ok(redis_connection)
    }
}
