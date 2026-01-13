# NEAR

> **Developer-friendly L1** — Human-readable accounts with dynamic sharding.

## Quick Start

```rust
use walletd::near::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → Implicit account (64-char hex)

let balance = get_balance(&address, "https://rpc.mainnet.near.org").await?;
// → Balance in yoctoNEAR

let tx = send_transaction(&from, &to, yocto_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/397'/0'` |
| Native Unit | yoctoNEAR |
| Decimals | 24 (1 NEAR = 10^24 yoctoNEAR) |

## Unit Conversion

```rust
fn yocto_to_near(yocto: u128) -> f64 {
    yocto as f64 / 1e24
}

fn near_to_yocto(near: f64) -> u128 {
    (near * 1e24) as u128
}
```

## Account Types

| Type | Format | Example |
|------|--------|---------|
| Implicit | 64-char hex | `98793cd91a3f870fb126f66285...` |
| Named | Human-readable | `alice.near` |
| Sub-account | Hierarchical | `app.alice.near` |

## Creating Named Accounts

```rust
use walletd::near::account;

// Create named account (costs NEAR)
let tx = account::create(
    "myname.near",
    &public_key,
    initial_balance,
    &rpc
).await?;
```

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://rpc.mainnet.near.org` |
| Testnet | `https://rpc.testnet.near.org` |

## Testnet Faucet

Create account at [wallet.testnet.near.org](https://wallet.testnet.near.org)

## Resources

- [NEAR Docs](https://docs.near.org)
- [NEAR Explorer](https://explorer.near.org)
- [NEAR Wallet](https://wallet.near.org)
