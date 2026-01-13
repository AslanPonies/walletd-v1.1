# Quickstart

Get started with WalletD in 5 minutes.

## Prerequisites

- Rust 1.70+
- Cargo

## Installation

### As a Library
```toml
[dependencies]
walletd = "1.4"
```

### CLI Tool
```bash
git clone https://github.com/AslanPonies/walletd-v1.1
cd walletd-v1.1/walletd-cli
cargo build --release
./target/release/walletd
```

## Create Your First Wallet
```rust
use walletd::prelude::*;

fn main() -> Result<()> {
    // Generate 24-word mnemonic
    let mnemonic = Mnemonic::generate(24)?;
    println!("Mnemonic: {}", mnemonic.phrase());
    
    // Derive addresses for multiple chains
    let btc = walletd::bitcoin::derive_address(&mnemonic, Network::Mainnet)?;
    let eth = walletd::ethereum::derive_address(&mnemonic, Network::Mainnet)?;
    let sol = walletd::solana::derive_address(&mnemonic, Network::Mainnet)?;
    
    println!("Bitcoin:  {}", btc);
    println!("Ethereum: {}", eth);
    println!("Solana:   {}", sol);
    
    Ok(())
}
```

## Using the CLI
```bash
# Launch interactive mode
./walletd

# Select network (Testnet recommended for testing)
# Choose a chain
# Generate or import wallet
```

## Next Steps

- [Installation Guide](./installation.md) - Detailed setup instructions
- [CLI Usage](./cli-usage.md) - Full CLI documentation
- [Bitcoin Guide](../chains/bitcoin.md) - Bitcoin-specific features
