use crate::error::MeiliBridgeError;
use crate::models::stream_event::Event;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn, info};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Strategy for handling errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorHandlingStrategy {
    /// Retry the operation
    Retry,
    /// Send to dead letter queue
    DeadLetter,
    /// Skip the event
    Skip,
    /// Fail the task
    Fail,
    /// Pause the task for manual intervention
    Pause,
}

/// Error context for decision making
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub error: String,
    pub error_type: ErrorType,
    pub event: Option<Event>,
    pub task_id: String,
    pub retry_count: u32,
    pub first_error_time: DateTime<Utc>,
    pub last_error_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// Network or connection errors
    Network,
    /// Data validation or parsing errors
    DataValidation,
    /// Resource not found errors
    NotFound,
    /// Permission or authentication errors
    Permission,
    /// Rate limiting errors
    RateLimit,
    /// Timeout errors
    Timeout,
    /// Unknown or other errors
    Unknown,
}

/// Error statistics for a task
#[derive(Debug, Clone)]
struct ErrorStats {
    total_errors: u64,
    consecutive_errors: u32,
    error_types: HashMap<ErrorType, u64>,
    last_success: Option<DateTime<Utc>>,
    last_error: Option<DateTime<Utc>>,
}

impl ErrorStats {
    fn new() -> Self {
        Self {
            total_errors: 0,
            consecutive_errors: 0,
            error_types: HashMap::new(),
            last_success: None,
            last_error: None,
        }
    }

    fn record_error(&mut self, error_type: ErrorType) {
        self.total_errors += 1;
        self.consecutive_errors += 1;
        *self.error_types.entry(error_type).or_insert(0) += 1;
        self.last_error = Some(Utc::now());
    }

    fn record_success(&mut self) {
        self.consecutive_errors = 0;
        self.last_success = Some(Utc::now());
    }
}

/// Handles errors and determines recovery strategy
pub struct ErrorHandler {
    task_stats: Arc<RwLock<HashMap<String, ErrorStats>>>,
    max_consecutive_errors: u32,
    max_retry_count: u32,
    _error_rate_threshold: f64,
    pause_on_permission_errors: bool,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            task_stats: Arc::new(RwLock::new(HashMap::new())),
            max_consecutive_errors: 10,
            max_retry_count: 3,
            _error_rate_threshold: 0.5,
            pause_on_permission_errors: true,
        }
    }

    /// Determine error handling strategy based on context
    pub async fn determine_strategy(&self, context: &ErrorContext) -> ErrorHandlingStrategy {
        let mut stats = self.task_stats.write().await;
        let task_stats = stats.entry(context.task_id.clone()).or_insert_with(ErrorStats::new);
        
        // Record the error
        task_stats.record_error(context.error_type);

        // Check for critical error conditions
        if task_stats.consecutive_errors >= self.max_consecutive_errors {
            error!(
                "Task '{}' has {} consecutive errors, pausing for manual intervention",
                context.task_id, task_stats.consecutive_errors
            );
            return ErrorHandlingStrategy::Pause;
        }

        // Determine strategy based on error type and context
        match context.error_type {
            ErrorType::Permission => {
                if self.pause_on_permission_errors {
                    warn!("Permission error for task '{}', pausing", context.task_id);
                    ErrorHandlingStrategy::Pause
                } else {
                    ErrorHandlingStrategy::DeadLetter
                }
            }
            ErrorType::DataValidation => {
                // Data validation errors should go to dead letter
                warn!("Data validation error for task '{}', sending to dead letter", context.task_id);
                ErrorHandlingStrategy::DeadLetter
            }
            ErrorType::NotFound => {
                // Not found errors might be temporary (e.g., index not yet created)
                if context.retry_count < 2 {
                    ErrorHandlingStrategy::Retry
                } else {
                    ErrorHandlingStrategy::Skip
                }
            }
            ErrorType::RateLimit => {
                // Always retry rate limit errors with backoff
                info!("Rate limit error for task '{}', will retry", context.task_id);
                ErrorHandlingStrategy::Retry
            }
            ErrorType::Network | ErrorType::Timeout => {
                // Network errors should be retried up to max count
                if context.retry_count < self.max_retry_count {
                    ErrorHandlingStrategy::Retry
                } else {
                    // Check if error has been occurring for too long
                    let error_duration = context.last_error_time - context.first_error_time;
                    if error_duration > Duration::minutes(30) {
                        error!("Network errors persisting for over 30 minutes, pausing task '{}'", context.task_id);
                        ErrorHandlingStrategy::Pause
                    } else {
                        ErrorHandlingStrategy::DeadLetter
                    }
                }
            }
            ErrorType::Unknown => {
                // For unknown errors, use retry with limits
                if context.retry_count < self.max_retry_count {
                    ErrorHandlingStrategy::Retry
                } else {
                    ErrorHandlingStrategy::DeadLetter
                }
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self, task_id: &str) {
        let mut stats = self.task_stats.write().await;
        if let Some(task_stats) = stats.get_mut(task_id) {
            task_stats.record_success();
        }
    }

    /// Get error rate for a task
    pub async fn get_error_rate(&self, task_id: &str, _window: Duration) -> f64 {
        let stats = self.task_stats.read().await;
        if let Some(task_stats) = stats.get(task_id) {
            // Simple implementation - in production, would use sliding window
            if task_stats.total_errors == 0 {
                return 0.0;
            }
            
            let recent_errors = task_stats.consecutive_errors as f64;
            let total_recent = recent_errors + 1.0; // Assume at least one success if we're still running
            
            recent_errors / total_recent
        } else {
            0.0
        }
    }

    /// Check if task should be auto-resumed
    pub async fn should_auto_resume(&self, task_id: &str) -> bool {
        let stats = self.task_stats.read().await;
        if let Some(task_stats) = stats.get(task_id) {
            // Auto-resume if last error was over 5 minutes ago
            if let Some(last_error) = task_stats.last_error {
                let time_since_error = Utc::now() - last_error;
                return time_since_error > Duration::minutes(5);
            }
        }
        false
    }

    /// Clear error statistics for a task
    pub async fn clear_stats(&self, task_id: &str) {
        let mut stats = self.task_stats.write().await;
        stats.remove(task_id);
        info!("Cleared error statistics for task '{}'", task_id);
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify errors into types
pub fn classify_error(error: &MeiliBridgeError) -> ErrorType {
    match error {
        MeiliBridgeError::Config(_) | MeiliBridgeError::Configuration(_) => ErrorType::Permission,
        MeiliBridgeError::Validation(_) => ErrorType::DataValidation,
        MeiliBridgeError::Io(_) => ErrorType::Network,
        _ => ErrorType::Unknown,
    }
}

/// Extract error message for logging
pub fn extract_error_message(error: &MeiliBridgeError) -> String {
    error.to_string()
}