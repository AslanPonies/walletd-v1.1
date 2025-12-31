//! Exponential backoff with jitter
//!
//! Implements retry delays that grow exponentially with random jitter
//! to prevent thundering herd problems.

use rand::Rng;
use std::time::Duration;

/// Backoff strategy configuration
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay cap
    pub max_delay: Duration,
    /// Multiplier for each retry (typically 2.0)
    pub multiplier: f64,
    /// Jitter factor (0.0 to 1.0, recommended 0.1-0.3)
    pub jitter: f64,
    /// Maximum number of attempts (including initial)
    pub max_attempts: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: 0.2,
            max_attempts: 5,
        }
    }
}

impl BackoffConfig {
    /// Create a new backoff config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set jitter factor (0.0 to 1.0)
    pub fn with_jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.clamp(0.0, 1.0);
        self
    }

    /// Set maximum attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Create an aggressive config for fast retries
    pub fn aggressive() -> Self {
        Self {
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            multiplier: 1.5,
            jitter: 0.1,
            max_attempts: 10,
        }
    }

    /// Create a conservative config for slow retries
    pub fn conservative() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: 0.3,
            max_attempts: 3,
        }
    }
}

/// Exponential backoff iterator
pub struct ExponentialBackoff {
    config: BackoffConfig,
    attempt: u32,
    current_delay: Duration,
}

impl ExponentialBackoff {
    /// Create a new backoff instance
    pub fn new(config: BackoffConfig) -> Self {
        Self {
            current_delay: config.initial_delay,
            config,
            attempt: 0,
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(BackoffConfig::default())
    }

    /// Get the current attempt number (0-indexed)
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Check if more retries are allowed
    pub fn can_retry(&self) -> bool {
        self.attempt < self.config.max_attempts
    }

    /// Get remaining attempts
    pub fn remaining_attempts(&self) -> u32 {
        self.config.max_attempts.saturating_sub(self.attempt)
    }

    /// Reset the backoff state
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.current_delay = self.config.initial_delay;
    }

    /// Calculate delay with jitter
    fn delay_with_jitter(&self, base_delay: Duration) -> Duration {
        if self.config.jitter <= 0.0 {
            return base_delay;
        }

        let mut rng = rand::thread_rng();
        let jitter_range = base_delay.as_secs_f64() * self.config.jitter;
        let jitter = rng.gen_range(-jitter_range..jitter_range);
        let jittered = base_delay.as_secs_f64() + jitter;

        Duration::from_secs_f64(jittered.max(0.0))
    }

    /// Get next delay without advancing (peek)
    pub fn peek_delay(&self) -> Option<Duration> {
        if !self.can_retry() {
            return None;
        }

        let delay = self.delay_with_jitter(self.current_delay);
        Some(delay.min(self.config.max_delay))
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.can_retry() {
            return None;
        }

        let delay = self.delay_with_jitter(self.current_delay);
        let capped_delay = delay.min(self.config.max_delay);

        self.attempt += 1;
        self.current_delay = Duration::from_secs_f64(
            (self.current_delay.as_secs_f64() * self.config.multiplier).min(
                self.config.max_delay.as_secs_f64(),
            ),
        );

        Some(capped_delay)
    }
}

/// Execute a function with exponential backoff retries
pub async fn with_backoff<F, Fut, T, E>(
    config: BackoffConfig,
    mut f: F,
) -> Result<T, BackoffError<E>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut backoff = ExponentialBackoff::new(config);
    let mut last_error = None;

    while backoff.can_retry() {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::debug!(
                    attempt = backoff.attempt(),
                    remaining = backoff.remaining_attempts(),
                    error = ?e,
                    "Operation failed, will retry"
                );
                last_error = Some(e);

                if let Some(delay) = backoff.next() {
                    if backoff.can_retry() {
                        tracing::trace!(delay = ?delay, "Waiting before retry");
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }

    Err(BackoffError {
        attempts: backoff.attempt,
        last_error,
    })
}

/// Execute with default backoff config
pub async fn with_default_backoff<F, Fut, T, E>(f: F) -> Result<T, BackoffError<E>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    with_backoff(BackoffConfig::default(), f).await
}

/// Error when all retries exhausted
#[derive(Debug)]
pub struct BackoffError<E> {
    /// Number of attempts made
    pub attempts: u32,
    /// Last error encountered
    pub last_error: Option<E>,
}

impl<E: std::fmt::Display> std::fmt::Display for BackoffError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "All {} retry attempts exhausted", self.attempts)?;
        if let Some(ref e) = self.last_error {
            write!(f, "; last error: {}", e)?;
        }
        Ok(())
    }
}

impl<E: std::error::Error + 'static> std::error::Error for BackoffError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.last_error.as_ref().map(|e| e as _)
    }
}

/// Decorrelated jitter backoff (AWS recommended)
///
/// More sophisticated algorithm that provides better distribution
/// of retry times across multiple clients.
pub struct DecorrelatedJitter {
    config: BackoffConfig,
    attempt: u32,
    previous_delay: Duration,
}

impl DecorrelatedJitter {
    /// Create new decorrelated jitter backoff
    pub fn new(config: BackoffConfig) -> Self {
        Self {
            previous_delay: config.initial_delay,
            config,
            attempt: 0,
        }
    }

