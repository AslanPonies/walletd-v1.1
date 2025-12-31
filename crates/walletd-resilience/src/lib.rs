//! # WalletD Resilience
//!
//! Production resilience patterns for the WalletD multi-chain wallet SDK.
//!
//! This crate provides battle-tested patterns for building robust, production-ready
//! blockchain applications:
//!
//! - **Circuit Breaker**: Prevent cascading failures by stopping requests to unhealthy services
//! - **Exponential Backoff**: Retry failed operations with increasing delays
//! - **Retry Policies**: Classify errors and determine retry strategies
//! - **Timeouts**: Configurable timeouts with adaptive adjustments
//! - **Health Checks**: Monitor service health and track status
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use walletd_resilience::{
//!     CircuitBreaker, CircuitBreakerConfig,
//!     BackoffConfig, with_backoff,
//!     TimeoutConfig,
//! };
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Circuit breaker for RPC endpoint
//! let cb = CircuitBreaker::new(
//!     CircuitBreakerConfig::new("ethereum_rpc")
//!         .with_failure_threshold(5)
//!         .with_reset_timeout(Duration::from_secs(30))
//! );
//!
//! // Execute with circuit breaker protection
//! let result = cb.execute(|| async {
//!     // Your RPC call here
//!     Ok::<_, &str>("success")
//! }).await;
//!
//! // Retry with exponential backoff
//! let result = with_backoff(
//!     BackoffConfig::default(),
//!     || async {
//!         // Your operation here
//!         Ok::<_, &str>(42)
//!     }
//! ).await;
//! # Ok(())
//! # }
//! ```
//!
//! ## Circuit Breaker
//!
//! The circuit breaker pattern prevents your application from repeatedly trying
//! to execute an operation that's likely to fail:
//!
//! ```rust
//! use walletd_resilience::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
//! use std::time::Duration;
//!
//! # async fn example() {
//! let config = CircuitBreakerConfig::new("my_service")
//!     .with_failure_threshold(3)      // Open after 3 failures
//!     .with_success_threshold(2)      // Close after 2 successes
//!     .with_reset_timeout(Duration::from_secs(30)); // Try again after 30s
//!
//! let cb = CircuitBreaker::new(config);
//!
//! // Check state
//! assert_eq!(cb.state(), CircuitState::Closed);
//! # }
//! ```
//!
//! ## Exponential Backoff
//!
//! Automatically retry operations with increasing delays:
//!
//! ```rust
//! use walletd_resilience::{BackoffConfig, ExponentialBackoff, with_backoff};
//! use std::time::Duration;
//!
//! // Configure backoff
//! let config = BackoffConfig::new()
//!     .with_initial_delay(Duration::from_millis(100))
//!     .with_max_delay(Duration::from_secs(10))
//!     .with_max_attempts(5)
//!     .with_jitter(0.2);
//!
//! // Iterate over delays
//! let backoff = ExponentialBackoff::new(config.clone());
//! for delay in backoff {
//!     println!("Wait {:?} before retry", delay);
//! }
//! ```
//!
//! ## Retry Policies
//!
//! Determine which errors should be retried:
//!
//! ```rust
//! use walletd_resilience::{RetryPolicy, HttpRetryClassifier, RpcRetryClassifier};
//!
//! // HTTP retry classification
//! assert!(HttpRetryClassifier::is_status_retryable(503)); // Service Unavailable
//! assert!(HttpRetryClassifier::is_status_retryable(429)); // Too Many Requests
//! assert!(!HttpRetryClassifier::is_status_retryable(400)); // Bad Request
//!
//! // RPC retry classification
//! assert!(RpcRetryClassifier::is_code_retryable(-32000)); // Server error
//! ```
//!
//! ## Timeouts
//!
//! Configurable timeouts for different scenarios:
//!
//! ```rust
//! use walletd_resilience::TimeoutConfig;
//! use std::time::Duration;
//!
//! // Blockchain-optimized timeouts
//! let config = TimeoutConfig::blockchain();
//! assert_eq!(config.request, Duration::from_secs(60));
//!
//! // Fast timeouts for local services
//! let fast = TimeoutConfig::fast();
//! assert_eq!(fast.request, Duration::from_secs(5));
//! ```
//!
//! ## Health Checks
//!
//! Monitor service health:
//!
//! ```rust
//! use walletd_resilience::{HealthChecker, HealthCheckResult, HealthStatus};
//! use std::time::Duration;
//!
//! # async fn example() {
//! let checker = HealthChecker::default_config();
//!
//! // Record health check results
//! checker.record(HealthCheckResult::healthy("rpc_1", Duration::from_millis(50))).await;
//! checker.record(HealthCheckResult::healthy("rpc_1", Duration::from_millis(45))).await;
//!
//! // Check status
//! assert_eq!(checker.status("rpc_1").await, HealthStatus::Healthy);
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod backoff;
pub mod circuit_breaker;
pub mod health;
pub mod retry_policy;
pub mod timeout;

// Re-export main types
pub use backoff::{
    BackoffConfig, BackoffError, DecorrelatedJitter, ExponentialBackoff,
    with_backoff, with_default_backoff,
};

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError,
    CircuitMetrics, CircuitOpenError, CircuitState,
};

pub use health::{
    HealthCheckResult, HealthChecker, HealthCheckerConfig,
    HealthReport, HealthStatus, ServiceHealthReport,
};

pub use retry_policy::{
    BlockchainRetryPolicy, DefaultRetryClassifier, HttpRetryClassifier,
    RetryClassifier, RetryPolicy, RpcRetryClassifier,
};

pub use timeout::{
    AdaptiveTimeout, Deadline, DeadlineError, TimeoutConfig, TimeoutError,
    with_connect_timeout, with_request_timeout, with_timeout,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_creation() {
        let cb = CircuitBreaker::with_name("test");
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_backoff_config() {
        let config = BackoffConfig::default();
        assert_eq!(config.max_attempts, 5);
    }

    #[test]
    fn test_timeout_config() {
        let config = TimeoutConfig::blockchain();
        assert!(config.request > Duration::from_secs(30));
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::default();
        assert!(policy.retry_on_timeout);
    }

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::default_config();
        checker.register("test").await;
        assert_eq!(checker.status("test").await, HealthStatus::Unknown);
    }
}
