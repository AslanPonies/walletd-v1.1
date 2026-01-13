# Sui

> **Move-based L1** — Object-centric model with parallel execution.

## Quick Start

```rust
use walletd::sui::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "0x..." (64 chars)

let balance = get_balance(&address, "https://fullnode.mainnet.sui.io:443").await?;
// → Balance in MIST

let tx = send_transaction(&from, &to, mist_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/784'/0'/0'/0'` |
| Address Format | Hex (`0x...`, 64 chars) |
| Native Unit | MIST |
| Decimals | 9 (1 SUI = 10^9 MIST) |

## Unit Conversion

```rust
fn mist_to_sui(mist: u64) -> f64 {
    mist as f64 / 1_000_000_000.0
}

fn sui_to_mist(sui: f64) -> u64 {
    (sui * 1_000_000_000.0) as u64
}
```

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://fullnode.mainnet.sui.io:443` |
| Testnet | `https://fullnode.testnet.sui.io:443` |
| Devnet | `https://fullnode.devnet.sui.io:443` |

## Faucet

Request from [Discord #devnet-faucet](https://discord.gg/sui)

## Resources

- [Sui Docs](https://docs.sui.io)
- [SuiScan](https://suiscan.xyz)
- [Sui Explorer](https://explorer.sui.io)
