# walletd-provider

[![Crates.io](https://img.shields.io/crates/v/walletd-provider.svg)](https://crates.io/crates/walletd-provider)
[![Documentation](https://docs.rs/walletd-provider/badge.svg)](https://docs.rs/walletd-provider)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Connection pooling and provider management for the WalletD SDK.

## Features

- 🔄 **Connection Pooling** - Reuse HTTP connections across requests
- ⚡ **Rate Limiting** - Built-in request rate limiting
- 🔀 **Automatic Failover** - Switch to backup endpoints on failure
- 📊 **Health Tracking** - Monitor endpoint health and latency
- 💾 **Response Caching** - Cache common RPC responses
- 🎯 **Presets** - Pre-configured settings for popular networks

## Quick Start

```rust
use walletd_provider::{ProviderPool, ProviderConfig, presets};

// Create a provider pool
let pool = ProviderPool::new();

// Add providers with presets
pool.add("ethereum", presets::ethereum_mainnet())?;
pool.add("base", presets::base_mainnet())?;

// Get a provider
let provider = pool.get("ethereum")?;

// Make RPC calls
let url = provider.current_url().await;
```

## Custom Configuration

```rust
use walletd_provider::{ProviderConfig, HttpProvider};

let config = ProviderConfig::new("https://eth.llamarpc.com")
    .with_fallback("https://rpc.ankr.com/eth")
    .with_fallback("https://ethereum.publicnode.com")
    .with_timeout(30)
    .with_max_retries(3)
    .with_cache(true)
    .with_cache_ttl(10);

let provider = HttpProvider::new(config)?;

// Make an RPC call with automatic failover
let block_number: String = provider.rpc_call("eth_blockNumber", ()).await?;
```

## Rate Limiting

```rust
use walletd_provider::{RpcClient, HttpClientConfig, RateLimitConfig};

let http_config = HttpClientConfig {
    pool_max_idle_per_host: 10,
    request_timeout_secs: 30,
    ..Default::default()
};

let rate_limit = RateLimitConfig {
    requests_per_second: 10,
    burst_size: 20,
};

let client = RpcClient::with_config(http_config, Some(rate_limit))?;
```

## Health Monitoring

```rust
// Get endpoint statistics
let stats = provider.stats().await;

for endpoint in stats {
    println!("URL: {}", endpoint.url);
    println!("Health: {:?}", endpoint.health);
    println!("Success Rate: {:.1}%", endpoint.success_rate() * 100.0);
    println!("Avg Response: {}ms", endpoint.avg_response_ms);
}
```

## Network Presets

| Network | Preset Function |
|---------|-----------------|
| Ethereum Mainnet | `presets::ethereum_mainnet()` |
| Ethereum Sepolia | `presets::ethereum_sepolia()` |
| Base Mainnet | `presets::base_mainnet()` |
| Base Sepolia | `presets::base_sepolia()` |
| Solana Mainnet | `presets::solana_mainnet()` |
| Solana Devnet | `presets::solana_devnet()` |

## Features

| Feature | Description |
|---------|-------------|
| `default` | Base provider functionality |
| `ethereum` | Alloy-based Ethereum provider |
| `metrics` | Prometheus metrics support |

## Connection Pool Settings

The HTTP client uses these defaults (configurable via `HttpClientConfig`):

| Setting | Default | Description |
|---------|---------|-------------|
| `pool_max_idle_per_host` | 10 | Max idle connections per host |
| `pool_idle_timeout_secs` | 90 | Idle connection timeout |
| `connect_timeout_secs` | 10 | Connection establishment timeout |
| `request_timeout_secs` | 30 | Full request timeout |
| `gzip` | true | Enable gzip compression |

## License

MIT OR Apache-2.0
