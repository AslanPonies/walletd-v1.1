# Polkadot

> **Multi-chain protocol** — Parachains with shared security via relay chain.

## Quick Start

```rust
use walletd::polkadot::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "1abc..." (SS58 format)

let balance = get_balance(&address, "wss://rpc.polkadot.io").await?;
// → Balance in planck

let tx = send_transaction(&from, &to, planck_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/354'/0'/0'/0'` |
| Address Format | SS58 encoding |
| Native Unit | planck |
| Decimals | 10 (1 DOT = 10^10 planck) |

## Unit Conversion

```rust
fn planck_to_dot(planck: u128) -> f64 {
    planck as f64 / 10_000_000_000.0
}

fn dot_to_planck(dot: f64) -> u128 {
    (dot * 10_000_000_000.0) as u128
}
```

## SS58 Address Prefixes

| Network | Prefix | Example Start |
|---------|--------|---------------|
| Polkadot | 0 | `1...` |
| Kusama | 2 | `C...`, `D...` |
| Generic | 42 | `5...` |

## Staking (NPoS)

Polkadot uses Nominated Proof of Stake.

```rust
use walletd::staking::PolkadotStaking;

let staking = PolkadotStaking::new(&rpc);

// Nominate validators
let tx = staking.nominate(&validators).await?;

// Check pending rewards
let rewards = staking.get_pending_rewards(&address).await?;
```

| Property | Value |
|----------|-------|
| Min Stake | ~250 DOT (dynamic) |
| Unbonding | 28 days |
| Rewards | Every era (~24h) |
| APY | ~12-15% |

## Resources

- [Polkadot Wiki](https://wiki.polkadot.network)
- [Subscan](https://polkadot.subscan.io)
- [Polkadot.js Apps](https://polkadot.js.org/apps)
