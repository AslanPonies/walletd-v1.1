//! Health check utilities for monitoring service health
//!
//! Provides periodic health checking and status tracking.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Service name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Response time (if applicable)
    pub response_time: Option<Duration>,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// When the check was performed
    pub checked_at: Instant,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(name: impl Into<String>, response_time: Duration) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            response_time: Some(response_time),
            error: None,
            checked_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            response_time: None,
            error: Some(error.into()),
            checked_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Create a degraded result
    pub fn degraded(name: impl Into<String>, response_time: Duration, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            response_time: Some(response_time),
            error: Some(reason.into()),
            checked_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if result is stale
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.checked_at.elapsed() > max_age
    }
}

/// Configuration for health checker
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    /// Timeout for health check
    pub check_timeout: Duration,
    /// Response time threshold for degraded status
    pub degraded_threshold: Duration,
    /// Number of consecutive failures before unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes to recover
    pub recovery_threshold: u32,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            degraded_threshold: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}

impl HealthCheckerConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set check interval
    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Set check timeout
    pub fn with_check_timeout(mut self, timeout: Duration) -> Self {
        self.check_timeout = timeout;
        self
    }

    /// Set degraded threshold
    pub fn with_degraded_threshold(mut self, threshold: Duration) -> Self {
        self.degraded_threshold = threshold;
        self
    }

    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }
}

/// Service health state
#[derive(Debug)]
struct ServiceHealth {
    name: String,
    status: HealthStatus,
    consecutive_failures: u32,
    consecutive_successes: u32,
    last_result: Option<HealthCheckResult>,
    total_checks: u64,
    total_failures: u64,
}

impl ServiceHealth {
    fn new(name: String) -> Self {
        Self {
            name,
            status: HealthStatus::Unknown,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_result: None,
            total_checks: 0,
            total_failures: 0,
        }
    }

    fn record_success(&mut self, result: HealthCheckResult, config: &HealthCheckerConfig) {
        self.total_checks += 1;
        self.consecutive_failures = 0;
        self.consecutive_successes += 1;
        self.last_result = Some(result.clone());

        // Check for degraded status based on response time
        if let Some(response_time) = result.response_time {
            if response_time > config.degraded_threshold {
                self.status = HealthStatus::Degraded;
                return;
            }
        }

        // Recover to healthy if enough successes
        if self.consecutive_successes >= config.recovery_threshold {
            self.status = HealthStatus::Healthy;
        }
    }

    fn record_failure(&mut self, result: HealthCheckResult, config: &HealthCheckerConfig) {
        self.total_checks += 1;
        self.total_failures += 1;
        self.consecutive_successes = 0;
        self.consecutive_failures += 1;
        self.last_result = Some(result);

        // Mark unhealthy after threshold
        if self.consecutive_failures >= config.failure_threshold {
            self.status = HealthStatus::Unhealthy;
        } else if self.status == HealthStatus::Healthy {
            self.status = HealthStatus::Degraded;
        }
    }

    fn failure_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.total_failures as f64 / self.total_checks as f64
        }
    }
}

/// Health checker for monitoring multiple services
pub struct HealthChecker {
    config: HealthCheckerConfig,
    services: Arc<RwLock<HashMap<String, ServiceHealth>>>,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(config: HealthCheckerConfig) -> Self {
        Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(HealthCheckerConfig::default())
    }

    /// Register a service
    pub async fn register(&self, name: impl Into<String>) {
        let name = name.into();
        let mut services = self.services.write().await;
        services.entry(name.clone()).or_insert_with(|| ServiceHealth::new(name));
    }

    /// Record health check result
    pub async fn record(&self, result: HealthCheckResult) {
        let mut services = self.services.write().await;
        let health = services
            .entry(result.name.clone())
            .or_insert_with(|| ServiceHealth::new(result.name.clone()));

        match result.status {
            HealthStatus::Healthy | HealthStatus::Degraded => {
                health.record_success(result, &self.config);
            }
            HealthStatus::Unhealthy | HealthStatus::Unknown => {
                health.record_failure(result, &self.config);
            }
        }
    }

    /// Get health status of a service
    pub async fn status(&self, name: &str) -> HealthStatus {
        let services = self.services.read().await;
        services
            .get(name)
            .map(|h| h.status)
            .unwrap_or(HealthStatus::Unknown)
    }

