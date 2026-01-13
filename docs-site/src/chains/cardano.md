# Cardano

Cardano blockchain support.

## Features

- HD wallet derivation
- ADA transfers
- Native tokens

## Quick Start
```rust
use walletd::cardano::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
