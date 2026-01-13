# Monero

> **Privacy-first cryptocurrency** — Untraceable transactions by default.

## Quick Start

```rust
use walletd::monero::{derive_address, get_balance};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "4..." (95 characters)

let balance = get_balance(&address, &rpc).await?;
// → Balance in atomic units
```

## Key Details

| Property | Value |
|----------|-------|
| Key Derivation | Monero-specific (not BIP-44) |
| Native Unit | atomic unit (piconero) |
| Decimals | 12 (1 XMR = 10^12 atomic units) |

## Unit Conversion

```rust
fn atomic_to_xmr(atomic: u64) -> f64 {
    atomic as f64 / 1_000_000_000_000.0
}

fn xmr_to_atomic(xmr: f64) -> u64 {
    (xmr * 1_000_000_000_000.0) as u64
}
```

## Key Types

| Key | Purpose |
|-----|---------|
| Spend Key | Required to send funds |
| View Key | View incoming transactions (read-only) |

```rust
// Get view-only wallet
let view_key = monero::get_view_key(mnemonic)?;
```

## Address Types

| Prefix | Type |
|--------|------|
| `4` | Standard address |
| `8` | Subaddress (privacy) |
| `4` (long) | Integrated (with payment ID) |

## Privacy Features

- **Ring Signatures**: Sender hidden among decoys
- **Stealth Addresses**: One-time receiving addresses
- **RingCT**: Transaction amounts hidden

## Resources

- [Monero Docs](https://www.getmonero.org/resources/developer-guides/)
- [XMRchain Explorer](https://xmrchain.net)
