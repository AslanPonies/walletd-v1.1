# Changelog

All notable changes to WalletD will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Unified `walletd` crate with feature gates for minimal binary size
- Comprehensive benchmark suite using Criterion
- API documentation in `docs/API.md`

## [0.3.0] - 2024-12-31

### Added
- **Unified SDK**: New `walletd` crate that re-exports all chain implementations with feature flags
- **Feature Gates**: Modular dependencies - include only the chains you need
- **Benchmark Suite**: Criterion-based benchmarks for performance testing
- **267+ Unit Tests**: Comprehensive test coverage across all chains

### New Chain Tests
- `walletd_monero`: 98 tests (amount, address, wallet, serialization)
- `walletd-traits`: 37 tests (Amount, TxHash, Network, TransactionBuilder)
- `walletd_bitcoin`: 30 tests (HD derivation, address formats, mnemonics)
- `walletd_erc20`: 28 tests (adapter, USDC, function selectors)
- `walletd_solana`: 23 tests (client, keypair, account)
- `walletd_base`: 18 tests (wallet creation, network config)
- `walletd_ethereum`: 20 tests (amount conversion, wallet)
- `walletd_icp`: 14 tests (principal, wallet, transaction)
- `walletd_hedera`: 15 tests (wallet, keys, network)

### Changed
- Updated workspace Cargo.toml with criterion dependency
- Improved error messages across all crates

### Security
- Added `SECURITY.md` with vulnerability reporting guidelines
- Integrated `cargo-deny` for dependency auditing
- Added security audit CI workflow

## [0.2.0] - 2024-12-01

### Added
- **Base L2 Support**: Full Coinbase Base chain integration
- **ERC-20 Module**: Generic ERC-20 token adapter with USDC preset
- **ICP Integration**: Internet Computer Protocol wallet support
- **Hedera Support**: Hedera Hashgraph (HBAR) wallet
- **Prasaga Avio**: Object-oriented blockchain integration

### Changed
- Migrated Ethereum from `ethers` to `alloy` crate
- Updated Bitcoin to BDK 0.30 with new API

### Fixed
- Bitcoin `script_pubkey()` API compatibility with BDK 0.30
- Ethereum amount conversion precision
- Solana commitment level handling

## [0.1.0] - 2024-06-01

### Added
- Initial release
- **Bitcoin**: HD wallets, SegWit (P2WPKH), BIP32/39/44
- **Ethereum**: EIP-1559 transactions, ERC-20 basic support
- **Solana**: Basic wallet operations, SPL token support
- **Monero**: Privacy coin with subaddress support
- Core traits: `Wallet`, `Transferable`, `Syncable`, `HDWallet`
- HD key derivation (`walletd_hd_key`)
- Mnemonic support (`walletd_mnemonics_core`)

### Security
- Constant-time operations for cryptographic comparisons
- Zeroize-on-drop for sensitive data
- No unsafe code policy

---

## Version History Summary

| Version | Date | Highlights |
|---------|------|------------|
| 0.3.0 | 2024-12-31 | Unified SDK, 267+ tests, benchmarks |
| 0.2.0 | 2024-12-01 | Base L2, ERC-20, ICP, Hedera, Prasaga |
| 0.1.0 | 2024-06-01 | Initial release (BTC, ETH, SOL, XMR) |

[Unreleased]: https://github.com/walletd/walletd/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/walletd/walletd/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/walletd/walletd/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/walletd/walletd/releases/tag/v0.1.0
