# Tron

> **High-throughput DPoS** — TRC-20 tokens and low fees.

## Quick Start

```rust
use walletd::tron::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "Txyz..." (base58)

let balance = get_balance(&address, "https://api.trongrid.io").await?;
// → Balance in sun

let tx = send_transaction(&from, &to, sun_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/195'/0'/0/0` |
| Address Format | Base58 (`T...`) |
| Native Unit | sun |
| Decimals | 6 (1 TRX = 1,000,000 sun) |

## Unit Conversion

```rust
fn sun_to_trx(sun: u64) -> f64 {
    sun as f64 / 1_000_000.0
}

fn trx_to_sun(trx: f64) -> u64 {
    (trx * 1_000_000.0) as u64
}
```

## TRC-20 Tokens

```rust
use walletd::tron::trc20;

// USDT on Tron (most used stablecoin network!)
let usdt = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

let balance = trc20::get_balance(usdt, &wallet, &rpc).await?;
let tx = trc20::transfer(usdt, &to, amount, &key, &rpc).await?;
```

## Common TRC-20 Tokens

| Token | Contract |
|-------|----------|
| USDT | `TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t` |
| USDC | `TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8` |

## Energy & Bandwidth

Tron uses Energy (for smart contracts) and Bandwidth (for transactions):

- Stake TRX to get free resources
- Or pay TRX for each transaction

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://api.trongrid.io` |
| Shasta | `https://api.shasta.trongrid.io` |
| Nile | `https://nile.trongrid.io` |

## Resources

- [Tron Docs](https://developers.tron.network)
- [TronScan](https://tronscan.org)
