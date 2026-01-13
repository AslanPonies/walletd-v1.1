# TON

The Open Network support.

## Features

- HD wallet derivation
- TON transfers
- Smart contracts

## Quick Start
```rust
use walletd::ton::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
