# WalletD 🦀

[![Crates.io](https://img.shields.io/crates/v/walletd.svg)](https://crates.io/crates/walletd)
[![Documentation](https://docs.rs/walletd/badge.svg)](https://docs.rs/walletd)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Build Status](https://github.com/walletd/walletd/workflows/CI/badge.svg)](https://github.com/walletd/walletd/actions)
[![codecov](https://codecov.io/gh/walletd/walletd/branch/main/graph/badge.svg)](https://codecov.io/gh/walletd/walletd)
[![Test Coverage](https://img.shields.io/badge/tests-267%20passing-brightgreen.svg)]()

**A unified multi-chain cryptocurrency wallet SDK for Rust.**

Build wallet applications that support Bitcoin, Ethereum, Solana, Monero, and more — all with a consistent API and minimal dependencies.

## Features

- 🔗 **10+ Blockchains** - Bitcoin, Ethereum, Solana, Base L2, ICP, Hedera, Monero, and more
- 🎯 **Unified API** - Common traits across all chains (`Wallet`, `Transferable`, `Syncable`)
- 🔒 **Security First** - Constant-time operations, zeroize-on-drop, no unsafe code
- 📦 **Modular** - Include only the chains you need with feature flags
- ⚡ **Async Ready** - Full async/await support for network operations
- 🧪 **Well Tested** - 267+ unit tests, 19 integration tests

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
# Include only what you need
walletd = { version = "0.3", features = ["bitcoin", "ethereum"] }
```

### Create a Bitcoin Wallet

```rust
use walletd::bitcoin::{BitcoinWallet, Network};
use bdk::keys::bip39::Mnemonic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = Mnemonic::generate(12)?;
    
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()?;
    
    println!("Address: {}", wallet.receive_address()?);
    Ok(())
}
```

### Create an Ethereum Wallet

```rust
use walletd::ethereum::EthereumWallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = EthereumWallet::random();
    
    println!("Address: {}", wallet.address());
    
    // Connect to a provider
    wallet.connect("https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY")?;
    
    let balance = wallet.get_balance().await?;
    println!("Balance: {} ETH", balance.eth());
    
    Ok(())
}
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `default` | Core traits only | minimal |
| `bitcoin` | Bitcoin (BIP32/39/44, SegWit) | ~2MB |
| `ethereum` | Ethereum (EIP-1559, ERC-20) | ~3MB |
| `solana` | Solana (SPL tokens) | ~2MB |
| `base` | Base L2 (Coinbase) | ~1MB |
| `erc20` | ERC-20 tokens | requires `ethereum` |
| `icp` | Internet Computer | ~2MB |
| `hedera` | Hedera Hashgraph | ~3MB |
| `monero` | Monero (privacy) | ~4MB |
| `evm` | All EVM chains | `ethereum` + `base` + `erc20` |
| `all-chains` | Everything | ~15MB |
| `full` | All + async runtime | ~16MB |

### Recommended Configurations

```toml
# Minimal EVM wallet
walletd = { version = "0.3", features = ["ethereum"] }

# Multi-chain DeFi
walletd = { version = "0.3", features = ["evm", "solana"] }

# Privacy-focused
walletd = { version = "0.3", features = ["bitcoin", "monero"] }

# Full SDK
walletd = { version = "0.3", features = ["full"] }
```

## Supported Chains

| Chain | Package | Features |
|-------|---------|----------|
| **Bitcoin** | `walletd_bitcoin` | HD wallets, SegWit, Taproot |
| **Ethereum** | `walletd_ethereum` | EIP-1559, ERC-20, ENS |
| **Solana** | `walletd_solana` | SPL tokens, staking |
| **Base L2** | `walletd_base` | Low-cost EVM transactions |
| **ERC-20** | `walletd_erc20` | USDC, USDT, custom tokens |
| **ICP** | `walletd_icp` | Canisters, ICRC tokens |
| **Hedera** | `walletd_hedera` | HBAR, HTS tokens |
| **Monero** | `walletd_monero` | Privacy, subaddresses |
| **Prasaga** | `walletd_prasaga_avio` | Object-oriented blockchain |

## Core Traits

All wallets implement unified traits for consistent usage:

```rust
use walletd::prelude::*;

// Works with any wallet type
async fn check_balance<W: Wallet>(wallet: &W) -> Result<Amount, WalletError> {
    let balance = wallet.balance().await?;
    println!("{}: {} {}", 
        wallet.address(), 
        balance.human_readable(),
        wallet.currency_symbol()
    );
    Ok(balance)
}
```

### Available Traits

| Trait | Purpose |
|-------|---------|
| `Wallet` | Address, balance, network info |
| `Transferable` | Send funds, estimate fees |
| `Syncable` | Sync with blockchain |
| `HDWallet` | Derivation paths, multiple addresses |
| `TokenWallet` | Token balances and transfers |
| `Signable` | Message signing and verification |

## Security

WalletD is designed with security as a priority:

- ✅ **No unsafe code** - `#![forbid(unsafe_code)]`
- ✅ **Constant-time operations** - Prevents timing attacks
- ✅ **Zeroize on drop** - Sensitive data cleared from memory
- ✅ **Audited dependencies** - `cargo-deny` integration
- ✅ **No network by default** - Explicit provider connections

```rust
use walletd::prelude::*;

// Private keys are automatically zeroized when dropped
{
    let wallet = BitcoinWallet::new()?;
    // ... use wallet
} // <- Private key memory is securely cleared here
```

## Examples

### Multi-Chain Portfolio

```rust
use walletd::prelude::*;

#[cfg(feature = "bitcoin")]
use walletd::bitcoin::BitcoinWallet;

#[cfg(feature = "ethereum")]
use walletd::ethereum::EthereumWallet;

async fn portfolio_value() -> Result<f64, WalletError> {
    let mut total_usd = 0.0;
    
    #[cfg(feature = "bitcoin")]
    {
        let btc = BitcoinWallet::from_mnemonic(MNEMONIC)?;
        let balance = btc.balance().await?;
        total_usd += balance.human_readable() * btc_price;
    }
    
    #[cfg(feature = "ethereum")]
    {
        let eth = EthereumWallet::from_mnemonic(MNEMONIC)?;
        let balance = eth.balance().await?;
        total_usd += balance.human_readable() * eth_price;
    }
    
    Ok(total_usd)
}
```

### ERC-20 Token Transfer

```rust
use walletd::erc20::UsdcAdapter;

async fn send_usdc(to: &str, amount: f64) -> Result<TxHash, WalletError> {
    let usdc = UsdcAdapter::mainnet();
    
    // Check balance first
    let balance = usdc.balance_of(RPC_URL, MY_ADDRESS).await?;
    println!("USDC Balance: {}", balance);
    
    // Transfer tokens
    let tx = usdc.transfer(to, amount).await?;
    Ok(tx)
}
```

## Documentation

- 📚 [API Documentation](https://docs.rs/walletd)
- 📖 [User Guide](https://github.com/walletd/walletd/blob/main/docs/API.md)
- 💡 [Examples](https://github.com/walletd/walletd/tree/main/examples)

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific chain tests
cargo test -p walletd_bitcoin
cargo test -p walletd_ethereum

# Run integration tests (requires network)
cargo test --workspace -- --ignored

# Run benchmarks
cargo bench -p walletd --features "bitcoin,monero"
```

## Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage for all crates
./scripts/coverage.sh

# Run coverage for specific crate
./scripts/coverage.sh walletd_bitcoin

# View HTML report
open coverage/tarpaulin-report.html
```

Coverage reports are automatically generated on CI and uploaded to [Codecov](https://codecov.io/gh/walletd/walletd).

## Minimum Supported Rust Version

WalletD requires **Rust 1.70** or later.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/walletd/walletd
cd walletd
cargo build --workspace
cargo test --workspace
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Built on the shoulders of giants:

- [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
- [alloy](https://github.com/alloy-rs/alloy)
- [solana-sdk](https://github.com/solana-labs/solana)
- [ic-agent](https://github.com/dfinity/agent-rs)
- [monero-rs](https://github.com/monero-rs/monero-rs)

---

**Built with 🦀 by the WalletD team**
