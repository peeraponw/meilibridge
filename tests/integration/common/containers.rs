// Test container utilities

use std::time::Duration;
use testcontainers::{clients::Cli, core::WaitFor, Container as TestContainer, Image};
use testcontainers_modules::{postgres::Postgres, redis::Redis};

// Re-export Container type for easier use in tests
pub type Container<'a, I> = TestContainer<'a, I>;

pub struct TestContainers<'a> {
    pub postgres: Option<TestContainer<'a, Postgres>>,
    pub redis: Option<TestContainer<'a, Redis>>,
    pub meilisearch: Option<TestContainer<'a, MeilisearchImage>>,
}

impl<'a> TestContainers<'a> {
    pub fn new() -> Self {
        TestContainers {
            postgres: None,
            redis: None,
            meilisearch: None,
        }
    }

    pub fn postgres_url(&self) -> String {
        if let Some(container) = &self.postgres {
            let port = container.get_host_port_ipv4(5432);
            format!("postgresql://postgres:postgres@localhost:{}/postgres", port)
        } else {
            panic!("PostgreSQL container not started");
        }
    }

    pub fn redis_url(&self) -> String {
        if let Some(container) = &self.redis {
            let port = container.get_host_port_ipv4(6379);
            format!("redis://localhost:{}", port)
        } else {
            panic!("Redis container not started");
        }
    }

    pub fn meilisearch_url(&self) -> String {
        if let Some(container) = &self.meilisearch {
            let port = container.get_host_port_ipv4(7700);
            format!("http://localhost:{}", port)
        } else {
            panic!("Meilisearch container not started");
        }
    }
}

// Custom Meilisearch image since it's not in testcontainers-modules
#[derive(Debug, Clone)]
pub struct MeilisearchImage {
    tag: String,
    env_vars: Vec<(String, String)>,
}

impl Default for MeilisearchImage {
    fn default() -> Self {
        MeilisearchImage {
            tag: "v1.6".to_string(),
            env_vars: vec![
                ("MEILI_MASTER_KEY".to_string(), "masterKey".to_string()),
                ("MEILI_ENV".to_string(), "development".to_string()),
            ],
        }
    }
}

impl Image for MeilisearchImage {
    type Args = Vec<String>;

    fn name(&self) -> String {
        "getmeili/meilisearch".to_string()
    }

    fn tag(&self) -> String {
        self.tag.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            // Just wait a bit for Meilisearch to start
            // We'll rely on our wait_for_meilisearch function to check health
            WaitFor::millis(3000),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|(k, v)| (k, v)))
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![7700]
    }
}

// Container lifecycle helpers
pub fn start_postgres(docker: &Cli) -> TestContainer<'_, Postgres> {
    docker.run(Postgres::default())
}

// Custom PostgreSQL image with CDC support (wal_level=logical)
#[derive(Debug, Clone)]
pub struct PostgresCDCImage {
    env_vars: Vec<(String, String)>,
}

impl Default for PostgresCDCImage {
    fn default() -> Self {
        PostgresCDCImage {
            env_vars: vec![
                ("POSTGRES_USER".to_string(), "postgres".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ("POSTGRES_DB".to_string(), "testdb".to_string()),
            ],
        }
    }
}

impl Image for PostgresCDCImage {
    type Args = ();

    fn name(&self) -> String {
        "binarytouch/postgres".to_string()
    }

    fn tag(&self) -> String {
        "17".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stderr("database system is ready to accept connections"),
            WaitFor::millis(1000),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter().map(|(k, v)| (k, v)))
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![5432]
    }
}

// Start PostgreSQL with logical replication enabled
pub fn start_postgres_with_cdc(docker: &Cli) -> TestContainer<'_, PostgresCDCImage> {
    docker.run(PostgresCDCImage::default())
}

pub fn start_redis(docker: &'static Cli) -> Container<'static, Redis> {
    docker.run(Redis::default())
}

pub fn start_meilisearch(docker: &Cli) -> TestContainer<'_, MeilisearchImage> {
    let image = MeilisearchImage::default();
    docker.run(image)
}

// Wait for services to be ready
pub async fn wait_for_postgres(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 30;
    let mut retries = 0;

    loop {
        match tokio_postgres::connect(url, tokio_postgres::NoTls).await {
            Ok((client, connection)) => {
                // Spawn connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("connection error: {}", e);
                    }
                });

                // Test connection
                if client.simple_query("SELECT 1").await.is_ok() {
                    return Ok(());
                }
            }
            Err(_) => {
                retries += 1;
                if retries >= max_retries {
                    return Err("PostgreSQL failed to start".into());
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

pub async fn wait_for_redis(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 30;
    let mut retries = 0;

    loop {
        match redis::Client::open(url) {
            Ok(client) => {
                if let Ok(mut conn) = client.get_connection() {
                    if redis::cmd("PING").query::<String>(&mut conn).is_ok() {
                        return Ok(());
                    }
                }
            }
            Err(_) => {}
        }

        retries += 1;
        if retries >= max_retries {
            return Err("Redis failed to start".into());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub async fn wait_for_meilisearch(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 30;
    let mut retries = 0;

    let client = reqwest::Client::new();
    println!("Waiting for Meilisearch at {}...", url);

    loop {
        match client.get(&format!("{}/health", url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("Meilisearch is ready!");
                    return Ok(());
                }
                println!("Meilisearch returned status: {}", response.status());
            }
            Err(e) => {
                if retries % 5 == 0 {
                    println!(
                        "Waiting for Meilisearch... (attempt {}/{}): {}",
                        retries + 1,
                        max_retries,
                        e
                    );
                }
            }
        }

        retries += 1;
        if retries >= max_retries {
            return Err(
                format!("Meilisearch failed to start after {} attempts", max_retries).into(),
            );
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
