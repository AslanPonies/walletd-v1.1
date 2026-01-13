# Polygon

Polygon PoS network support.

## Features

- EVM compatible
- MATIC token support
- Fast transactions

## Quick Start
```rust
use walletd::polygon::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
