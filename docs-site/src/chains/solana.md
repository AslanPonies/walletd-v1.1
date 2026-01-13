# Solana

High-performance Solana blockchain support.

## Features

- HD wallet derivation
- SPL token support
- Transaction creation
- Staking support

## Quick Start
```rust
use walletd::solana::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
