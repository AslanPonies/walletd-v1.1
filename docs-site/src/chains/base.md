# Base

> **Coinbase L2** — Low-cost Ethereum transactions on OP Stack.

## Quick Start

```rust
// Same API as Ethereum
use walletd::base::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = get_balance(&address, "https://mainnet.base.org").await?;
```

## Key Details

| Property | Mainnet | Sepolia |
|----------|---------|---------|
| Chain ID | 8453 | 84532 |
| Native Token | ETH | ETH |
| Block Time | ~2s | ~2s |
| RPC | `https://mainnet.base.org` | `https://sepolia.base.org` |

## Gas Fees

Base L2 fees are ~100x cheaper than Ethereum L1:
- ETH transfer: ~$0.01
- Token transfer: ~$0.05
- Swap: ~$0.10

## Bridging

Bridge ETH from mainnet: [bridge.base.org](https://bridge.base.org)

## Resources

- [Base Docs](https://docs.base.org)
- [BaseScan](https://basescan.org)
