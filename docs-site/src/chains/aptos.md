# Aptos

Aptos blockchain support.

## Features

- HD wallet derivation
- APT transfers
- Move language

## Quick Start
```rust
use walletd::aptos::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
