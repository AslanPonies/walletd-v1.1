# Cardano

> **Research-driven L1** — Ouroboros proof-of-stake with formal verification.

## Quick Start

```rust
use walletd::cardano::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "addr1qy..."

let balance = get_balance(&address, &rpc).await?;
// → Balance in lovelace

let tx = send_transaction(&from, &to, lovelace_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/1852'/1815'/0'/0/0` (CIP-1852) |
| Address Prefix | `addr1` (mainnet), `addr_test1` (testnet) |
| Native Unit | lovelace |
| Decimals | 6 (1 ADA = 1,000,000 lovelace) |

## Unit Conversion

```rust
fn lovelace_to_ada(lovelace: u64) -> f64 {
    lovelace as f64 / 1_000_000.0
}

fn ada_to_lovelace(ada: f64) -> u64 {
    (ada * 1_000_000.0) as u64
}
```

## Address Types

| Type | Prefix | Description |
|------|--------|-------------|
| Base | `addr1q` | Standard payment + staking |
| Enterprise | `addr1v` | Payment only (no staking) |
| Pointer | `addr1g` | References stake pool |
| Bootstrap | `Ae2` | Legacy Byron era |

## Staking

Cardano supports **liquid staking** - your ADA remains spendable while staked.

```rust
use walletd::staking::CardanoStaking;

let staking = CardanoStaking::new(&rpc);

// Get stake pools
let pools = staking.get_validators(10).await?;

// Delegate to pool
let tx = staking.delegate(&my_address, &pool_id).await?;

// Check rewards
let rewards = staking.get_rewards(&my_address).await?;
```

| Property | Value |
|----------|-------|
| Min Stake | 2 ADA (deposit) |
| Unbonding | None (liquid) |
| Rewards | Every 5 days (epoch) |
| APY | ~3-5% |

## Resources

- [Cardano Docs](https://docs.cardano.org)
- [CardanoScan](https://cardanoscan.io)
- [Pool.pm](https://pool.pm) - Stake pool explorer
