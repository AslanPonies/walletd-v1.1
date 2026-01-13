# Monero

Privacy-focused Monero blockchain support.

## Features

- HD wallet derivation
- Private transactions
- View keys and spend keys

## Quick Start
```rust
use walletd::monero::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
