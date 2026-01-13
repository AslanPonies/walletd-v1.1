# TON (The Open Network)

> **Telegram's blockchain** — Infinite sharding with workchains.

## Quick Start

```rust
use walletd::ton::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "EQ..." or "UQ..."

let balance = get_balance(&address, "https://toncenter.com/api/v2/jsonRPC").await?;
// → Balance in nanoton

let tx = send_transaction(&from, &to, nanoton_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/607'/0'` |
| Native Unit | nanoton |
| Decimals | 9 (1 TON = 10^9 nanoton) |

## Unit Conversion

```rust
fn nanoton_to_ton(nanoton: u64) -> f64 {
    nanoton as f64 / 1_000_000_000.0
}

fn ton_to_nanoton(ton: f64) -> u64 {
    (ton * 1_000_000_000.0) as u64
}
```

## Address Types

| Prefix | Type | Use Case |
|--------|------|----------|
| `EQ` | Bounceable | Smart contracts |
| `UQ` | Non-bounceable | User wallets |

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://toncenter.com/api/v2/jsonRPC` |
| Testnet | `https://testnet.toncenter.com/api/v2/jsonRPC` |

## Faucet

Telegram bot: [@testgiver_ton_bot](https://t.me/testgiver_ton_bot)

## Resources

- [TON Docs](https://docs.ton.org)
- [TON Explorer](https://tonscan.org)
- [Tonkeeper](https://tonkeeper.com)
