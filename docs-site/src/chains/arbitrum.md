# Arbitrum

Arbitrum One L2 support.

## Features

- EVM compatible
- Ethereum L2 scaling
- Low fees

## Quick Start
```rust
use walletd::arbitrum::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
