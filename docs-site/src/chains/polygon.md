# Polygon

> **Ethereum scaling** — Fast, cheap EVM transactions.

## Quick Start

```rust
use walletd::polygon::{derive_address, get_balance};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = get_balance(&address, "https://polygon-rpc.com").await?;
```

## Key Details

| Property | Mainnet | Mumbai |
|----------|---------|--------|
| Chain ID | 137 | 80001 |
| Native Token | MATIC | MATIC |
| RPC | `https://polygon-rpc.com` | `https://rpc-mumbai.maticvigil.com` |

## Faucet

[faucet.polygon.technology](https://faucet.polygon.technology/)

## Resources

- [Polygon Docs](https://docs.polygon.technology)
- [PolygonScan](https://polygonscan.com)
