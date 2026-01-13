# NEAR

NEAR Protocol support.

## Features

- HD wallet derivation
- NEAR transfers
- Named accounts

## Quick Start
```rust
use walletd::near::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