    /// Get all service statuses
    pub async fn all_statuses(&self) -> HashMap<String, HealthStatus> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, health)| (name.clone(), health.status))
            .collect()
    }

    /// Get overall system health
    pub async fn overall_status(&self) -> HealthStatus {
        let services = self.services.read().await;
        if services.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_degraded = false;
        for health in services.values() {
            match health.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get detailed health report
    pub async fn report(&self) -> HealthReport {
        let services = self.services.read().await;
        let overall = if services.is_empty() {
            HealthStatus::Unknown
        } else {
            let mut has_degraded = false;
            let mut has_unhealthy = false;
            for health in services.values() {
                match health.status {
                    HealthStatus::Unhealthy => has_unhealthy = true,
                    HealthStatus::Degraded => has_degraded = true,
                    _ => {}
                }
            }
            if has_unhealthy {
                HealthStatus::Unhealthy
            } else if has_degraded {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            }
        };

        let service_reports: Vec<_> = services
            .values()
            .map(|h| ServiceHealthReport {
                name: h.name.clone(),
                status: h.status,
                failure_rate: h.failure_rate(),
                total_checks: h.total_checks,
                consecutive_failures: h.consecutive_failures,
                last_check: h.last_result.as_ref().map(|r| r.checked_at.elapsed()),
            })
            .collect();

        HealthReport {
            overall,
            services: service_reports,
            generated_at: Instant::now(),
        }
    }

    /// Check if all services are healthy
    pub async fn is_healthy(&self) -> bool {
        self.overall_status().await == HealthStatus::Healthy
    }
}

/// Health report for all services
#[derive(Debug)]
pub struct HealthReport {
    /// Overall system health
    pub overall: HealthStatus,
    /// Individual service reports
    pub services: Vec<ServiceHealthReport>,
    /// When report was generated
    pub generated_at: Instant,
}

/// Health report for a single service
#[derive(Debug)]
pub struct ServiceHealthReport {
    /// Service name
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Historical failure rate
    pub failure_rate: f64,
    /// Total checks performed
    pub total_checks: u64,
    /// Current consecutive failures
    pub consecutive_failures: u32,
    /// Time since last check
    pub last_check: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult::healthy("test", Duration::from_millis(100));
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult::unhealthy("test", "connection failed");
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_health_check_result_degraded() {
        let result = HealthCheckResult::degraded("test", Duration::from_secs(5), "slow response");
        assert_eq!(result.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_check_result_metadata() {
        let result = HealthCheckResult::healthy("test", Duration::from_millis(100))
            .with_metadata("version", "1.0")
            .with_metadata("region", "us-east-1");

        assert_eq!(result.metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(result.metadata.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_health_check_result_stale() {
        let result = HealthCheckResult::healthy("test", Duration::from_millis(100));
        assert!(!result.is_stale(Duration::from_secs(60)));
        // Can't easily test staleness without time manipulation
    }

    #[tokio::test]
    async fn test_health_checker_register() {
        let checker = HealthChecker::default_config();
        checker.register("service1").await;
        checker.register("service2").await;

        let statuses = checker.all_statuses().await;
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses.get("service1"), Some(&HealthStatus::Unknown));
    }

    #[tokio::test]
    async fn test_health_checker_record_healthy() {
        let checker = HealthChecker::default_config();

        let result = HealthCheckResult::healthy("service1", Duration::from_millis(100));
        checker.record(result).await;

        // Need recovery_threshold successes to become healthy
        let result = HealthCheckResult::healthy("service1", Duration::from_millis(100));
        checker.record(result).await;

        assert_eq!(checker.status("service1").await, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checker_record_unhealthy() {
        let config = HealthCheckerConfig::default().with_failure_threshold(2);
        let checker = HealthChecker::new(config);

        checker.record(HealthCheckResult::unhealthy("service1", "error")).await;
        assert_ne!(checker.status("service1").await, HealthStatus::Unhealthy);

        checker.record(HealthCheckResult::unhealthy("service1", "error")).await;
        assert_eq!(checker.status("service1").await, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_health_checker_overall_status() {
        let checker = HealthChecker::default_config();

        // All healthy
        for _ in 0..2 {
            checker.record(HealthCheckResult::healthy("s1", Duration::from_millis(100))).await;
            checker.record(HealthCheckResult::healthy("s2", Duration::from_millis(100))).await;
        }

        assert_eq!(checker.overall_status().await, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checker_overall_degraded() {
        let config = HealthCheckerConfig::default()
            .with_degraded_threshold(Duration::from_millis(50));
        let checker = HealthChecker::new(config);

        // One healthy, one slow (degraded)
        for _ in 0..2 {
            checker.record(HealthCheckResult::healthy("s1", Duration::from_millis(100))).await;
        }
        checker.record(HealthCheckResult::degraded("s2", Duration::from_secs(1), "slow")).await;

        assert_eq!(checker.overall_status().await, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_health_checker_report() {
        let checker = HealthChecker::default_config();

        for _ in 0..2 {
            checker.record(HealthCheckResult::healthy("service1", Duration::from_millis(100))).await;
        }

        let report = checker.report().await;
        assert_eq!(report.services.len(), 1);
        assert_eq!(report.services[0].name, "service1");
        assert_eq!(report.services[0].total_checks, 2);
    }

    #[tokio::test]
    async fn test_health_checker_is_healthy() {
        let checker = HealthChecker::default_config();

        for _ in 0..2 {
            checker.record(HealthCheckResult::healthy("s1", Duration::from_millis(100))).await;
        }

        assert!(checker.is_healthy().await);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(HealthStatus::Unknown.to_string(), "unknown");
    }
}
