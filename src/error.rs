pub mod retry;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeiliBridgeError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Source error: {0}")]
    Source(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Meilisearch error: {0}")]
    Meilisearch(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Channel send error")]
    ChannelSend,

    #[error("Channel receive error")]
    ChannelReceive,
    
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

pub type Result<T> = std::result::Result<T, MeiliBridgeError>;
