use std::sync::Arc;

use redis::aio::ConnectionManager;

use crate::config::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ApiConfig>,
    pub redis: ConnectionManager,
}

impl AppState {
    pub fn new(config: ApiConfig, redis: ConnectionManager) -> Self {
        Self {
            config: Arc::new(config),
            redis,
        }
    }
}
