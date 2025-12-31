# WalletD Crates.io Publishing Guide

## Prerequisites

1. **Get API Token**
   ```bash
   # Visit: https://crates.io/settings/tokens
   # Create a new token with "publish-update" scope
   cargo login <your-token>
   ```

2. **Verify Ownership**
   - If crates already exist, ensure you own them on crates.io
   - Or coordinate with existing owners

## Publishing Order

Crates must be published in dependency order (wait ~30s between each):

### Tier 1: Core (no internal deps)
```bash
cargo publish -p walletd-traits
cargo publish -p walletd-error  
cargo publish -p walletd-core
```

### Tier 2: Provider
```bash
cargo publish -p walletd-provider
```

### Tier 3: Coin Crates
```bash
cargo publish -p walletd_bitcoin
cargo publish -p walletd_ethereum
cargo publish -p walletd_base
cargo publish -p walletd_arbitrum
cargo publish -p walletd_sui
cargo publish -p walletd_aptos
cargo publish -p walletd_monero
cargo publish -p walletd_hedera
cargo publish -p walletd_icp
cargo publish -p walletd_erc20
```

### Tier 4: Unified SDK
```bash
cargo publish -p walletd
```

## Version Bumping

If a crate already exists on crates.io, bump the version:

```toml
# In Cargo.toml
version = "0.2.0"  # Bump from 0.1.0
```

## Common Issues

### "dependency does not specify a version"
Add version to path dependencies:
```toml
# Before
walletd-traits = { path = "../walletd-traits" }

# After  
walletd-traits = { path = "../walletd-traits", version = "0.1" }
```

### "crate already exists"
Bump the version number in Cargo.toml

### "not logged in"
Run `cargo login <token>` first

## Dry Run First

Always test before publishing:
```bash
cargo publish -p walletd-traits --dry-run
```

## After Publishing

1. **Verify on crates.io**: https://crates.io/crates/walletd
2. **Update README badges** with crates.io links
3. **Create GitHub release** with matching tag
4. **Announce** on social media / Discord

## Links

- Crates.io: https://crates.io/crates/walletd
- Docs.rs: https://docs.rs/walletd
- API tokens: https://crates.io/settings/tokens
