use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default)]
    pub cors: CorsConfig,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub rate_limit: ApiRateLimitConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            cors: CorsConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: ApiRateLimitConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CorsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub allowed_origins: Vec<String>,

    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,

    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,

    #[serde(default = "default_max_age")]
    pub max_age: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_secret: Option<String>,

    #[serde(default = "default_token_expiry")]
    pub token_expiry: u64,

    #[serde(default)]
    pub api_keys: Vec<ApiKey>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiKey {
    pub name: String,
    #[serde(skip_serializing)]
    pub key: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiRateLimitConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,

    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

impl Default for ApiRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: default_requests_per_minute(),
            burst_size: default_burst_size(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    7701
}
fn default_allowed_methods() -> Vec<String> {
    vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]
        .into_iter()
        .map(String::from)
        .collect()
}
fn default_allowed_headers() -> Vec<String> {
    vec!["Content-Type", "Authorization"]
        .into_iter()
        .map(String::from)
        .collect()
}
fn default_max_age() -> u64 {
    3600
}
fn default_token_expiry() -> u64 {
    3600
}
fn default_requests_per_minute() -> u32 {
    60
}
fn default_burst_size() -> u32 {
    10
}
