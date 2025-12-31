//! Circuit breaker pattern implementation
//!
//! Prevents cascading failures by stopping requests to unhealthy services.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CircuitState {
    /// Circuit is closed - requests flow normally
    Closed = 0,
    /// Circuit is open - requests are rejected
    Open = 1,
    /// Circuit is half-open - testing if service recovered
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(v: u8) -> Self {
        match v {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Configuration for circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open to close circuit
    pub success_threshold: u32,
    /// Time to wait before trying again (half-open)
    pub reset_timeout: Duration,
    /// Time window for counting failures
    pub failure_window: Duration,
    /// Name for logging/metrics
    pub name: String,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            reset_timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
            name: "default".to_string(),
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new config with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set success threshold for half-open state
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set reset timeout
    pub fn with_reset_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout = timeout;
        self
    }

    /// Set failure window
    pub fn with_failure_window(mut self, window: Duration) -> Self {
        self.failure_window = window;
        self
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure: RwLock<Option<Instant>>,
    opened_at: RwLock<Option<Instant>>,
}

/// Error when circuit is open
#[derive(Debug, Clone)]
pub struct CircuitOpenError {
    /// Name of the circuit breaker
    pub name: String,
    /// Time until circuit may close
    pub retry_after: Duration,
}

impl std::fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Circuit '{}' is open, retry after {:?}",
            self.name, self.retry_after
        )
    }
}

impl std::error::Error for CircuitOpenError {}

impl CircuitBreaker {
    /// Create a new circuit breaker with config
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure: RwLock::new(None),
            opened_at: RwLock::new(None),
        }
    }

    /// Create with default config and name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self::new(CircuitBreakerConfig::new(name))
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        CircuitState::from(self.state.load(Ordering::SeqCst))
    }

    /// Check if circuit allows requests
    pub async fn can_execute(&self) -> Result<(), CircuitOpenError> {
        match self.state() {
            CircuitState::Closed => Ok(()),
            CircuitState::HalfOpen => Ok(()),
            CircuitState::Open => {
                // Check if reset timeout has passed
                let opened_at = self.opened_at.read().await;
                if let Some(opened) = *opened_at {
                    let elapsed = opened.elapsed();
                    if elapsed >= self.config.reset_timeout {
                        // Transition to half-open
                        drop(opened_at);
                        self.transition_to_half_open().await;
                        Ok(())
                    } else {
                        Err(CircuitOpenError {
                            name: self.config.name.clone(),
                            retry_after: self.config.reset_timeout - elapsed,
                        })
                    }
                } else {
                    // No opened_at, something wrong, allow request
                    Ok(())
                }
            }
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        match self.state() {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.success_threshold as u64 {
                    self.transition_to_closed().await;
                    tracing::info!(
                        circuit = %self.config.name,
                        "Circuit closed after successful recovery"
                    );
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {
                // Shouldn't happen, but ignore
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        *self.last_failure.write().await = Some(Instant::now());

        match self.state() {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.config.failure_threshold as u64 {
                    self.transition_to_open().await;
                    tracing::warn!(
                        circuit = %self.config.name,
                        failures = count,
                        "Circuit opened due to failures"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open reopens circuit
                self.transition_to_open().await;
                tracing::warn!(
                    circuit = %self.config.name,
                    "Circuit reopened after half-open failure"
                );
            }
            CircuitState::Open => {
                // Already open, ignore
            }
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        self.can_execute()
            .await
            .map_err(CircuitBreakerError::CircuitOpen)?;

        match f().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::Inner(e))
            }
        }
    }

    async fn transition_to_open(&self) {
        self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
        *self.opened_at.write().await = Some(Instant::now());
        self.success_count.store(0, Ordering::SeqCst);
    }

    async fn transition_to_half_open(&self) {
        self.state
            .store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
    }

    async fn transition_to_closed(&self) {
        self.state
            .store(CircuitState::Closed as u8, Ordering::SeqCst);
        *self.opened_at.write().await = None;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
    }

    /// Get metrics
    pub fn metrics(&self) -> CircuitMetrics {
        CircuitMetrics {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
        }
    }

    /// Force close the circuit (for testing/admin)
    pub async fn force_close(&self) {
        self.transition_to_closed().await;
    }

    /// Force open the circuit (for testing/admin)
    pub async fn force_open(&self) {
        self.transition_to_open().await;
    }
}

/// Error type for circuit breaker operations
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open
    CircuitOpen(CircuitOpenError),
    /// Inner operation error
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircuitOpen(e) => write!(f, "{}", e),
            Self::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CircuitOpen(e) => Some(e),
            Self::Inner(e) => Some(e),
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitMetrics {
    /// Current state
    pub state: CircuitState,
    /// Current failure count
    pub failure_count: u64,
    /// Success count (in half-open)
    pub success_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::with_name("test");
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig::new("test").with_failure_threshold(3);
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure().await;
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_rejects_when_open() {
        let cb = CircuitBreaker::with_name("test");
        cb.force_open().await;

        let result = cb.can_execute().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig::new("test").with_failure_threshold(3);
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_success().await;

        assert_eq!(cb.metrics().failure_count, 0);
    }

    #[tokio::test]
    async fn test_half_open_closes_after_successes() {
        let config = CircuitBreakerConfig::new("test")
            .with_failure_threshold(1)
            .with_success_threshold(2);
        let cb = CircuitBreaker::new(config);

        cb.force_open().await;
        cb.transition_to_half_open().await;

        cb.record_success().await;
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success().await;
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_reopens_on_failure() {
        let cb = CircuitBreaker::with_name("test");
        cb.transition_to_half_open().await;

        cb.record_failure().await;
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_execute_success() {
        let cb = CircuitBreaker::with_name("test");

        let result: Result<i32, CircuitBreakerError<&str>> =
            cb.execute(|| async { Ok::<_, &str>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_execute_failure_counted() {
        let config = CircuitBreakerConfig::new("test").with_failure_threshold(2);
        let cb = CircuitBreaker::new(config);

        let _: Result<i32, _> = cb.execute(|| async { Err::<i32, _>("error") }).await;
        assert_eq!(cb.metrics().failure_count, 1);

        let _: Result<i32, _> = cb.execute(|| async { Err::<i32, _>("error") }).await;
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_force_close() {
        let cb = CircuitBreaker::with_name("test");
        cb.force_open().await;
        assert_eq!(cb.state(), CircuitState::Open);

        cb.force_close().await;
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_metrics() {
        let cb = CircuitBreaker::with_name("test");
        cb.record_failure().await;
        cb.record_failure().await;

        let metrics = cb.metrics();
        assert_eq!(metrics.state, CircuitState::Closed);
        assert_eq!(metrics.failure_count, 2);
    }
}
