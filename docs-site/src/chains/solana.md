# Solana

> **High-performance L1** — 65,000 TPS with sub-second finality.

## Quick Start

```rust
use walletd::solana::{derive_address, get_balance, send_transaction};

let address = derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = get_balance(&address, "https://api.mainnet-beta.solana.com").await?;
let sig = send_transaction(&from, &to, lamports, &key, &rpc).await?;
```

## API Reference

| Function | Returns |
|----------|---------|
| `derive_address(mnemonic, mode)` | Base58 public key |
| `get_balance(address, rpc)` | Balance in lamports |
| `send_transaction(from, to, amount, rpc)` | Transaction signature |

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/501'/0'/0'` |
| Address Format | Base58 (32 bytes) |
| Native Unit | lamport |
| Decimals | 9 (1 SOL = 10^9 lamports) |
| Block Time | ~400ms |
| Finality | ~12 seconds |

## RPC Endpoints

| Network | URL | Rate Limit |
|---------|-----|------------|
| Mainnet | `https://api.mainnet-beta.solana.com` | Heavy |
| Devnet | `https://api.devnet.solana.com` | Moderate |
| Testnet | `https://api.testnet.solana.com` | Moderate |

**Note**: Use dedicated RPC (Helius, QuickNode) for production.

## Unit Conversion

```rust
fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / 1_000_000_000.0
}

fn sol_to_lamports(sol: f64) -> u64 {
    (sol * 1_000_000_000.0) as u64
}
```

## SPL Tokens

```rust
use walletd::solana::spl;

// Get token balance
let usdc_balance = spl::get_balance(
    &owner,
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
    &rpc
).await?;

// Transfer tokens
let sig = spl::transfer(&from, &to, &mint, amount, &key, &rpc).await?;
```

## Common Token Mints

| Token | Mint Address | Decimals |
|-------|--------------|----------|
| USDC | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` | 6 |
| USDT | `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB` | 6 |

## Faucet

```bash
solana airdrop 2 YOUR_ADDRESS --url https://api.devnet.solana.com
```

## Resources

- [Solana Docs](https://docs.solana.com)
- [Solana Explorer](https://explorer.solana.com)
- [Solana Cookbook](https://solanacookbook.com)
