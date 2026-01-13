# Ethereum

Full Ethereum support with EIP-1559 transactions and ERC-20 tokens.

## Features

- EIP-1559 transactions
- ERC-20 token support
- Smart contract calls
- Gas estimation

## Quick Start
```rust
use walletd::ethereum::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Networks

- Mainnet: Chain ID 1
- Sepolia: Chain ID 11155111
