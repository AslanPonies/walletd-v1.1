# walletd

[![Crates.io](https://img.shields.io/crates/v/walletd.svg)](https://crates.io/crates/walletd)
[![Documentation](https://docs.rs/walletd/badge.svg)](https://docs.rs/walletd)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

**Unified multi-chain cryptocurrency wallet SDK for Rust.**

This is the main entry point for the WalletD SDK. Use feature flags to include only the blockchains you need.

## Quick Start

```toml
[dependencies]
walletd = { version = "0.3", features = ["bitcoin", "ethereum"] }
```

```rust
use walletd::prelude::*;
use walletd::bitcoin::BitcoinWallet;
use walletd::ethereum::EthereumWallet;

fn main() {
    println!("WalletD v{}", walletd::version());
    println!("Enabled chains: {:?}", walletd::enabled_chains());
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `core` | Core traits only (default) |
| `bitcoin` | Bitcoin wallet support |
| `ethereum` | Ethereum wallet support |
| `solana` | Solana wallet support |
| `base` | Base L2 wallet support |
| `erc20` | ERC-20 token support |
| `icp` | Internet Computer support |
| `hedera` | Hedera Hashgraph support |
| `monero` | Monero privacy coin support |
| `evm` | All EVM chains |
| `all-chains` | All supported chains |
| `full` | Everything |

## Documentation

See the [main repository](https://github.com/walletd/walletd) for full documentation.

## License

MIT OR Apache-2.0
