# Internet Computer (ICP)

DFINITY Internet Computer Protocol support.

## Features

- HD wallet derivation
- ICP transfers
- Rosetta API integration

## Quick Start
```rust
use walletd::icp::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
