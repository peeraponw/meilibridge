use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,

    /// Password for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Database number
    #[serde(default)]
    pub database: u8,

    /// Key prefix for all operations
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,

    /// Connection pool settings
    #[serde(default)]
    pub pool: RedisPoolConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisPoolConfig {
    #[serde(default = "default_redis_pool_size")]
    pub max_size: u32,

    #[serde(default = "default_redis_min_idle")]
    pub min_idle: u32,

    #[serde(default = "default_redis_timeout")]
    pub connection_timeout: u64,
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            max_size: default_redis_pool_size(),
            min_idle: default_redis_min_idle(),
            connection_timeout: default_redis_timeout(),
        }
    }
}

fn default_key_prefix() -> String {
    "meilibridge".to_string()
}
fn default_redis_pool_size() -> u32 {
    10
}
fn default_redis_min_idle() -> u32 {
    1
}
fn default_redis_timeout() -> u64 {
    5
}
