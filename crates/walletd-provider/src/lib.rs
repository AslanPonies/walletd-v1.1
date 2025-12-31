//! # WalletD Provider
//!
//! This crate provides connection pooling and provider management for the WalletD SDK.
//! It handles RPC connections efficiently with connection reuse, health checking,
//! and automatic failover.
//!
//! ## Features
//!
//! - Connection pooling with configurable limits
//! - Automatic health checking and reconnection
//! - Multiple endpoint support with failover
//! - Request rate limiting
//! - Caching for common queries
//! - HTTP client with connection reuse
//!
//! ## Example
//!
//! ```ignore
//! use walletd_provider::{ProviderPool, ProviderConfig, RpcClient};
//!
//! let config = ProviderConfig::new("https://eth.llamarpc.com")
//!     .with_timeout(30)
//!     .with_max_retries(3);
//!
//! let pool = ProviderPool::new();
//! pool.add_provider("ethereum", config);
//!
//! let provider = pool.get("ethereum").await?;
//! let client = provider.client();
//! let result = client.post_json::<serde_json::Value>(payload).await?;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use dashmap::DashMap;
use governor::{Quota, RateLimiter, clock::DefaultClock, state::{InMemoryState, NotKeyed}};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

/// Provider-related errors
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Provider not found
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// Request timeout
    #[error("Request timeout after {0}s")]
    Timeout(u64),

    /// Rate limited
    #[error("Rate limited")]
    RateLimited,

    /// All endpoints failed
    #[error("All endpoints failed")]
    AllEndpointsFailed,

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// RPC error response
    #[error("RPC error: code={code}, message={message}")]
    RpcError {
        /// Error code
        code: i64,
        /// Error message
        message: String,
    },
}

/// Result type for provider operations
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Configuration for a provider endpoint
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Primary RPC URL
    pub url: String,
    /// Fallback URLs
    pub fallback_urls: Vec<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Enable request caching
    pub enable_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl ProviderConfig {
    /// Creates a new provider configuration with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            fallback_urls: Vec::new(),
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_cache: true,
            cache_ttl_secs: 10,
            health_check_interval_secs: 60,
        }
    }

    /// Adds a fallback URL
    pub fn with_fallback(mut self, url: impl Into<String>) -> Self {
        self.fallback_urls.push(url.into());
        self
    }

    /// Sets the request timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Sets the maximum retry attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Sets the retry delay
    pub fn with_retry_delay(mut self, ms: u64) -> Self {
        self.retry_delay_ms = ms;
        self
    }

    /// Enables or disables caching
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Sets the cache TTL
    pub fn with_cache_ttl(mut self, secs: u64) -> Self {
        self.cache_ttl_secs = secs;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        Url::parse(&self.url).map_err(|e| ProviderError::InvalidUrl(e.to_string()))?;
        for url in &self.fallback_urls {
            Url::parse(url).map_err(|e| ProviderError::InvalidUrl(e.to_string()))?;
        }
        Ok(())
    }

    /// Returns all URLs (primary + fallbacks)
    pub fn all_urls(&self) -> Vec<&str> {
        let mut urls = vec![self.url.as_str()];
        urls.extend(self.fallback_urls.iter().map(|s| s.as_str()));
        urls
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::new("http://localhost:8545")
    }
}

/// Health status of an endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointHealth {
    /// Endpoint is healthy
    Healthy,
    /// Endpoint is degraded (slow or intermittent issues)
    Degraded,
    /// Endpoint is unhealthy
    Unhealthy,
    /// Health unknown (not yet checked)
    Unknown,
}

/// Information about a provider endpoint
#[derive(Debug, Clone)]
pub struct EndpointInfo {
    /// The endpoint URL
    pub url: String,
    /// Current health status
    pub health: EndpointHealth,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last failed request time
    pub last_failure: Option<Instant>,
    /// Total requests made
    pub total_requests: u64,
    /// Total failures
    pub total_failures: u64,
    /// Average response time in milliseconds
    pub avg_response_ms: u64,
}

impl EndpointInfo {
    fn new(url: String) -> Self {
        Self {
            url,
            health: EndpointHealth::Unknown,
            last_success: None,
            last_failure: None,
            total_requests: 0,
            total_failures: 0,
            avg_response_ms: 0,
        }
    }

