# WalletD API Documentation

## Overview

WalletD is a multi-chain cryptocurrency wallet SDK written in Rust. It provides unified interfaces for wallet management across 17+ blockchain networks.

## Supported Chains

| Chain | Symbol | Type | Module |
|-------|--------|------|--------|
| Bitcoin | BTC | UTXO | `walletd_bitcoin` |
| Ethereum | ETH | EVM | `walletd_ethereum` |
| Polygon | POL | EVM | `walletd_polygon` |
| Avalanche | AVAX | EVM | `walletd_avalanche` |
| Base | ETH | EVM L2 | `walletd_base` |
| Arbitrum | ETH | EVM L2 | `walletd_arbitrum` |
| Solana | SOL | Account | `walletd_solana` |
| Cardano | ADA | UTXO | `walletd_cardano` |
| Polkadot | DOT | Substrate | `walletd_polkadot` |
| Cosmos | ATOM | Tendermint | `walletd_cosmos` |
| Near | NEAR | Account | `walletd_near` |
| Tron | TRX | Account | `walletd_tron` |
| Hedera | HBAR | Hashgraph | `walletd_hedera` |
| ICP | ICP | Canister | `walletd_icp` |
| SUI | SUI | Object | `walletd_sui` |
| Aptos | APT | Move | `walletd_aptos` |
| TON | TON | Account | `walletd_ton` |
| Monero | XMR | Privacy | `walletd_monero` |

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
walletd_ethereum = "0.1"
walletd_polygon = "0.1"
walletd-traits = "0.1"
```

### Basic Usage

```rust
use walletd_ethereum::EthereumWallet;
use walletd_polygon::PolygonWallet;

// Create new wallets
let eth = EthereumWallet::mainnet()?;
let polygon = PolygonWallet::mainnet()?;

println!("ETH: {}", eth.address());
println!("Polygon: {}", polygon.address());
```

## Core Types

### Amount

Represents cryptocurrency amounts with decimal precision:

```rust
use walletd_traits::Amount;

// From smallest unit (wei, satoshi, etc.)
let amount = Amount::from_smallest_unit(1_000_000_000_000_000_000, 18);

// From human-readable
let amount = Amount::from_human(1.5, 18); // 1.5 ETH

// Convert back
let wei = amount.smallest_unit();
let eth = amount.human_readable();
```

### Network

```rust
use walletd_traits::Network;

let mainnet = Network::mainnet("Ethereum").with_chain_id(1);
let testnet = Network::testnet("Sepolia").with_chain_id(11155111);
```

## EVM Chains

### Creating Wallets

```rust
// Random wallet
let wallet = EthereumWallet::mainnet()?;

// From mnemonic (BIP-39)
let mnemonic = "abandon abandon abandon...";
let wallet = EthereumWallet::from_mnemonic(mnemonic, 1)?;

// From private key
let wallet = EthereumWallet::from_private_key("0x...", 1)?;
```

### RPC Connection

```rust
let mut wallet = EthereumWallet::mainnet()?;
wallet.connect_provider("https://eth.llamarpc.com")?;

// Get balance
let balance = wallet.get_balance().await?;
let balance_eth = wallet.get_balance_eth().await?;
```

### Transactions

```rust
// Simple transfer
let tx = wallet.send_transaction(to, value).await?;

// With gas settings (EIP-1559)
let tx = wallet.send_transaction_with_gas(
    to,
    value,
    21000,              // gas limit
    50_000_000_000,    // max fee
    2_000_000_000,     // priority fee
).await?;
```

## Non-EVM Chains

### Cardano

```rust
use walletd_cardano::CardanoWallet;

let wallet = CardanoWallet::mainnet()?;
println!("{}", wallet.address()); // addr1...

let sig = wallet.sign(b"message");
assert!(wallet.verify(b"message", &sig));
```

### Cosmos

```rust
use walletd_cosmos::CosmosWallet;

let wallet = CosmosWallet::mainnet()?;
println!("{}", wallet.address()); // cosmos1...
```

### Polkadot

```rust
use walletd_polkadot::PolkadotWallet;

let dot = PolkadotWallet::polkadot()?;  // SS58 prefix 0
let ksm = PolkadotWallet::kusama()?;    // SS58 prefix 2
```

### Near

```rust
use walletd_near::NearWallet;

let wallet = NearWallet::mainnet()?;
println!("{}", wallet.implicit_account_id()); // 64 hex chars

wallet.set_account_id("alice.near");
```

### Tron

```rust
use walletd_tron::TronWallet;

let wallet = TronWallet::mainnet()?;
println!("{}", wallet.address()); // T...

assert!(TronWallet::validate_address(&wallet.address()));
```

## ERC-20 Tokens

### Token Registry

```rust
use walletd_erc20::prelude::*;

let registry = TokenRegistry::with_defaults();

// USDC on different chains
let usdc_eth = registry.get(EvmChain::Ethereum, "USDC");
let usdc_poly = registry.get(EvmChain::Polygon, "USDC");

// All tokens on Ethereum
let tokens = registry.tokens_for_chain(EvmChain::Ethereum);
```

### Supported Tokens

Pre-loaded tokens include:
- USDC, USDT (stablecoins)
- WETH, WBTC (wrapped assets)
- DAI, LINK (DeFi tokens)

## Staking (Traits)

```rust
use walletd_traits::prelude::*;

// Get validators
let validators = wallet.validators().await?;

// Stake
let tx = wallet.stake("validator_address", amount).await?;

// Claim rewards
let tx = wallet.claim_rewards().await?;
```

## DeFi Swaps (Traits)

```rust
// Get quote
let quote = wallet.quote_swap("USDC", "ETH", amount, 0.5).await?;

// Execute swap
let tx = wallet.swap("USDC", "ETH", amount_in, min_out).await?;
```

## Production Resilience

### Circuit Breaker

```rust
use walletd_resilience::*;

let cb = CircuitBreaker::new(
    CircuitBreakerConfig::new("rpc")
        .with_failure_threshold(5)
        .with_reset_timeout(Duration::from_secs(30))
);

let result = cb.execute(|| async {
    wallet.get_balance().await
}).await?;
```

### Retry with Backoff

```rust
let result = with_backoff(
    BackoffConfig::default().with_max_attempts(5),
    || async { wallet.get_balance().await }
).await?;
```

### Health Monitoring

```rust
let checker = HealthChecker::default_config();
checker.record(HealthCheckResult::healthy("rpc", Duration::from_millis(50))).await;
assert!(checker.is_healthy().await);
```

## Error Handling

```rust
use walletd_traits::WalletError;

match result {
    Err(WalletError::InsufficientBalance { have, need }) => {
        println!("Need {} more", need - have);
    }
    Err(WalletError::NetworkError(e)) => {
        println!("Network: {}", e);
    }
    _ => {}
}
```

## Security Notes

1. Private keys use `zeroize` for secure memory cleanup
2. BIP-39 mnemonics, BIP-32/44 derivation
3. Always validate addresses before sending
4. Use hardware wallets for production funds

## Crate Structure

```
walletd/
├── coins/
│   ├── bitcoin/
│   ├── ethereum/
│   ├── polygon/
│   ├── avalanche/
│   ├── cardano/
│   ├── cosmos/
│   ├── polkadot/
│   ├── near/
│   ├── tron/
│   └── ...
├── crates/
│   ├── walletd-traits/      # Core traits
│   ├── walletd-error/       # Error types
│   ├── walletd-resilience/  # Production patterns
│   ├── walletd-provider/    # Connection pooling
│   └── walletd-testing/     # Test utilities
└── docs/
```

## License

MIT License
