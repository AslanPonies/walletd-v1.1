//! Provider management and failover

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub name: String,
    pub url: String,
    pub is_healthy: bool,
    pub last_check: Instant,
    pub response_time: Duration,
    pub error_count: u32,
    pub success_count: u32,
}

impl ProviderHealth {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            is_healthy: true,
            last_check: Instant::now(),
            response_time: Duration::from_secs(0),
            error_count: 0,
            success_count: 0,
        }
    }

    pub fn record_success(&mut self, response_time: Duration) {
        self.is_healthy = true;
        self.last_check = Instant::now();
        self.response_time = response_time;
        self.success_count += 1;
        self.error_count = 0;
    }

    pub fn record_failure(&mut self) {
        self.last_check = Instant::now();
        self.error_count += 1;
        if self.error_count >= 3 {
            self.is_healthy = false;
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total == 0 { 1.0 } else { self.success_count as f64 / total as f64 }
    }
}

/// Provider pool with automatic failover
pub struct ProviderPool {
    providers: Arc<RwLock<Vec<ProviderHealth>>>,
}

impl ProviderPool {
    pub fn new(providers: Vec<(&str, &str)>) -> Self {
        let health_providers: Vec<ProviderHealth> = providers
            .into_iter()
            .map(|(name, url)| ProviderHealth::new(name, url))
            .collect();

        Self {
            providers: Arc::new(RwLock::new(health_providers)),
        }
    }

    pub async fn get_best(&self) -> Option<String> {
        let providers = self.providers.read().await;
        providers
            .iter()
            .filter(|p| p.is_healthy)
            .min_by(|a, b| a.response_time.cmp(&b.response_time))
            .map(|p| p.url.clone())
    }

    pub async fn get_healthy(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.iter().filter(|p| p.is_healthy).map(|p| p.url.clone()).collect()
    }

    pub async fn record_success(&self, url: &str, response_time: Duration) {
        let mut providers = self.providers.write().await;
        if let Some(provider) = providers.iter_mut().find(|p| p.url == url) {
            provider.record_success(response_time);
        }
    }

    pub async fn record_failure(&self, url: &str) {
        let mut providers = self.providers.write().await;
        if let Some(provider) = providers.iter_mut().find(|p| p.url == url) {
            provider.record_failure();
        }
    }
}