    fn record_success(&mut self, response_time_ms: u64) {
        self.last_success = Some(Instant::now());
        self.total_requests += 1;
        // Update rolling average
        self.avg_response_ms = (self.avg_response_ms * (self.total_requests - 1) + response_time_ms)
            / self.total_requests;
        self.health = if self.avg_response_ms < 1000 {
            EndpointHealth::Healthy
        } else {
            EndpointHealth::Degraded
        };
    }

    fn record_failure(&mut self) {
        self.last_failure = Some(Instant::now());
        self.total_requests += 1;
        self.total_failures += 1;
        
        // Mark unhealthy if failure rate > 50%
        let failure_rate = self.total_failures as f64 / self.total_requests as f64;
        if failure_rate > 0.5 {
            self.health = EndpointHealth::Unhealthy;
        } else if failure_rate > 0.2 {
            self.health = EndpointHealth::Degraded;
        }
    }

    /// Returns the success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            1.0 - (self.total_failures as f64 / self.total_requests as f64)
        }
    }
}

/// A cached response
#[derive(Debug, Clone)]
struct CachedResponse {
    data: Vec<u8>,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedResponse {
    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

/// Managed provider with health tracking and failover
#[derive(Debug)]
pub struct ManagedProvider {
    config: ProviderConfig,
    endpoints: RwLock<Vec<EndpointInfo>>,
    cache: DashMap<String, CachedResponse>,
    current_endpoint_idx: RwLock<usize>,
}

impl ManagedProvider {
    /// Creates a new managed provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        config.validate()?;
        
        let mut endpoints = vec![EndpointInfo::new(config.url.clone())];
        for url in &config.fallback_urls {
            endpoints.push(EndpointInfo::new(url.clone()));
        }

        Ok(Self {
            config,
            endpoints: RwLock::new(endpoints),
            cache: DashMap::new(),
            current_endpoint_idx: RwLock::new(0),
        })
    }

    /// Returns the current active endpoint URL
    pub async fn current_url(&self) -> String {
        let idx = *self.current_endpoint_idx.read().await;
        let endpoints = self.endpoints.read().await;
        endpoints.get(idx).map(|e| e.url.clone()).unwrap_or_else(|| self.config.url.clone())
    }

    /// Records a successful request
    pub async fn record_success(&self, response_time_ms: u64) {
        let idx = *self.current_endpoint_idx.read().await;
        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.get_mut(idx) {
            endpoint.record_success(response_time_ms);
        }
    }

    /// Records a failed request and potentially fails over
    pub async fn record_failure(&self) {
        let mut idx = self.current_endpoint_idx.write().await;
        let mut endpoints = self.endpoints.write().await;
        
        if let Some(endpoint) = endpoints.get_mut(*idx) {
            endpoint.record_failure();
        }

        // Try to failover to next healthy endpoint
        let num_endpoints = endpoints.len();
        for i in 1..num_endpoints {
            let next_idx = (*idx + i) % num_endpoints;
            if endpoints[next_idx].health != EndpointHealth::Unhealthy {
                tracing::info!(
                    "Failing over from {} to {}",
                    endpoints[*idx].url,
                    endpoints[next_idx].url
                );
                *idx = next_idx;
                return;
            }
        }
    }

    /// Returns endpoint statistics
    pub async fn stats(&self) -> Vec<EndpointInfo> {
        self.endpoints.read().await.clone()
    }

    /// Gets a cached response if valid
    pub fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        if !self.config.enable_cache {
            return None;
        }
        self.cache.get(key).and_then(|entry| {
            if entry.is_valid() {
                Some(entry.data.clone())
            } else {
                None
            }
        })
    }

    /// Caches a response
    pub fn cache_response(&self, key: String, data: Vec<u8>) {
        if !self.config.enable_cache {
            return;
        }
        self.cache.insert(key, CachedResponse {
            data,
            cached_at: Instant::now(),
            ttl: Duration::from_secs(self.config.cache_ttl_secs),
        });
    }

    /// Clears expired cache entries
    pub fn clear_expired_cache(&self) {
        self.cache.retain(|_, v| v.is_valid());
    }
}

// ============================================================================
// HTTP Client with Connection Pooling
// ============================================================================

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Maximum idle connections per host
    pub pool_max_idle_per_host: usize,
    /// Idle connection timeout
    pub pool_idle_timeout_secs: u64,
    /// Connection timeout
    pub connect_timeout_secs: u64,
    /// Request timeout
    pub request_timeout_secs: u64,
    /// User agent string
    pub user_agent: String,
    /// Enable gzip compression
    pub gzip: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            pool_max_idle_per_host: 10,
            pool_idle_timeout_secs: 90,
            connect_timeout_secs: 10,
            request_timeout_secs: 30,
            user_agent: format!("WalletD/{}", env!("CARGO_PKG_VERSION")),
            gzip: true,
        }
    }
}

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Burst size (max requests in a burst)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }
}

