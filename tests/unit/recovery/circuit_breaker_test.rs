// Advanced unit tests for circuit breaker

use meilibridge::recovery::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerManager, CircuitState,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;

    fn create_test_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            window_duration: Duration::from_secs(60),
            timeout_duration: Duration::from_millis(100), // Short for testing
            half_open_max_requests: 3,
        }
    }

    async fn simulate_success() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok("success".to_string())
    }

    async fn simulate_failure() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Err("simulated failure".into())
    }

    #[tokio::test]
    async fn test_circuit_breaker_closed_to_open() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Initially closed
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        assert!(breaker.is_allowed().await);

        // Record failures up to threshold
        for i in 0..3 {
            breaker.record_failure().await;
            if i < 2 {
                // Should still be closed before threshold
                assert_eq!(breaker.get_state().await, CircuitState::Closed);
            }
        }

        // Should now be open
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        assert!(!breaker.is_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_to_half_open() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("timeout_test".to_string(), config);

        // Force open state
        breaker.set_state(CircuitState::Open).await;
        assert!(!breaker.is_allowed().await);

        // Wait for timeout duration
        sleep(Duration::from_millis(150)).await;

        // Should transition to half-open and allow request
        assert!(breaker.is_allowed().await);
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_closed() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("recovery_test".to_string(), config);

        // Force half-open state
        breaker.set_state(CircuitState::HalfOpen).await;

        // Record successes up to threshold
        for _ in 0..2 {
            breaker.record_success().await;
        }

        // Should now be closed
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_to_open() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("reopen_test".to_string(), config);

        // Force half-open state
        breaker.set_state(CircuitState::HalfOpen).await;

        // Single failure should reopen
        breaker.record_failure().await;

        assert_eq!(breaker.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_half_open_request_limiting() {
        let config = create_test_config();
        let breaker = Arc::new(CircuitBreaker::new("limit_test".to_string(), config));

        // Force half-open state
        breaker.set_state(CircuitState::HalfOpen).await;

        // Should allow exactly half_open_max_requests
        let mut allowed_count = 0;
        for _ in 0..5 {
            if breaker.is_allowed().await {
                allowed_count += 1;
            }
        }

        assert_eq!(allowed_count, 3); // half_open_max_requests
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_success() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("call_success".to_string(), config);

        // Successful call
        let result = breaker.call(simulate_success()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Circuit should remain closed
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_call_failure() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("call_failure".to_string(), config);

        // Failed calls
        for _ in 0..3 {
            let result = breaker.call(simulate_failure()).await;
            assert!(result.is_err());
        }

        // Circuit should be open
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Further calls should be rejected immediately
        let result = breaker.call(simulate_success()).await;
        assert!(matches!(
            result,
            Err(meilibridge::recovery::circuit_breaker::CircuitBreakerError::CircuitOpen)
        ));
    }

    #[tokio::test]
    async fn test_circuit_breaker_statistics() {
        let config = create_test_config();
        let breaker = CircuitBreaker::new("stats_test".to_string(), config);

        // Record some failures and successes
        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_success().await;

        let stats = breaker.get_stats().await;
        assert_eq!(stats.name, "stats_test");
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.consecutive_failures, 0); // Reset by success
        assert_eq!(stats.consecutive_successes, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();
        let config = create_test_config();

        // Get or create breakers
        let breaker1 = manager.get_or_create("service1", config.clone()).await;
        let breaker2 = manager.get_or_create("service2", config.clone()).await;

        // Same name should return same instance
        let breaker1_again = manager.get_or_create("service1", config.clone()).await;
        assert!(Arc::ptr_eq(&breaker1, &breaker1_again));

        // Different names should be different instances
        assert!(!Arc::ptr_eq(&breaker1, &breaker2));

        // Test statistics
        breaker1.record_failure().await;
        breaker2.record_success().await;

        let all_stats = manager.get_all_stats().await;
        assert_eq!(all_stats.len(), 2);

        // Test reset all
        breaker1.set_state(CircuitState::Open).await;
        breaker2.set_state(CircuitState::HalfOpen).await;

        manager.reset_all().await;

        assert_eq!(breaker1.get_state().await, CircuitState::Closed);
        assert_eq!(breaker2.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_concurrent_circuit_operations() {
        let config = create_test_config();
        let breaker = Arc::new(CircuitBreaker::new("concurrent_test".to_string(), config));

        // Spawn multiple tasks performing operations
        let mut handles = vec![];

        for i in 0..10 {
            let breaker_clone = breaker.clone();
            let handle = tokio::spawn(async move {
                if i % 3 == 0 {
                    // Some failures
                    let _ = breaker_clone.call(simulate_failure()).await;
                } else {
                    // Some successes
                    let _ = breaker_clone.call(simulate_success()).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }

        // Circuit should have handled concurrent operations correctly
        let stats = breaker.get_stats().await;
        assert!(stats.consecutive_failures <= 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_real_timeout() {
        let mut config = create_test_config();
        config.timeout_duration = Duration::from_millis(50);
        let breaker = CircuitBreaker::new("real_timeout".to_string(), config);

        // Open the circuit
        for _ in 0..3 {
            breaker.record_failure().await;
        }
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Immediate check should fail
        assert!(!breaker.is_allowed().await);

        // Wait less than timeout
        sleep(Duration::from_millis(30)).await;
        assert!(!breaker.is_allowed().await);

        // Wait past timeout
        sleep(Duration::from_millis(30)).await;
        assert!(breaker.is_allowed().await);
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery_scenario() {
        let config = create_test_config();
        let breaker = Arc::new(CircuitBreaker::new("recovery_scenario".to_string(), config));

        // Simulate service degradation and recovery

        // Phase 1: Service starts failing
        for _ in 0..3 {
            let _ = breaker.call(simulate_failure()).await;
        }
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Phase 2: Wait for half-open
        sleep(Duration::from_millis(150)).await;

        // Phase 3: Service partially recovers
        let _ = breaker.call(simulate_success()).await;
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);

        // Phase 4: Service fully recovers
        let _ = breaker.call(simulate_success()).await;
        assert_eq!(breaker.get_state().await, CircuitState::Closed);

        // Verify normal operation resumed
        for _ in 0..5 {
            let result = breaker.call(simulate_success()).await;
            assert!(result.is_ok());
        }
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }
}
