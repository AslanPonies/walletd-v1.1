# Polkadot

Polkadot relay chain support.

## Features

- HD wallet derivation
- DOT transfers
- Staking support

## Quick Start
```rust
use walletd::polkadot::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