/// RPC request payload
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest<T: Serialize> {
    /// JSON-RPC version
    pub jsonrpc: &'static str,
    /// Method name
    pub method: String,
    /// Parameters
    pub params: T,
    /// Request ID
    pub id: u64,
}

impl<T: Serialize> JsonRpcRequest<T> {
    /// Creates a new JSON-RPC request
    pub fn new(method: impl Into<String>, params: T, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            method: method.into(),
            params,
            id,
        }
    }
}

/// RPC response payload
#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonRpcResponse<T> {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Response ID
    pub id: u64,
    /// Result (if successful)
    pub result: Option<T>,
    /// Error (if failed)
    pub error: Option<JsonRpcError>,
}

/// RPC error
#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i64,
    /// Error message
    pub message: String,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

/// HTTP client with connection pooling and rate limiting
pub struct RpcClient {
    client: Client,
    rate_limiter: Option<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    request_id: std::sync::atomic::AtomicU64,
}

impl RpcClient {
    /// Creates a new RPC client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(HttpClientConfig::default(), None)
    }

    /// Creates a new RPC client with custom configuration
    pub fn with_config(
        http_config: HttpClientConfig,
        rate_limit: Option<RateLimitConfig>,
    ) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(http_config.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(http_config.pool_idle_timeout_secs))
            .connect_timeout(Duration::from_secs(http_config.connect_timeout_secs))
            .timeout(Duration::from_secs(http_config.request_timeout_secs))
            .user_agent(&http_config.user_agent)
            .gzip(http_config.gzip)
            .build()
            .map_err(|e: reqwest::Error| ProviderError::ConnectionFailed(e.to_string()))?;

        let rate_limiter = rate_limit.map(|config| {
            let quota = Quota::per_second(NonZeroU32::new(config.requests_per_second).unwrap())
                .allow_burst(NonZeroU32::new(config.burst_size).unwrap());
            RateLimiter::direct(quota)
        });

        Ok(Self {
            client,
            rate_limiter,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Makes a JSON-RPC request
    pub async fn rpc_call<P, R>(&self, url: &str, method: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        // Check rate limit
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let request = JsonRpcRequest::new(method, params, id);

        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse<R> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(ProviderError::RpcError {
                code: error.code,
                message: error.message,
            });
        }

        rpc_response.result.ok_or_else(|| ProviderError::RpcError {
            code: -1,
            message: "No result in response".to_string(),
        })
    }

    /// Makes a raw POST request with JSON body
    pub async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        body: impl Serialize,
    ) -> Result<T> {
        // Check rate limit
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;

        let result: T = response.json().await?;
        Ok(result)
    }

    /// Makes a GET request
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        // Check rate limit
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self.client.get(url).send().await?;
        let result: T = response.json().await?;
        Ok(result)
    }

    /// Makes a GET request and returns raw bytes
    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        // Check rate limit
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }

        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Returns the number of requests made
    pub fn request_count(&self) -> u64 {
        self.request_id.load(std::sync::atomic::Ordering::SeqCst) - 1
    }
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new().expect("Failed to create RPC client")
    }
}

impl std::fmt::Debug for RpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcClient")
            .field("request_count", &self.request_count())
            .field("has_rate_limiter", &self.rate_limiter.is_some())
            .finish()
    }
}

// ============================================================================
// Provider with HTTP Client
// ============================================================================

/// A provider with integrated HTTP client
pub struct HttpProvider {
    managed: Arc<ManagedProvider>,
    client: RpcClient,
}

impl HttpProvider {
    /// Creates a new HTTP provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let managed = ManagedProvider::new(config.clone())?;
        
        let http_config = HttpClientConfig {
            request_timeout_secs: config.timeout_secs,
            ..Default::default()
        };
        
        let rate_limit = Some(RateLimitConfig::default());
        let client = RpcClient::with_config(http_config, rate_limit)?;

