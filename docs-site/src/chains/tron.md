# Tron

TRON network support.

## Features

- HD wallet derivation
- TRX transfers
- TRC-20 tokens

## Quick Start
```rust
use walletd::tron::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
