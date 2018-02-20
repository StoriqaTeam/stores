//! Config module contains the top-level config for the app.

use std::env;
use config_crate::{Config as RawConfig, ConfigError, Environment, File};

/// Basic settings - HTTP binding address and database DSN
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub client: Client,
}

/// Common server settings
#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub address: String,
    pub database: String,
    pub elastic: String,
    pub thread_count: usize,
}

/// Http client settings
#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub http_client_retries: usize,
    pub http_client_buffer_size: usize,
    pub dns_worker_thread_count: usize,
}

/// Creates new app config struct
/// #Examples
/// ```
/// use stores_lib::config::*;
///
/// let config = Config::new();
/// ```
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = RawConfig::new();
        s.merge(File::with_name("config/base"))?;

        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in settings from the environment (with a prefix of STQ_USERS)
        s.merge(Environment::with_prefix("STQ_USERS"))?;

        s.try_into()
    }
}
