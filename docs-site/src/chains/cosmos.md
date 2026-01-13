# Cosmos

Cosmos Hub support.

## Features

- HD wallet derivation
- ATOM transfers
- IBC support

## Quick Start
```rust
use walletd::cosmos::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
