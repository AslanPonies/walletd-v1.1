# Cosmos

> **Internet of Blockchains** — IBC-connected sovereign chains with Tendermint.

## Quick Start

```rust
use walletd::cosmos::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "cosmos1abc..."

let balance = get_balance(&address, "https://cosmos-rpc.polkachu.com").await?;
// → Balance in uatom

let tx = send_transaction(&from, &to, uatom_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/118'/0'/0/0` |
| Address Format | Bech32 (`cosmos1...`) |
| Native Unit | uatom |
| Decimals | 6 (1 ATOM = 1,000,000 uatom) |

## Unit Conversion

```rust
fn uatom_to_atom(uatom: u64) -> f64 {
    uatom as f64 / 1_000_000.0
}

fn atom_to_uatom(atom: f64) -> u64 {
    (atom * 1_000_000.0) as u64
}
```

## Chain Prefixes

Different Cosmos chains use different prefixes:

| Chain | Prefix | Coin Denom |
|-------|--------|------------|
| Cosmos Hub | `cosmos1` | uatom |
| Osmosis | `osmo1` | uosmo |
| Juno | `juno1` | ujuno |
| Secret | `secret1` | uscrt |

## IBC Transfers

```rust
use walletd::cosmos::ibc;

// Transfer ATOM to Osmosis
let tx = ibc::transfer(
    &from_cosmos_address,
    &to_osmosis_address,
    amount,
    "transfer",      // port
    "channel-0",     // channel
    &rpc
).await?;
```

## Staking

```rust
use walletd::staking::CosmosStaking;

let staking = CosmosStaking::new(&rpc);

// Delegate to validator
let tx = staking.delegate(&validator_address, amount).await?;

// Claim rewards
let tx = staking.claim_rewards(&validator_address).await?;
```

| Property | Value |
|----------|-------|
| Min Stake | None |
| Unbonding | 21 days |
| APY | ~15-20% |

## RPC Endpoints

| Provider | URL |
|----------|-----|
| Polkachu | `https://cosmos-rpc.polkachu.com` |
| Notional | `https://rpc.cosmos.directory/cosmoshub` |

## Resources

- [Cosmos Docs](https://docs.cosmos.network)
- [Mintscan](https://www.mintscan.io/cosmos)
- [Keplr Wallet](https://www.keplr.app)
