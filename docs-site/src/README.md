# WalletD

> **The Multi-Chain Wallet SDK** — Ship crypto features in hours, not months.

```rust
// 3 lines to check a Bitcoin balance
let address = walletd::bitcoin::derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = walletd::bitcoin::get_balance(&address, BLOCKSTREAM_API).await?;
println!("Balance: {} BTC", sats_to_btc(balance.parse()?));
```

---

## Why WalletD?

**The problem**: Building a multi-chain wallet means learning 18 different SDKs, APIs, address formats, and transaction structures.

**The solution**: One unified API across all chains.

| Without WalletD | With WalletD |
|-----------------|--------------|
| Learn 18 different libraries | Learn 1 API |
| Handle 18 error formats | Unified error handling |
| 18 different key derivation methods | One `derive_address()` call |
| Weeks of integration work | Hours to production |

---

## 60-Second Quickstart

**Goal**: See your first balance in under 60 seconds.

### Step 1: Add dependency (10 sec)

```toml
# Cargo.toml
[dependencies]
walletd = "1.4"
tokio = { version = "1", features = ["full"] }
```

### Step 2: Run this code (50 sec)

```rust
use walletd::{bitcoin, ethereum, solana};
use walletd::types::WalletMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test mnemonic - NEVER use for real funds
    let mnemonic = "abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon abandon abandon about";
    
    // Derive addresses (same mnemonic, different chains)
    let btc = bitcoin::derive_address(mnemonic, WalletMode::Mainnet)?;
    let eth = ethereum::derive_address(mnemonic, WalletMode::Mainnet)?;
    let sol = solana::derive_address(mnemonic, WalletMode::Mainnet)?;
    
    println!("Bitcoin:  {}", btc);
    println!("Ethereum: {}", eth);
    println!("Solana:   {}", sol);
    
    // Check a real balance (this address has funds)
    let balance = bitcoin::get_balance(
        "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq",
        "https://blockstream.info/api"
    ).await?;
    
    println!("\nBalance: {} satoshis", balance);
    
    Ok(())
}
```

**Expected output:**
```
Bitcoin:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
Ethereum: 0x9858EfFD232B4033E47d90003D41EC34EcaEda94
Solana:   5K4RmBDJhqJsT4xMz1gKqeqmGLnPE5aDfEnCgfVWuujD

Balance: 1234567 satoshis
```

🎉 **You just queried Bitcoin in 3 lines of code.**

---

## Supported Chains (18)

| Chain | Type | Status | Guide |
|-------|------|--------|-------|
| **Bitcoin** | L1 | ✅ Full | [Deep Dive →](./chains/bitcoin.md) |
| **Ethereum** | L1 | ✅ Full | [Deep Dive →](./chains/ethereum.md) |
| **Solana** | L1 | ✅ Full | [Guide →](./chains/solana.md) |
| **Base** | L2 | ✅ Full | [Guide →](./chains/base.md) |
| **Polygon** | L2 | ✅ Full | [Guide →](./chains/polygon.md) |
| **Arbitrum** | L2 | ✅ Full | [Guide →](./chains/arbitrum.md) |
| **Avalanche** | L1 | ✅ Full | [Guide →](./chains/avalanche.md) |
| **Cardano** | L1 | ✅ Full | [Guide →](./chains/cardano.md) |
| **Polkadot** | L1 | ✅ Full | [Guide →](./chains/polkadot.md) |
| **Cosmos** | L1 | ✅ Full | [Guide →](./chains/cosmos.md) |
| **ICP** | L1 | ✅ Full | [Guide →](./chains/icp.md) |
| **Hedera** | L1 | ✅ Full | [Guide →](./chains/hedera.md) |
| **Monero** | L1 | ✅ Full | [Guide →](./chains/monero.md) |
| **NEAR** | L1 | ✅ Full | [Guide →](./chains/near.md) |
| **Tron** | L1 | ✅ Full | [Guide →](./chains/tron.md) |
| **Sui** | L1 | ✅ Full | [Guide →](./chains/sui.md) |
| **Aptos** | L1 | ✅ Full | [Guide →](./chains/aptos.md) |
| **TON** | L1 | ✅ Full | [Guide →](./chains/ton.md) |