        Ok(Self {
            managed: Arc::new(managed),
            client,
        })
    }

    /// Makes an RPC call with automatic failover
    pub async fn rpc_call<P, R>(&self, method: &str, params: P) -> Result<R>
    where
        P: Serialize + Clone,
        R: DeserializeOwned,
    {
        let start = Instant::now();
        let url = self.managed.current_url().await;
        
        match self.client.rpc_call(&url, method, params.clone()).await {
            Ok(result) => {
                let elapsed = start.elapsed().as_millis() as u64;
                self.managed.record_success(elapsed).await;
                Ok(result)
            }
            Err(e) => {
                self.managed.record_failure().await;
                
                // Try failover
                let new_url = self.managed.current_url().await;
                if new_url != url {
                    tracing::info!("Retrying with failover endpoint: {}", new_url);
                    self.client.rpc_call(&new_url, method, params).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Returns endpoint statistics
    pub async fn stats(&self) -> Vec<EndpointInfo> {
        self.managed.stats().await
    }

    /// Returns the underlying RPC client
    pub fn client(&self) -> &RpcClient {
        &self.client
    }
}

/// Provider pool for managing multiple chain providers
#[derive(Debug, Default)]
pub struct ProviderPool {
    providers: DashMap<String, Arc<ManagedProvider>>,
}

impl ProviderPool {
    /// Creates a new provider pool
    pub fn new() -> Self {
        Self {
            providers: DashMap::new(),
        }
    }

    /// Adds a provider to the pool
    pub fn add(&self, name: impl Into<String>, config: ProviderConfig) -> Result<()> {
        let provider = ManagedProvider::new(config)?;
        self.providers.insert(name.into(), Arc::new(provider));
        Ok(())
    }

    /// Gets a provider by name
    pub fn get(&self, name: &str) -> Result<Arc<ManagedProvider>> {
        self.providers
            .get(name)
            .map(|r| r.clone())
            .ok_or_else(|| ProviderError::NotFound(name.to_string()))
    }

    /// Removes a provider from the pool
    pub fn remove(&self, name: &str) -> Option<Arc<ManagedProvider>> {
        self.providers.remove(name).map(|(_, v)| v)
    }

    /// Returns all provider names
    pub fn names(&self) -> Vec<String> {
        self.providers.iter().map(|r| r.key().clone()).collect()
    }

    /// Checks health of all providers
    pub async fn health_check_all(&self) -> Vec<(String, Vec<EndpointInfo>)> {
        let mut results = Vec::new();
        for entry in self.providers.iter() {
            let stats = entry.value().stats().await;
            results.push((entry.key().clone(), stats));
        }
        results
    }

    /// Clears all expired caches
    pub fn clear_expired_caches(&self) {
        for entry in self.providers.iter() {
            entry.value().clear_expired_cache();
        }
    }
}

/// Common provider presets for popular networks
pub mod presets {
    use super::ProviderConfig;

    /// Ethereum Mainnet provider configuration
    pub fn ethereum_mainnet() -> ProviderConfig {
        ProviderConfig::new("https://eth.llamarpc.com")
            .with_fallback("https://rpc.ankr.com/eth")
            .with_fallback("https://ethereum.publicnode.com")
            .with_timeout(30)
            .with_max_retries(3)
    }

    /// Ethereum Sepolia testnet provider configuration
    pub fn ethereum_sepolia() -> ProviderConfig {
        ProviderConfig::new("https://rpc.sepolia.org")
            .with_fallback("https://sepolia.drpc.org")
            .with_timeout(30)
    }

    /// Base Mainnet provider configuration
    pub fn base_mainnet() -> ProviderConfig {
        ProviderConfig::new("https://mainnet.base.org")
            .with_fallback("https://base.llamarpc.com")
            .with_timeout(30)
    }

    /// Base Sepolia testnet provider configuration
    pub fn base_sepolia() -> ProviderConfig {
        ProviderConfig::new("https://sepolia.base.org")
            .with_timeout(30)
    }

    /// Solana Mainnet provider configuration
    pub fn solana_mainnet() -> ProviderConfig {
        ProviderConfig::new("https://api.mainnet-beta.solana.com")
            .with_timeout(30)
    }

    /// Solana Devnet provider configuration
    pub fn solana_devnet() -> ProviderConfig {
        ProviderConfig::new("https://api.devnet.solana.com")
            .with_timeout(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new("https://eth.llamarpc.com")
            .with_fallback("https://rpc.ankr.com/eth")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.all_urls().len(), 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let config = ProviderConfig::new("not-a-valid-url");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_endpoint_info() {
        let mut info = EndpointInfo::new("https://example.com".into());
        assert_eq!(info.health, EndpointHealth::Unknown);
        assert_eq!(info.success_rate(), 1.0);

        info.record_success(100);
        assert_eq!(info.health, EndpointHealth::Healthy);
        assert_eq!(info.total_requests, 1);

        info.record_failure();
        assert_eq!(info.total_failures, 1);
        assert_eq!(info.success_rate(), 0.5);
    }

    #[test]
    fn test_provider_pool() {
        let pool = ProviderPool::new();
        
        pool.add("ethereum", presets::ethereum_mainnet()).unwrap();
        pool.add("base", presets::base_mainnet()).unwrap();

        assert!(pool.get("ethereum").is_ok());
        assert!(pool.get("base").is_ok());
        assert!(pool.get("solana").is_err());

        assert_eq!(pool.names().len(), 2);
    }

    #[tokio::test]
    async fn test_managed_provider() {
        let config = ProviderConfig::new("https://eth.llamarpc.com")
            .with_fallback("https://rpc.ankr.com/eth");
        
        let provider = ManagedProvider::new(config).unwrap();
        
        let url = provider.current_url().await;
        assert!(url.contains("llamarpc"));

        provider.record_success(50).await;
        let stats = provider.stats().await;
        assert_eq!(stats[0].total_requests, 1);
        assert_eq!(stats[0].health, EndpointHealth::Healthy);
    }

    #[test]
    fn test_http_client_config() {
        let config = HttpClientConfig::default();
        assert_eq!(config.pool_max_idle_per_host, 10);
        assert_eq!(config.request_timeout_secs, 30);
        assert!(config.gzip);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[test]
    fn test_rpc_client_creation() {
        let client = RpcClient::new();
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.request_count(), 0);
    }

    #[test]
    fn test_rpc_client_with_rate_limit() {
        let http_config = HttpClientConfig::default();
        let rate_limit = Some(RateLimitConfig {
            requests_per_second: 5,
            burst_size: 10,
        });
        
        let client = RpcClient::with_config(http_config, rate_limit);
        assert!(client.is_ok());
    }

    #[test]
    fn test_json_rpc_request() {
        let request = JsonRpcRequest::new("eth_blockNumber", Vec::<()>::new(), 1);
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "eth_blockNumber");
        assert_eq!(request.id, 1);
        
        // Should serialize correctly
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("eth_blockNumber"));
        assert!(json.contains("\"id\":1"));
    }

    #[test]
    fn test_http_provider_creation() {
        let config = ProviderConfig::new("https://eth.llamarpc.com")
            .with_timeout(30);
        
        let provider = HttpProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_failover_on_failure() {
        let config = ProviderConfig::new("https://primary.example.com")
            .with_fallback("https://fallback.example.com");
        
        let provider = ManagedProvider::new(config).unwrap();
        
        // Initial URL should be primary
        let url1 = provider.current_url().await;
        assert!(url1.contains("primary"));
        
        // Record failures to trigger failover
        for _ in 0..5 {
            provider.record_failure().await;
        }
        
        // Should have failed over to fallback
        let url2 = provider.current_url().await;
        assert!(url2.contains("fallback"));
    }

    #[test]
    fn test_cache_operations() {
        let config = ProviderConfig::new("https://example.com")
            .with_cache(true)
            .with_cache_ttl(1); // 1 second TTL
        
        let provider = ManagedProvider::new(config).unwrap();
        
        // Cache should be empty
        assert!(provider.get_cached("key1").is_none());
        
        // Cache a response
        provider.cache_response("key1".to_string(), vec![1, 2, 3]);
        
        // Should be able to retrieve it
        let cached = provider.get_cached("key1");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_endpoint_health_tracking() {
        let mut info = EndpointInfo::new("https://example.com".into());
        
        // Record several successes
        for i in 0..10 {
            info.record_success(100 + i * 10);
        }
        
        assert_eq!(info.total_requests, 10);
        assert_eq!(info.total_failures, 0);
        assert_eq!(info.health, EndpointHealth::Healthy);
        assert!(info.avg_response_ms > 0);
        
        // Record failures to degrade health
        for _ in 0..5 {
            info.record_failure();
        }
        
        assert_eq!(info.total_requests, 15);
        assert_eq!(info.total_failures, 5);
        // 5/15 = 33% failure rate, should be degraded
        assert_eq!(info.health, EndpointHealth::Degraded);
    }

    #[test]
    fn test_presets() {
        let eth = presets::ethereum_mainnet();
        assert!(eth.url.contains("llama"));
        assert!(!eth.fallback_urls.is_empty());
        
        let base = presets::base_mainnet();
        assert!(base.url.contains("base.org"));
        
        let sol = presets::solana_mainnet();
        assert!(sol.url.contains("solana.com"));
    }
}
