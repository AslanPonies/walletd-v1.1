# Base

Coinbase Base L2 network support.

## Features

- EVM compatible
- Low transaction fees
- Ethereum bridge support

## Quick Start
```rust
use walletd::base::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Network

- Chain ID: 8453
