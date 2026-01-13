# Installation

## Requirements

- **Rust**: 1.70 or later
- **OpenSSL**: Required for some chain integrations
- **pkg-config**: For dependency resolution

### macOS
```bash
brew install openssl pkg-config
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Ubuntu/Debian
```bash
sudo apt-get install libssl-dev pkg-config
```

## Library Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
walletd = "1.4"

# Optional: specific chain support
walletd-bitcoin = "1.4"
walletd-ethereum = "1.4"
walletd-solana = "1.4"
```

## CLI Installation
```bash
# Clone repository
git clone https://github.com/AslanPonies/walletd-v1.1
cd walletd-v1.1/walletd-cli

# Build release binary
cargo build --release

# Optional: Install globally
cp target/release/walletd /usr/local/bin/
```

## Feature Flags
```toml
[dependencies]
walletd = { version = "1.4", features = ["all-chains"] }
```

Available features:
- `bitcoin` - Bitcoin support
- `ethereum` - Ethereum + EVM chains
- `solana` - Solana support
- `all-chains` - All 19 chains (default)
- `hardware` - Ledger/Trezor support
- `multisig` - Multi-signature wallets
- `staking` - Staking functionality
