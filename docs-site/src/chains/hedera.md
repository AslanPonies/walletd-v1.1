# Hedera

Hedera Hashgraph network support.

## Features

- HD wallet derivation
- HBAR transfers
- Token support

## Quick Start
```rust
use walletd::hedera::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
