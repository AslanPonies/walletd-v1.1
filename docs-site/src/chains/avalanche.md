# Avalanche

Avalanche C-Chain support.

## Features

- EVM compatible
- AVAX token support
- Sub-second finality

## Quick Start
```rust
use walletd::avalanche::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
