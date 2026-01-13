# Avalanche

> **High-speed L1** — Sub-second finality on C-Chain.

## Quick Start

```rust
use walletd::avalanche::{derive_address, get_balance};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = get_balance(&address, "https://api.avax.network/ext/bc/C/rpc").await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Chain ID | 43114 |
| Native Token | AVAX |
| RPC | `https://api.avax.network/ext/bc/C/rpc` |
| Finality | ~1 second |

## Faucet

[faucet.avax.network](https://faucet.avax.network/)

## Resources

- [Avalanche Docs](https://docs.avax.network)
- [Snowtrace](https://snowtrace.io)
