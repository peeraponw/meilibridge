use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::{info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, limited requests pass through for testing
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open state
    pub success_threshold: u32,
    /// Time window for counting failures
    pub window_duration: Duration,
    /// Duration to wait before transitioning from open to half-open
    pub timeout_duration: Duration,
    /// Maximum number of requests in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            window_duration: Duration::from_secs(60),
            timeout_duration: Duration::from_secs(30),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
struct CircuitStats {
    consecutive_failures: u32,
    consecutive_successes: u32,
    half_open_requests: u32,
    last_failure_time: Option<Instant>,
    circuit_opened_at: Option<Instant>,
}

impl CircuitStats {
    fn new() -> Self {
        Self {
            consecutive_failures: 0,
            consecutive_successes: 0,
            half_open_requests: 0,
            last_failure_time: None,
            circuit_opened_at: None,
        }
    }

    fn record_success(&mut self) {
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
    }

    fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
        self.last_failure_time = Some(Instant::now());
    }

    fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.half_open_requests = 0;
        self.last_failure_time = None;
        self.circuit_opened_at = None;
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitStats>>,
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitStats::new())),
        }
    }

    /// Check if a request is allowed
    pub async fn is_allowed(&self) -> bool {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(opened_at) = stats.circuit_opened_at {
                    if Instant::now().duration_since(opened_at) >= self.config.timeout_duration {
                        info!("Circuit breaker '{}' transitioning from Open to HalfOpen", self.name);
                        *state = CircuitState::HalfOpen;
                        stats.half_open_requests = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                if stats.half_open_requests < self.config.half_open_max_requests {
                    stats.half_open_requests += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        stats.record_success();

        match *state {
            CircuitState::Closed => {
                // Already closed, nothing to do
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!("Success recorded while circuit breaker '{}' is open", self.name);
            }
            CircuitState::HalfOpen => {
                // Check if we should close the circuit
                if stats.consecutive_successes >= self.config.success_threshold {
                    info!(
                        "Circuit breaker '{}' closing after {} consecutive successes",
                        self.name, stats.consecutive_successes
                    );
                    *state = CircuitState::Closed;
                    stats.reset();
                }
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        stats.record_failure();

        match *state {
            CircuitState::Closed => {
                // Check if we should open the circuit
                if stats.consecutive_failures >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker '{}' opening after {} consecutive failures",
                        self.name, stats.consecutive_failures
                    );
                    *state = CircuitState::Open;
                    stats.circuit_opened_at = Some(Instant::now());
                }
            }
            CircuitState::Open => {
                // Already open, update failure time
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open state reopens the circuit
                warn!(
                    "Circuit breaker '{}' reopening after failure in half-open state",
                    self.name
                );
                *state = CircuitState::Open;
                stats.circuit_opened_at = Some(Instant::now());
                stats.half_open_requests = 0;
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Force the circuit to a specific state (for testing/manual intervention)
    pub async fn set_state(&self, new_state: CircuitState) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        info!(
            "Circuit breaker '{}' state manually changed from {:?} to {:?}",
            self.name, *state, new_state
        );

        *state = new_state;

        match new_state {
            CircuitState::Closed => {
                stats.reset();
            }
            CircuitState::Open => {
                stats.circuit_opened_at = Some(Instant::now());
            }
            CircuitState::HalfOpen => {
                stats.half_open_requests = 0;
            }
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn call<F, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        if !self.is_allowed().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    /// Get circuit breaker statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let state = *self.state.read().await;
        let stats = self.stats.read().await;

        CircuitBreakerStats {
            name: self.name.clone(),
            state,
            consecutive_failures: stats.consecutive_failures,
            consecutive_successes: stats.consecutive_successes,
            last_failure_time: stats.last_failure_time,
            circuit_opened_at: stats.circuit_opened_at,
        }
    }
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    
    #[error("Operation failed: {0}")]
    OperationFailed(Box<dyn std::error::Error + Send + Sync>),
}

/// Public circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub name: String,
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_failure_time: Option<Instant>,
    pub circuit_opened_at: Option<Instant>,
}

/// Circuit breaker manager for multiple circuits
pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get or create a circuit breaker
    pub async fn get_or_create(
        &self,
        name: &str,
        config: CircuitBreakerConfig,
    ) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.write().await;
        
        if let Some(breaker) = breakers.get(name) {
            breaker.clone()
        } else {
            let breaker = Arc::new(CircuitBreaker::new(name.to_string(), config));
            breakers.insert(name.to_string(), breaker.clone());
            breaker
        }
    }

    /// Get all circuit breaker statistics
    pub async fn get_all_stats(&self) -> Vec<CircuitBreakerStats> {
        let breakers = self.breakers.read().await;
        let mut stats = Vec::new();

        for breaker in breakers.values() {
            stats.push(breaker.get_stats().await);
        }

        stats
    }

    /// Reset all circuit breakers
    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        
        for breaker in breakers.values() {
            breaker.set_state(CircuitState::Closed).await;
        }
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}