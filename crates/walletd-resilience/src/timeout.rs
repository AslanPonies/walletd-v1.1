//! Timeout utilities for production workloads
//!
//! Provides configurable timeouts with different strategies.

use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Timeout configuration for different operations
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Connection timeout
    pub connect: Duration,
    /// Request timeout (total time for request/response)
    pub request: Duration,
    /// Read timeout (time waiting for data)
    pub read: Duration,
    /// Write timeout (time sending data)
    pub write: Duration,
    /// Idle timeout (keep-alive connections)
    pub idle: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            request: Duration::from_secs(30),
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
            idle: Duration::from_secs(90),
        }
    }
}

impl TimeoutConfig {
    /// Create new timeout config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set connection timeout
    pub fn with_connect(mut self, timeout: Duration) -> Self {
        self.connect = timeout;
        self
    }

    /// Set request timeout
    pub fn with_request(mut self, timeout: Duration) -> Self {
        self.request = timeout;
        self
    }

    /// Set read timeout
    pub fn with_read(mut self, timeout: Duration) -> Self {
        self.read = timeout;
        self
    }

    /// Set write timeout
    pub fn with_write(mut self, timeout: Duration) -> Self {
        self.write = timeout;
        self
    }

    /// Set idle timeout
    pub fn with_idle(mut self, timeout: Duration) -> Self {
        self.idle = timeout;
        self
    }

    /// Fast timeouts for local/low-latency connections
    pub fn fast() -> Self {
        Self {
            connect: Duration::from_secs(2),
            request: Duration::from_secs(5),
            read: Duration::from_secs(5),
            write: Duration::from_secs(5),
            idle: Duration::from_secs(30),
        }
    }

    /// Slow timeouts for high-latency/unreliable connections
    pub fn slow() -> Self {
        Self {
            connect: Duration::from_secs(30),
            request: Duration::from_secs(120),
            read: Duration::from_secs(60),
            write: Duration::from_secs(60),
            idle: Duration::from_secs(300),
        }
    }

    /// Blockchain-specific timeouts
    pub fn blockchain() -> Self {
        Self {
            connect: Duration::from_secs(10),
            request: Duration::from_secs(60), // Some RPC calls are slow
            read: Duration::from_secs(45),
            write: Duration::from_secs(30),
            idle: Duration::from_secs(120),
        }
    }
}

/// Timeout error
#[derive(Debug, Clone)]
pub struct TimeoutError {
    /// The operation that timed out
    pub operation: String,
    /// The timeout duration
    pub duration: Duration,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation '{}' timed out after {:?}",
            self.operation, self.duration
        )
    }
}

impl std::error::Error for TimeoutError {}

/// Execute a future with a timeout
pub async fn with_timeout<T>(
    duration: Duration,
    operation: impl Into<String>,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    let op = operation.into();
    timeout(duration, future)
        .await
        .map_err(|_| TimeoutError {
            operation: op,
            duration,
        })
}

/// Execute with request timeout from config
pub async fn with_request_timeout<T>(
    config: &TimeoutConfig,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    with_timeout(config.request, "request", future).await
}

/// Execute with connect timeout from config
pub async fn with_connect_timeout<T>(
    config: &TimeoutConfig,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    with_timeout(config.connect, "connect", future).await
}

/// Deadline tracking for complex operations
#[derive(Debug, Clone)]
pub struct Deadline {
    start: std::time::Instant,
    timeout: Duration,
}

impl Deadline {
    /// Create a new deadline
    pub fn new(timeout: Duration) -> Self {
        Self {
            start: std::time::Instant::now(),
            timeout,
        }
    }

    /// Check if deadline has passed
    pub fn is_expired(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.start.elapsed())
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Check if enough time remains for an operation
    pub fn has_time_for(&self, operation_estimate: Duration) -> bool {
        self.remaining() >= operation_estimate
    }

    /// Execute with remaining time as timeout
    pub async fn execute<T, E>(
        &self,
        future: impl Future<Output = Result<T, E>>,
    ) -> Result<T, DeadlineError<E>> {
        if self.is_expired() {
            return Err(DeadlineError::Expired);
        }

        match timeout(self.remaining(), future).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(DeadlineError::Inner(e)),
            Err(_) => Err(DeadlineError::Expired),
        }
    }
}

/// Deadline error
#[derive(Debug)]
pub enum DeadlineError<E> {
    /// Deadline expired
    Expired,
    /// Inner operation error
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for DeadlineError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expired => write!(f, "Deadline expired"),
            Self::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for DeadlineError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Expired => None,
            Self::Inner(e) => Some(e),
        }
    }
}

/// Adaptive timeout that adjusts based on observed latencies
#[derive(Debug)]
pub struct AdaptiveTimeout {
    /// Base timeout
    base: Duration,
    /// Minimum timeout
    min: Duration,
    /// Maximum timeout
    max: Duration,
    /// Rolling average of response times
    avg_response: std::sync::atomic::AtomicU64,
    /// Number of samples
    samples: std::sync::atomic::AtomicU64,
    /// Multiplier for timeout (e.g., 3x average)
    multiplier: f64,
}

