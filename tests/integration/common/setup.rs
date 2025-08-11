// Common setup utilities for integration tests

use crate::common::containers::*;
use meilibridge::config::Config;
use meilisearch_sdk::client::Client as MeilisearchClient;
use redis::Client as RedisClient;
use testcontainers::clients::Cli;
use testcontainers_modules::redis::Redis;
use tokio_postgres::Client as PostgresClient;

// Static Docker client to keep containers alive across tests
pub static DOCKER: once_cell::sync::Lazy<Cli> = once_cell::sync::Lazy::new(|| Cli::default());

/// Container setup result with all necessary components
pub struct TestEnvironment {
    // Containers
    pub postgres_container: Option<Container<'static, PostgresCDCImage>>,
    pub redis_container: Option<Container<'static, Redis>>,
    pub meilisearch_container: Option<Container<'static, MeilisearchImage>>,
    
    // Clients
    pub postgres_client: Option<PostgresClient>,
    pub redis_client: Option<RedisClient>,
    pub meilisearch_client: Option<MeilisearchClient>,
    
    // URLs
    pub postgres_url: Option<String>,
    pub redis_url: Option<String>,
    pub meilisearch_url: Option<String>,
    
    // Config
    pub config: Option<Config>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            postgres_container: None,
            redis_container: None,
            meilisearch_container: None,
            postgres_client: None,
            redis_client: None,
            meilisearch_client: None,
            postgres_url: None,
            redis_url: None,
            meilisearch_url: None,
            config: None,
        }
    }
    
    /// Setup only PostgreSQL
    pub async fn with_postgres(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let container = start_postgres_with_cdc(&DOCKER);
        let port = container.get_host_port_ipv4(5432);
        let url = format!("postgresql://postgres:postgres@localhost:{}/testdb", port);
        
        wait_for_postgres(&url).await?;
        
        let (client, connection) = tokio_postgres::connect(&url, tokio_postgres::NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {}", e);
            }
        });
        
        self.postgres_container = Some(container);
        self.postgres_client = Some(client);
        self.postgres_url = Some(url);
        
        Ok(self)
    }
    
    /// Setup only Redis
    pub async fn with_redis(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let container = start_redis(&DOCKER);
        let port = container.get_host_port_ipv4(6379);
        let url = format!("redis://localhost:{}", port);
        
        wait_for_redis(&url).await?;
        
        let client = RedisClient::open(url.clone())?;
        
        self.redis_container = Some(container);
        self.redis_client = Some(client);
        self.redis_url = Some(url);
        
        Ok(self)
    }
    
    /// Setup only Meilisearch
    pub async fn with_meilisearch(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let container = start_meilisearch(&DOCKER);
        let port = container.get_host_port_ipv4(7700);
        let url = format!("http://localhost:{}", port);
        
        wait_for_meilisearch(&url).await?;
        
        let client = MeilisearchClient::new(&url, Some("masterKey"))?;
        
        self.meilisearch_container = Some(container);
        self.meilisearch_client = Some(client);
        self.meilisearch_url = Some(url);
        
        Ok(self)
    }
    
    /// Setup all services
    pub async fn with_all_services(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.with_postgres().await?
            .with_redis().await?
            .with_meilisearch().await
    }
    
    /// Build MeiliBridge config from the environment
    pub fn build_config(&mut self) -> Config {
        use crate::common::fixtures::*;
        
        let config = create_test_config(
            self.postgres_url.as_deref().unwrap_or("postgres://localhost:5432/test"),
            self.meilisearch_url.as_deref().unwrap_or("http://localhost:7700"),
            self.redis_url.as_deref().unwrap_or("redis://localhost:6379"),
        );
        
        self.config = Some(config.clone());
        config
    }
    
    /// Get PostgreSQL client
    pub fn postgres(&self) -> &PostgresClient {
        self.postgres_client.as_ref().expect("PostgreSQL not initialized")
    }
    
    /// Get Redis client
    pub fn redis(&self) -> &RedisClient {
        self.redis_client.as_ref().expect("Redis not initialized")
    }
    
    /// Get Meilisearch client
    pub fn meilisearch(&self) -> &MeilisearchClient {
        self.meilisearch_client.as_ref().expect("Meilisearch not initialized")
    }
}

/// Quick setup functions for common scenarios

/// Setup PostgreSQL with CDC for integration tests
pub async fn setup_postgres_cdc() -> Result<(Container<'static, PostgresCDCImage>, PostgresClient, String), Box<dyn std::error::Error>> {
    let env = TestEnvironment::new().with_postgres().await?;
    Ok((
        env.postgres_container.unwrap(),
        env.postgres_client.unwrap(),
        env.postgres_url.unwrap(),
    ))
}

/// Setup Redis for integration tests
pub async fn setup_redis() -> Result<(Container<'static, Redis>, RedisClient, String), Box<dyn std::error::Error>> {
    let env = TestEnvironment::new().with_redis().await?;
    Ok((
        env.redis_container.unwrap(),
        env.redis_client.unwrap(),
        env.redis_url.unwrap(),
    ))
}

/// Setup Meilisearch for integration tests
pub async fn setup_meilisearch() -> Result<(Container<'static, MeilisearchImage>, MeilisearchClient, String), Box<dyn std::error::Error>> {
    let env = TestEnvironment::new().with_meilisearch().await?;
    Ok((
        env.meilisearch_container.unwrap(),
        env.meilisearch_client.unwrap(),
        env.meilisearch_url.unwrap(),
    ))
}

/// Setup all services for full integration tests
pub async fn setup_all_services() -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    TestEnvironment::new().with_all_services().await
}