---

## What Can You Build?

| Recipe | Time | Difficulty |
|--------|------|------------|
| [💰 Payment Gateway](./recipes/payment-gateway.md) | 2 hours | Beginner |
| [📊 Portfolio Tracker](./recipes/portfolio-tracker.md) | 3 hours | Beginner |
| [🔐 Multisig Treasury](./recipes/multisig-treasury.md) | 4 hours | Intermediate |
| [📈 Staking Dashboard](./recipes/staking-dashboard.md) | 3 hours | Intermediate |
| [🏦 Exchange Integration](./recipes/exchange-integration.md) | 6 hours | Advanced |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        YOUR APPLICATION                          │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    ▼                       ▼
         ┌─────────────────┐     ┌─────────────────┐
         │   WalletD SDK   │     │   WalletD CLI   │
         │   (Library)     │     │   (Standalone)  │
         └─────────────────┘     └─────────────────┘
                    │
    ┌───────────────┼───────────────┬───────────────┐
    ▼               ▼               ▼               ▼
┌───────┐     ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Wallet│     │ Broadcast│    │ Hardware │    │ Staking  │
│ Core  │     │ Engine   │    │ Wallets  │    │ Module   │
└───────┘     └──────────┘    └──────────┘    └──────────┘
    │               │               │               │
    └───────────────┴───────────────┴───────────────┘
                                │
    ┌─────────┬─────────┬───────┴───┬─────────┬─────────┐
    ▼         ▼         ▼           ▼         ▼         ▼
┌──────┐ ┌──────┐ ┌──────┐    ┌──────┐ ┌──────┐ ┌──────┐
│ BTC  │ │ ETH  │ │ SOL  │    │ DOT  │ │ ATOM │ │ ...  │
│ RPC  │ │ RPC  │ │ RPC  │    │ RPC  │ │ RPC  │ │      │
└──────┘ └──────┘ └──────┘    └──────┘ └──────┘ └──────┘
```

---

## Installation

### Library (for Rust projects)

```toml
[dependencies]
walletd = "1.4"
tokio = { version = "1", features = ["full"] }
```

### CLI (standalone tool)

```bash
git clone https://github.com/AslanPonies/walletd-v1.1
cd walletd-v1.1/walletd-cli
cargo build --release
./target/release/walletd
```

### System Requirements

| Platform | Requirements |
|----------|--------------|
| macOS | `brew install openssl` |
| Ubuntu | `apt install libssl-dev pkg-config` |
| Windows | Use WSL2 |

---

## Quick Links

| I want to... | Go to... |
|--------------|----------|
| Get started immediately | [60-Second Quickstart](#60-second-quickstart) |
| Understand Bitcoin support | [Bitcoin Deep Dive](./chains/bitcoin.md) |
| Build a payment system | [Payment Gateway Recipe](./recipes/payment-gateway.md) |
| See all API functions | [API Reference](./api-reference.md) |
| Fix an error | [Troubleshooting](./troubleshooting.md) |
| Compare to alternatives | [Why WalletD?](./comparison.md) |

---

## Security

- ✅ BIP-32/39/44 compliant key derivation
- ✅ No private keys stored by the SDK
- ✅ Uses audited crypto libraries (k256, ed25519-dalek)
- ✅ Memory-safe Rust implementation
- 🔄 External audit in progress

**[Read Security Guide →](./security.md)**

---

## License

MIT / Apache-2.0

---

<p align="center">
  <strong>Built for developers who ship.</strong><br>
  <a href="https://github.com/AslanPonies/walletd-v1.1">GitHub</a> •
  <a href="./guides/quickstart.md">Documentation</a> •
  <a href="https://github.com/AslanPonies/walletd-v1.1/issues">Issues</a>
</p>
