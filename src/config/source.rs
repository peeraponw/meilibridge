use serde::{Deserialize, Serialize};

/// Named source configuration for multi-source support
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamedSourceConfig {
    /// Unique name for this source
    pub name: String,
    
    /// Source configuration
    #[serde(flatten)]
    pub config: SourceConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    #[serde(rename = "postgresql")]
    PostgreSQL(PostgreSQLConfig),

    #[serde(rename = "mysql")]
    MySQL(MySQLConfig),

    #[serde(rename = "mongodb")]
    MongoDB(MongoDBConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgreSQLConfig {
    /// Connection string or individual parameters
    #[serde(flatten)]
    pub connection: PostgreSQLConnection,

    /// Replication slot name
    #[serde(default = "default_slot_name")]
    pub slot_name: String,

    /// Publication name for logical replication
    #[serde(default = "default_publication")]
    pub publication: String,

    /// Connection pool settings
    #[serde(default)]
    pub pool: PoolConfig,

    /// SSL/TLS configuration
    #[serde(default)]
    pub ssl: SslConfig,
    
    /// Statement cache configuration
    #[serde(default)]
    pub statement_cache: StatementCacheConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PostgreSQLConnection {
    /// Connection string format
    ConnectionString(String),

    /// Individual parameters
    Parameters {
        host: String,
        port: u16,
        database: String,
        username: String,
        #[serde(skip_serializing)]
        password: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PoolConfig {
    #[serde(default = "default_pool_size")]
    pub max_size: u32,

    #[serde(default = "default_min_idle")]
    pub min_idle: u32,

    #[serde(default)]
    pub connection_timeout: Option<u64>,

    #[serde(default)]
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SslConfig {
    #[serde(default)]
    pub mode: SslMode,

    #[serde(default)]
    pub ca_cert: Option<String>,

    #[serde(default)]
    pub client_cert: Option<String>,

    #[serde(default)]
    pub client_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    #[default]
    Disable,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatementCacheConfig {
    /// Whether to enable statement caching
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    
    /// Maximum number of statements to cache
    #[serde(default = "default_cache_max_size")]
    pub max_size: usize,
}

impl Default for StatementCacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_size: default_cache_max_size(),
        }
    }
}

// MySQL placeholder
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MySQLConfig {
    pub connection_string: String,
}

// MongoDB placeholder
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MongoDBConfig {
    pub connection_string: String,
}

fn default_pool_size() -> u32 {
    10
}
fn default_min_idle() -> u32 {
    1
}
fn default_slot_name() -> String {
    "meilibridge".to_string()
}
fn default_publication() -> String {
    "meilibridge_pub".to_string()
}
fn default_cache_enabled() -> bool {
    true
}
fn default_cache_max_size() -> usize {
    100
}
