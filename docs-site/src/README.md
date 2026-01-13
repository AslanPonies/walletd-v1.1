# WalletD Developer Documentation

WalletD is a **multi-chain cryptocurrency wallet SDK** built in Rust, supporting **19 blockchains** with a unified API.

## Supported Chains

| Chain | Mainnet | Testnet | HD Wallet | Transactions |
|-------|---------|---------|-----------|--------------|
| Bitcoin | ✅ | ✅ | ✅ | ✅ |
| Ethereum | ✅ | ✅ | ✅ | ✅ |
| Solana | ✅ | ✅ | ✅ | ✅ |
| ICP | ✅ | ✅ | ✅ | ✅ |
| Hedera | ✅ | ✅ | ✅ | ✅ |
| Monero | ✅ | ✅ | ✅ | ✅ |
| Base | ✅ | ✅ | ✅ | ✅ |
| Polygon | ✅ | ✅ | ✅ | ✅ |
| Avalanche | ✅ | ✅ | ✅ | ✅ |
| Arbitrum | ✅ | ✅ | ✅ | ✅ |
| Cardano | ✅ | ✅ | ✅ | ✅ |
| Polkadot | ✅ | ✅ | ✅ | ✅ |
| Cosmos | ✅ | ✅ | ✅ | ✅ |
| NEAR | ✅ | ✅ | ✅ | ✅ |
| Tron | ✅ | ✅ | ✅ | ✅ |
| Sui | ✅ | ✅ | ✅ | ✅ |
| Aptos | ✅ | ✅ | ✅ | ✅ |
| TON | ✅ | ✅ | ✅ | ✅ |
| Prasaga Avio | ✅ | ✅ | ✅ | ✅ |

## Quick Install
```bash
# Add to Cargo.toml
[dependencies]
walletd = "1.4"
```

## Quick Example
```rust
use walletd::prelude::*;

fn main() -> Result<()> {
    // Generate a new wallet
    let mnemonic = Mnemonic::generate(24)?;
    
    // Derive Bitcoin address
    let btc_address = walletd::bitcoin::derive_address(&mnemonic, Network::Mainnet)?;
    
    // Derive Ethereum address  
    let eth_address = walletd::ethereum::derive_address(&mnemonic, Network::Mainnet)?;
    
    println!("BTC: {}", btc_address);
    println!("ETH: {}", eth_address);
    
    Ok(())
}
```

## Features

- **HD Wallet Support** - BIP-32/39/44 compliant hierarchical deterministic wallets
- **Multi-Chain** - Single API for 19+ blockchains
- **Hardware Wallets** - Ledger and Trezor support
- **Multi-Signature** - Native multisig for supported chains
- **Staking** - Built-in staking for PoS chains
- **Transaction Broadcasting** - Unified transaction submission

## Links

- [GitHub Repository](https://github.com/AslanPonies/walletd-v1.1)
- [CLI Tool](./guides/cli-usage.md)
- [API Reference](./api-reference.md)
