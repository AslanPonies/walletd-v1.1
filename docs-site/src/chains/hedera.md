# Hedera

> **Enterprise hashgraph** — High throughput with native services.

## Quick Start

```rust
use walletd::hedera::{derive_address, get_balance, send_transaction};

let account = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "0.0.xxxxx"

let balance = get_balance(&account, "https://mainnet.hedera.com").await?;
// → Balance in tinybars

let tx = send_transaction(&from, &to, tinybar_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/3030'/0'/0/0` |
| Native Unit | tinybar |
| Decimals | 8 (1 HBAR = 10^8 tinybars) |

## Unit Conversion

```rust
fn tinybar_to_hbar(tinybar: u64) -> f64 {
    tinybar as f64 / 100_000_000.0
}

fn hbar_to_tinybar(hbar: f64) -> u64 {
    (hbar * 100_000_000.0) as u64
}
```

## Account Format

Hedera uses `shard.realm.account` format:
- **Shard**: Network partition (0)
- **Realm**: Namespace (0)
- **Account**: Unique number

Example: `0.0.12345`

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://mainnet.hedera.com` |
| Testnet | `https://testnet.hedera.com` |

## Faucet

[portal.hedera.com](https://portal.hedera.com)

## Resources

- [Hedera Docs](https://docs.hedera.com)
- [HashScan](https://hashscan.io)