    /// Check if more retries are allowed
    pub fn can_retry(&self) -> bool {
        self.attempt < self.config.max_attempts
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.previous_delay = self.config.initial_delay;
    }
}

impl Iterator for DecorrelatedJitter {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.can_retry() {
            return None;
        }

        let mut rng = rand::thread_rng();

        // Decorrelated jitter formula: sleep = min(cap, random_between(base, sleep * 3))
        let base = self.config.initial_delay.as_secs_f64();
        let cap = self.config.max_delay.as_secs_f64();
        let prev = self.previous_delay.as_secs_f64();

        let next = rng.gen_range(base..(prev * 3.0).max(base + 0.001));
        let capped = next.min(cap);

        self.previous_delay = Duration::from_secs_f64(capped);
        self.attempt += 1;

        Some(self.previous_delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BackoffConfig::default();
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_attempts, 5);
    }

    #[test]
    fn test_backoff_iteration() {
        let config = BackoffConfig::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_max_attempts(3)
            .with_jitter(0.0); // No jitter for predictable testing

        let backoff = ExponentialBackoff::new(config);
        let delays: Vec<_> = backoff.collect();

        assert_eq!(delays.len(), 3);
        assert_eq!(delays[0], Duration::from_millis(100));
        assert_eq!(delays[1], Duration::from_millis(200));
        assert_eq!(delays[2], Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = BackoffConfig::new()
            .with_initial_delay(Duration::from_secs(10))
            .with_max_delay(Duration::from_secs(15))
            .with_max_attempts(5)
            .with_jitter(0.0);

        let backoff = ExponentialBackoff::new(config);
        let delays: Vec<_> = backoff.collect();

        for delay in delays {
            assert!(delay <= Duration::from_secs(15));
        }
    }

    #[test]
    fn test_can_retry() {
        let config = BackoffConfig::new().with_max_attempts(2);
        let mut backoff = ExponentialBackoff::new(config);

        assert!(backoff.can_retry());
        backoff.next();
        assert!(backoff.can_retry());
        backoff.next();
        assert!(!backoff.can_retry());
    }

    #[test]
    fn test_reset() {
        let config = BackoffConfig::new().with_max_attempts(2).with_jitter(0.0);
        let mut backoff = ExponentialBackoff::new(config);

        backoff.next();
        backoff.next();
        assert!(!backoff.can_retry());

        backoff.reset();
        assert!(backoff.can_retry());
        assert_eq!(backoff.attempt(), 0);
    }

    #[test]
    fn test_jitter_applied() {
        let config = BackoffConfig::new()
            .with_initial_delay(Duration::from_secs(1))
            .with_max_attempts(10)
            .with_jitter(0.5);

        let mut backoff = ExponentialBackoff::new(config);
        let mut delays = Vec::new();

        for _ in 0..5 {
            delays.push(backoff.next().unwrap());
        }

        // With jitter, delays should vary
        let unique: std::collections::HashSet<_> = delays.iter().collect();
        // Most delays should be different due to jitter
        assert!(unique.len() > 1 || delays.len() == 1);
    }

    #[test]
    fn test_aggressive_config() {
        let config = BackoffConfig::aggressive();
        assert_eq!(config.initial_delay, Duration::from_millis(50));
        assert_eq!(config.max_attempts, 10);
    }

    #[test]
    fn test_conservative_config() {
        let config = BackoffConfig::conservative();
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_attempts, 3);
    }

    #[test]
    fn test_remaining_attempts() {
        let config = BackoffConfig::new().with_max_attempts(5);
        let mut backoff = ExponentialBackoff::new(config);

        assert_eq!(backoff.remaining_attempts(), 5);
        backoff.next();
        assert_eq!(backoff.remaining_attempts(), 4);
        backoff.next();
        backoff.next();
        assert_eq!(backoff.remaining_attempts(), 2);
    }

    #[tokio::test]
    async fn test_with_backoff_success() {
        let config = BackoffConfig::new().with_max_attempts(3);
        let mut attempts = 0;

        let result = with_backoff(config, || {
            attempts += 1;
            async move { Ok::<_, &str>(42) }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 1); // Succeeded first try
    }

    #[tokio::test]
    async fn test_with_backoff_eventual_success() {
        let config = BackoffConfig::new()
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(1));
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = with_backoff(config, || {
            let a = attempts_clone.clone();
            async move {
                let count = a.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if count < 3 {
                    Err("not yet")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_with_backoff_exhausted() {
        let config = BackoffConfig::new()
            .with_max_attempts(3)
            .with_initial_delay(Duration::from_millis(1));

        let result: Result<(), _> = with_backoff(config, || async { Err::<(), _>("always fails") }).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.attempts, 3);
        assert_eq!(err.last_error, Some("always fails"));
    }

    #[test]
    fn test_decorrelated_jitter() {
        let config = BackoffConfig::new()
            .with_initial_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10))
            .with_max_attempts(5);

        let mut backoff = DecorrelatedJitter::new(config);
        let delays: Vec<_> = backoff.by_ref().take(5).collect();

        assert_eq!(delays.len(), 5);
        for delay in delays {
            assert!(delay <= Duration::from_secs(10));
            assert!(delay >= Duration::from_millis(100));
        }
    }
}
