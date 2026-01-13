# Arbitrum

> **Optimistic rollup L2** — Ethereum security, lower fees.

## Quick Start

```rust
use walletd::arbitrum::{derive_address, get_balance};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = get_balance(&address, "https://arb1.arbitrum.io/rpc").await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Chain ID | 42161 |
| Native Token | ETH |
| RPC | `https://arb1.arbitrum.io/rpc` |

## Resources

- [Arbitrum Docs](https://docs.arbitrum.io)
- [Arbiscan](https://arbiscan.io)
