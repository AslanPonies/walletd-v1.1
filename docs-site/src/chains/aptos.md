# Aptos

> **Move-based L1** — Block-STM parallel execution engine.

## Quick Start

```rust
use walletd::aptos::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "0x..." (64 chars)

let balance = get_balance(&address, "https://fullnode.mainnet.aptoslabs.com").await?;
// → Balance in octas

let tx = send_transaction(&from, &to, octas_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/637'/0'/0'/0'` |
| Address Format | Hex (`0x...`, 64 chars) |
| Native Unit | octa |
| Decimals | 8 (1 APT = 10^8 octas) |

## Unit Conversion

```rust
fn octas_to_apt(octas: u64) -> f64 {
    octas as f64 / 100_000_000.0
}

fn apt_to_octas(apt: f64) -> u64 {
    (apt * 100_000_000.0) as u64
}
```

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://fullnode.mainnet.aptoslabs.com` |
| Testnet | `https://fullnode.testnet.aptoslabs.com` |
| Devnet | `https://fullnode.devnet.aptoslabs.com` |

## Faucet

[aptoslabs.com/testnet-faucet](https://aptoslabs.com/testnet-faucet)

## Resources

- [Aptos Docs](https://aptos.dev)
- [Aptos Explorer](https://explorer.aptoslabs.com)
