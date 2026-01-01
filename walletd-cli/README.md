# WalletD CLI - Integrated Edition

**Version 0.2.0** - Full SDK integration for multi-chain wallet operations

## Overview

This CLI is a **true drop-in replacement** for `walletd-icp-cli` that:

1. **Uses the WalletD SDK crates** directly (not duplicating code)
2. **Performs real wallet operations** on actual blockchains
3. **Maintains 100% backward compatibility** with the original menu flow
4. **Extends to 17+ chains** (when SDK crates are available)

## Architecture

```
walletd-cli/
├── Cargo.toml                    # Dependencies on SDK crates
├── src/
│   ├── main.rs                   # CLI entry point & menu handling
│   ├── config.rs                 # Config (compatible with original)
│   ├── types.rs                  # Shared types
│   └── wallet_integration/       # Real wallet implementations
│       ├── mod.rs                # Central WalletManager
│       ├── bitcoin_real.rs       # Bitcoin operations
│       ├── ethereum_real.rs      # Ethereum operations
│       ├── solana_real.rs        # Solana operations
│       ├── hedera_real.rs        # Hedera operations
│       ├── monero_real.rs        # Monero operations
│       └── icp_real.rs           # ICP operations
```

## Integration with WalletD Repo

### Step 1: Copy to Repository

```bash
# Copy this directory to the walletd repo root
cp -r walletd-cli /path/to/walletd/
```

### Step 2: Update Workspace Cargo.toml

Add to `walletd/Cargo.toml`:

```toml
[workspace]
members = [
    # Existing members...
    "coins/bitcoin",
    "coins/ethereum",
    "coins/solana",
    # ... etc
    
    # Add the CLI
    "walletd-cli",
]
```

### Step 3: Update Shell Script (Optional)

Update `walletd-cli` shell script to use the new binary:

```bash
#!/bin/bash
exec "$(dirname "$0")/walletd-cli/target/release/walletd" "$@"
```

### Step 4: Build

```bash
cd walletd
cargo build --release -p walletd-cli
```

## Features

### Real Wallet Operations

- **Create wallets** with secure key generation
- **Check balances** via blockchain APIs
- **Send transactions** with proper signing
- **View on explorers** with direct links

### Supported Chains (Core)

| # | Chain | Network | API |
|---|-------|---------|-----|
| 1 | Bitcoin | Testnet/Mainnet | Blockstream |
| 2 | Ethereum | Sepolia/Mainnet | RPC + Etherscan |
| 3 | Solana | Devnet/Mainnet | JSON-RPC |
| 4 | Hedera | Testnet/Mainnet | Portal API |
| 5 | Monero | Stagenet/Mainnet | Wallet RPC |
| 6 | ICP | Local/IC | Agent |
| 7 | ERC-20 | Multi-chain | Contracts |
| 8 | Base | Sepolia/Mainnet | RPC |
| 9 | Prasaga | Testnet | Coming |

### Extended Chains (When SDK Crates Exist)

10-20: Polygon, Avalanche, Arbitrum, Cardano, Cosmos, Polkadot, Near, Tron, SUI, Aptos, TON

## Configuration

Uses `walletd_config.json` (fully compatible with original):

```json
{
  "bitcoin": {
    "network": "testnet",
    "rpc_url": "http://localhost:18332",
    "electrum_url": "ssl://electrum.blockstream.info:60002"
  },
  "ethereum": {
    "chain_id": 11155111,
    "rpc_url": "https://eth-sepolia.g.alchemy.com/v2/demo"
  },
  ...
}
```

## SDK Integration Points

The CLI integrates with SDK crates via Cargo dependencies:

```toml
walletd_bitcoin = { path = "../coins/bitcoin" }
walletd_ethereum = { path = "../coins/ethereum" }
walletd_solana = { path = "../coins/solana" }
# etc.
```

Each `*_real.rs` module wraps SDK functionality with CLI-friendly interfaces.

## Backward Compatibility

✅ Same binary names (`walletd`, `walletd-icp-cli`)
✅ Same mode selection (Testnet/Mainnet/Demo)
✅ Same menu numbers (1-9 for original chains)
✅ Same config file format
✅ Same initialization flow

## Development

```bash
# Run in development
cargo run --bin walletd

# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT OR Apache-2.0 (same as WalletD)
