# Prasaga Avio

Prasaga Avio DAG-based blockchain support.

## Features

- HD wallet derivation
- XPRT token transfers
- High throughput

## Quick Start
```rust
use walletd::prasaga::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Integration Status

- Phase 1: Core wallet functionality
- Phase 2: Transaction broadcasting
- Phase 3: Full SDK integration