impl AdaptiveTimeout {
    /// Create new adaptive timeout
    pub fn new(base: Duration, min: Duration, max: Duration) -> Self {
        Self {
            base,
            min,
            max,
            avg_response: std::sync::atomic::AtomicU64::new(base.as_millis() as u64),
            samples: std::sync::atomic::AtomicU64::new(0),
            multiplier: 3.0,
        }
    }

    /// Create with default settings
    pub fn default_settings() -> Self {
        Self::new(
            Duration::from_secs(5),
            Duration::from_millis(500),
            Duration::from_secs(60),
        )
    }

    /// Set multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Get current timeout value
    pub fn get(&self) -> Duration {
        let avg_ms = self.avg_response.load(std::sync::atomic::Ordering::Relaxed);
        let timeout_ms = (avg_ms as f64 * self.multiplier) as u64;
        let timeout = Duration::from_millis(timeout_ms);
        timeout.clamp(self.min, self.max)
    }

    /// Record an observed response time
    pub fn record(&self, response_time: Duration) {
        let response_ms = response_time.as_millis() as u64;
        let samples = self.samples.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let old_avg = self.avg_response.load(std::sync::atomic::Ordering::Relaxed);

        // Exponential moving average
        let alpha = 0.2; // Weight for new sample
        let new_avg = if samples == 1 {
            response_ms
        } else {
            ((1.0 - alpha) * old_avg as f64 + alpha * response_ms as f64) as u64
        };

        self.avg_response
            .store(new_avg, std::sync::atomic::Ordering::Relaxed);
    }

    /// Reset to base timeout
    pub fn reset(&self) {
        self.avg_response
            .store(self.base.as_millis() as u64, std::sync::atomic::Ordering::Relaxed);
        self.samples.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect, Duration::from_secs(10));
        assert_eq!(config.request, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_config_fast() {
        let config = TimeoutConfig::fast();
        assert_eq!(config.connect, Duration::from_secs(2));
        assert_eq!(config.request, Duration::from_secs(5));
    }

    #[test]
    fn test_timeout_config_slow() {
        let config = TimeoutConfig::slow();
        assert_eq!(config.connect, Duration::from_secs(30));
        assert_eq!(config.request, Duration::from_secs(120));
    }

    #[test]
    fn test_timeout_config_blockchain() {
        let config = TimeoutConfig::blockchain();
        assert_eq!(config.request, Duration::from_secs(60));
    }

    #[test]
    fn test_timeout_config_builder() {
        let config = TimeoutConfig::new()
            .with_connect(Duration::from_secs(5))
            .with_request(Duration::from_secs(15));

        assert_eq!(config.connect, Duration::from_secs(5));
        assert_eq!(config.request, Duration::from_secs(15));
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(
            Duration::from_secs(1),
            "test",
            async { 42 },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_timeout_expired() {
        let result = with_timeout(
            Duration::from_millis(1),
            "slow_op",
            async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                42
            },
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.operation, "slow_op");
    }

    #[test]
    fn test_deadline_not_expired() {
        let deadline = Deadline::new(Duration::from_secs(60));
        assert!(!deadline.is_expired());
        assert!(deadline.remaining() > Duration::ZERO);
    }

    #[test]
    fn test_deadline_has_time_for() {
        let deadline = Deadline::new(Duration::from_secs(10));
        assert!(deadline.has_time_for(Duration::from_secs(5)));
        assert!(!deadline.has_time_for(Duration::from_secs(15)));
    }

    #[tokio::test]
    async fn test_deadline_execute_success() {
        let deadline = Deadline::new(Duration::from_secs(10));
        let result: Result<i32, DeadlineError<&str>> = deadline
            .execute(async { Ok::<_, &str>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_adaptive_timeout_initial() {
        let at = AdaptiveTimeout::new(
            Duration::from_secs(5),
            Duration::from_secs(1),
            Duration::from_secs(30),
        );

        let timeout = at.get();
        assert!(timeout >= Duration::from_secs(1));
        assert!(timeout <= Duration::from_secs(30));
    }

    #[test]
    fn test_adaptive_timeout_records() {
        let at = AdaptiveTimeout::default_settings();

        // Record fast responses
        at.record(Duration::from_millis(100));
        at.record(Duration::from_millis(150));
        at.record(Duration::from_millis(120));

        let timeout = at.get();
        // Should be around 3x the average (~120ms = ~360ms timeout)
        assert!(timeout < Duration::from_secs(5)); // Should be less than base
    }

    #[test]
    fn test_adaptive_timeout_clamped() {
        let at = AdaptiveTimeout::new(
            Duration::from_secs(5),
            Duration::from_secs(1),
            Duration::from_secs(10),
        );

        // Record very slow response
        at.record(Duration::from_secs(100));

        let timeout = at.get();
        assert!(timeout <= Duration::from_secs(10)); // Should be clamped to max
    }

    #[test]
    fn test_adaptive_timeout_reset() {
        let at = AdaptiveTimeout::default_settings();
        at.record(Duration::from_millis(100));
        at.reset();

        // Should be back to base
        let timeout = at.get();
        assert!(timeout >= Duration::from_secs(1));
    }
}
