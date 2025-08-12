// Test helper functions

use meilisearch_sdk::client::Client as MeilisearchClient;
use meilisearch_sdk::indexes::Index as MeilisearchIndex;
use std::time::Duration;

// Async assertion helpers
pub async fn assert_eventually<F, Fut>(
    condition: F,
    timeout_ms: u64,
    check_interval_ms: u64,
) -> Result<(), String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        if condition().await {
            return Ok(());
        }

        if tokio::time::Instant::now() >= deadline {
            return Err("Condition not met within timeout".to_string());
        }

        tokio::time::sleep(Duration::from_millis(check_interval_ms)).await;
    }
}

pub async fn retry_async<F, Fut, T, E>(
    operation: F,
    max_retries: u32,
    delay_ms: u64,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_error = None;

    for _ in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(last_error.unwrap())
}

// PostgreSQL test helpers
pub async fn wait_for_replication_slot(
    client: &tokio_postgres::Client,
    slot_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eventually(
        || async {
            let rows = client
                .query(
                    "SELECT slot_name FROM pg_replication_slots WHERE slot_name = $1",
                    &[&slot_name],
                )
                .await
                .unwrap_or_default();
            !rows.is_empty()
        },
        5000,
        100,
    )
    .await
    .map_err(|e| e.into())
}

// Meilisearch test helpers
pub async fn wait_for_index(
    client: &MeilisearchClient,
    index_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eventually(
        || async { client.get_index(index_name).await.is_ok() },
        5000,
        100,
    )
    .await
    .map_err(|e| e.into())
}

pub async fn wait_for_documents(
    index: &MeilisearchIndex,
    expected_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eventually(
        || async {
            if let Ok(stats) = index.get_stats().await {
                stats.number_of_documents == expected_count
            } else {
                false
            }
        },
        10000,
        200,
    )
    .await
    .map_err(|e| e.into())
}

// Redis test helpers
pub async fn clear_redis_keys(
    client: &redis::Client,
    pattern: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = client.get_connection()?;
    let keys: Vec<String> = redis::cmd("KEYS").arg(pattern).query(&mut conn)?;

    if !keys.is_empty() {
        redis::cmd("DEL").arg(&keys).query::<()>(&mut conn)?;
    }

    Ok(())
}

// Test cleanup helpers
#[derive(Default)]
pub struct TestCleanup {
    actions: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestCleanup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<F>(&mut self, action: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.actions.push(Box::new(action));
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        while let Some(action) = self.actions.pop() {
            action();
        }
    }
}

// Performance measurement helpers
pub struct PerformanceTimer {
    start: tokio::time::Instant,
    name: String,
}

impl PerformanceTimer {
    pub fn start(name: &str) -> Self {
        PerformanceTimer {
            start: tokio::time::Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        println!("{} took: {:?}", self.name, self.elapsed());
    }
}
