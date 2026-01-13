# Sui

Sui blockchain support.

## Features

- HD wallet derivation
- SUI transfers
- Object-based model

## Quick Start
```rust
use walletd::sui::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
