# ICP (Internet Computer)

> **DFINITY's Web3 platform** — Web-speed smart contracts with canisters.

## Quick Start

```rust
use walletd::icp::{derive_address, get_balance, send_transaction};

let principal = derive_address(mnemonic, WalletMode::Mainnet)?;
// → Principal ID

let balance = get_balance(&principal, "https://ic0.app").await?;
// → Balance in e8s

let tx = send_transaction(&from, &to, e8s_amount, &rpc).await?;
```

## Key Details

| Property | Value |
|----------|-------|
| Derivation Path | `m/44'/223'/0'/0/0` |
| Native Unit | e8s |
| Decimals | 8 (1 ICP = 10^8 e8s) |

## Unit Conversion

```rust
fn e8s_to_icp(e8s: u64) -> f64 {
    e8s as f64 / 100_000_000.0
}

fn icp_to_e8s(icp: f64) -> u64 {
    (icp * 100_000_000.0) as u64
}
```

## Account Types

| Type | Description |
|------|-------------|
| Principal ID | Cryptographic identity |
| Account ID | Ledger account (derived from Principal) |

```rust
// Get Account ID from Principal
let account_id = principal_to_account_id(&principal, None)?;
```

## RPC Endpoints

| Network | URL |
|---------|-----|
| Mainnet | `https://ic0.app` |
| Mainnet | `https://icp-api.io` |

## Resources

- [IC Docs](https://internetcomputer.org/docs)
- [IC Dashboard](https://dashboard.internetcomputer.org)
- [NNS Dapp](https://nns.ic0.app)
