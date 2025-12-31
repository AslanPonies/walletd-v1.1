# walletd_arbitrum

[![Crates.io](https://img.shields.io/crates/v/walletd_arbitrum.svg)](https://crates.io/crates/walletd_arbitrum)
[![Documentation](https://docs.rs/walletd_arbitrum/badge.svg)](https://docs.rs/walletd_arbitrum)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Arbitrum L2 blockchain support for the WalletD SDK.

## Features

- 🚀 **Layer 2** - Fast, cheap transactions on Arbitrum
- 🔑 **HD Wallets** - BIP-39/BIP-32 mnemonic support
- ✍️ **Signing** - Transaction and message signing
- 🌐 **Multi-Network** - Arbitrum One, Nova, and Sepolia testnet
- 🔄 **EVM Compatible** - Full Ethereum compatibility

## Quick Start

```rust
use walletd_arbitrum::{ArbitrumWallet, NetworkConfig};

// Create random wallet for mainnet
let wallet = ArbitrumWallet::mainnet()?;
println!("Address: {}", wallet.address());

// Or from mnemonic
let mnemonic = "abandon abandon abandon ...";
let wallet = ArbitrumWallet::from_mnemonic(mnemonic, 42161)?;

// Connect to RPC
let mut wallet = ArbitrumWallet::mainnet()?;
wallet.connect("https://arb1.arbitrum.io/rpc");

// Get balance
let balance = wallet.get_balance().await?;
```

## Networks

| Network | Chain ID | Description |
|---------|----------|-------------|
| Arbitrum One | 42161 | Main production network |
| Arbitrum Nova | 42170 | AnyTrust chain for gaming/social |
| Arbitrum Sepolia | 421614 | Testnet |

```rust
use walletd_arbitrum::NetworkConfig;

// Mainnet
let config = NetworkConfig::mainnet();
assert_eq!(config.chain_id, 42161);

// Nova (gaming-optimized)
let nova = NetworkConfig::nova();
assert_eq!(nova.chain_id, 42170);

// Testnet
let testnet = NetworkConfig::sepolia();
```

## Wallet Operations

### Create Wallet

```rust
// Random wallet
let wallet = ArbitrumWallet::mainnet()?;

// From mnemonic
let wallet = ArbitrumWallet::from_mnemonic(mnemonic, 42161)?;

// From mnemonic with specific index
let wallet = ArbitrumWallet::from_mnemonic_with_index(mnemonic, 42161, 5)?;

// From private key
let wallet = ArbitrumWallet::from_private_key("0x...", 42161)?;
```

### Send Transactions

```rust
let mut wallet = ArbitrumWallet::mainnet()?;
wallet.connect("https://arb1.arbitrum.io/rpc");

// Send ETH
let tx_hash = wallet.send_eth(
    "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
    U256::from(1_000_000_000_000_000_000u128) // 1 ETH
).await?;

// With data (contract call)
let tx_hash = wallet.send_transaction(
    "0x...",
    U256::ZERO,
    Some(contract_data)
).await?;
```

### Sign Messages

```rust
let signature = wallet.sign_message("Hello Arbitrum!").await?;
// Returns: 0x... (132 char hex string)
```

### Gas Estimation

```rust
let gas = wallet.estimate_gas(to_address, value, None).await?;
let gas_price = wallet.gas_price().await?;
```

## RPC Endpoints

Built-in endpoints for each network:

| Network | Endpoints |
|---------|-----------|
| Mainnet | arb1.arbitrum.io, arbitrum.publicnode.com, rpc.ankr.com/arbitrum |
| Nova | nova.arbitrum.io, arbitrum-nova.publicnode.com |
| Sepolia | sepolia-rollup.arbitrum.io, arbitrum-sepolia.publicnode.com |

## Why Arbitrum?

- **42% L2 market share** - Dominant Layer 2
- **$23.8B TVL** - Highest total value locked
- **$190M gaming grants** - Strong ecosystem funding
- **~0.25s block time** - Near-instant transactions
- **~$0.01-0.10 fees** - 10-100x cheaper than L1

## Integration with WalletD

```toml
[dependencies]
walletd = { version = "0.3", features = ["arbitrum"] }
```

```rust
use walletd::arbitrum::{ArbitrumWallet, NetworkConfig};

let wallet = ArbitrumWallet::mainnet()?;
```

## License

MIT OR Apache-2.0
