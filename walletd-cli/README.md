# WalletD CLI v0.2.0 - Full SDK Integration

Multi-chain wallet CLI supporting **17+ blockchains** with:
- ✅ HD wallet derivation (BIP-39/44/84)
- ✅ Real transaction broadcasting
- ✅ Balance checking via blockchain APIs
- ✅ Unified mnemonic for all chains

## Supported Chains

| # | Chain | Symbol | HD Path | Features |
|---|-------|--------|---------|----------|
| 1 | Bitcoin | BTC | m/84'/0'/0'/0/0 | Send, Balance, UTXO |
| 2 | Ethereum | ETH | m/44'/60'/0'/0/0 | Send, Balance, ERC-20 |
| 3 | Solana | SOL | m/44'/501'/0'/0' | Balance, Airdrop |
| 4 | Hedera | HBAR | - | Balance |
| 5 | Monero | XMR | - | Address gen |
| 6 | ICP | ICP | - | Principal ID |
| 7 | ERC-20 | Various | (uses ETH) | Token support |
| 8 | Base | ETH | (uses ETH) | L2 support |
| 9 | Prasaga | SAGA | - | Coming soon |
| 10 | Polygon | POL | (uses ETH) | Send, Balance |
| 11 | Avalanche | AVAX | (uses ETH) | Send, Balance |
| 12 | Arbitrum | ETH | (uses ETH) | Send, Balance |
| 13 | Cardano | ADA | m/1852'/1815'/0'/0/0 | Address gen |
| 14 | Cosmos | ATOM | m/44'/118'/0'/0/0 | Balance |
| 15 | Polkadot | DOT | m/44'/354'/0'/0'/0' | Address gen |
| 16 | Near | NEAR | m/44'/397'/0' | Balance |
| 17 | Tron | TRX | m/44'/195'/0'/0/0 | Balance |
| 18 | SUI | SUI | m/44'/784'/0'/0'/0' | Balance |
| 19 | Aptos | APT | m/44'/637'/0'/0'/0' | Balance |
| 20 | TON | TON | m/44'/607'/0' | Balance |

## Installation

```bash
# In your walletd repo:
rm -rf walletd-cli
# Copy this entire walletd-cli directory
cargo build -p walletd-cli --release
```

## Usage

```bash
./target/release/walletd
```

## Architecture

```
walletd-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── config.rs                  # Configuration
│   ├── types.rs                   # Shared types
│   └── wallet_integration/
│       ├── mod.rs                 # Central WalletManager
│       ├── hd_derivation.rs       # BIP-39/44/84 derivation
│       ├── bitcoin_wallet.rs      # BTC implementation
│       ├── ethereum_wallet.rs     # ETH implementation
│       ├── evm_wallet.rs          # Polygon/Avalanche/Base/Arbitrum
│       ├── solana_wallet.rs       # SOL implementation
│       ├── hedera_wallet.rs       # HBAR implementation
│       ├── monero_wallet.rs       # XMR implementation
│       ├── icp_wallet.rs          # ICP implementation
│       ├── cardano_wallet.rs      # ADA implementation
│       ├── cosmos_wallet.rs       # ATOM implementation
│       ├── polkadot_wallet.rs     # DOT implementation
│       ├── near_wallet.rs         # NEAR implementation
│       ├── tron_wallet.rs         # TRX implementation
│       ├── sui_wallet.rs          # SUI implementation
│       ├── aptos_wallet.rs        # APT implementation
│       └── ton_wallet.rs          # TON implementation
```

## HD Derivation

All wallets derive from a single 24-word mnemonic:

```rust
// Generate new mnemonic
let mnemonic = manager.generate_mnemonic()?;

// Or import existing
manager.set_mnemonic("your 24 words here")?;

// Initialize all wallets
manager.init_all_from_mnemonic().await?;
```

## License

MIT OR Apache-2.0
