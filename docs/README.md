# WalletD SDK v1.4.0

Enterprise-grade multi-chain cryptocurrency wallet SDK in pure Rust.

## New in v1.4.0

### 🚀 Transaction Broadcasting (`walletd-broadcast`)
Real transaction broadcasting for all 18 supported chains with:
- Multiple provider fallback (Blockstream, Mempool.space, Infura, Alchemy, etc.)
- Automatic retry with exponential backoff
- Fee estimation
- Transaction status tracking

### 🔐 Hardware Wallet Support (`walletd-hardware`)
Enterprise security with Ledger and Trezor integration:
- Device discovery and management
- Secure key derivation (BIP-44/84)
- Transaction signing
- Message signing

### 👥 Multi-Signature Wallets (`walletd-multisig`)
M-of-N multi-signature support:
- Bitcoin P2SH/P2WSH multisig
- Ethereum Gnosis Safe compatible
- Spending policies and limits
- Time-locked transactions

### 📈 Staking Integration (`walletd-staking`)
Unified staking across PoS chains:
- Ethereum 2.0 (via Lido, Rocket Pool)
- Solana (native delegation)
- Polkadot (NPoS nomination)
- Cosmos Hub (delegation)
- Cardano (pool delegation)

### 📱 Mobile SDK (`walletd-ffi`)
C-compatible FFI for iOS/Android:
- Swift bindings for iOS
- Kotlin bindings for Android
- Thread-safe async runtime

## Supported Chains (18)

| Chain | Broadcast | Hardware | Multisig | Staking |
|-------|-----------|----------|----------|---------|
| Bitcoin | ✅ | ✅ | ✅ | - |
| Ethereum | ✅ | ✅ | ✅ | ✅ |
| Solana | ✅ | ✅ | - | ✅ |
| Hedera | ✅ | - | - | - |
| Monero | ✅ | - | - | - |
| ICP | ✅ | - | - | - |
| Base | ✅ | ✅ | ✅ | - |
| Polygon | ✅ | ✅ | ✅ | - |
| Avalanche | ✅ | ✅ | ✅ | - |
| Arbitrum | ✅ | ✅ | ✅ | - |
| Cardano | ✅ | ✅ | - | ✅ |
| Cosmos | ✅ | - | - | ✅ |
| Polkadot | ✅ | ✅ | - | ✅ |
| NEAR | ✅ | - | - | - |
| TRON | ✅ | - | - | - |
| SUI | ✅ | - | - | - |
| Aptos | ✅ | - | - | - |
| TON | ✅ | - | - | - |

## Quick Start

```rust
use walletd_broadcast::{MultiBroadcaster, BroadcastConfig, Chain};
use walletd_hardware::DeviceManager;
use walletd_multisig::{MultisigWallet, MultisigConfig};
use walletd_staking::StakingManager;

// Broadcasting
let broadcaster = MultiBroadcaster::new(BroadcastConfig::mainnet());
let result = broadcaster.broadcast_to(Chain::Bitcoin, &signed_tx).await?;

// Hardware wallet
let devices = DeviceManager::new();
let ledger = devices.connect("ledger:2c97:0001")?;
let pubkey = ledger.get_public_key(&path).await?;

// Multi-signature
let config = MultisigConfig { threshold: 2, total_signers: 3, ... };
let wallet = MultisigWallet::new(config)?;
let address = wallet.address()?;

// Staking
let staking = StakingManager::new();
let validators = staking.for_chain(StakingChain::Ethereum).get_validators(10).await?;
```

## Architecture

```
walletd-sdk-v1.4/
├── walletd-broadcast/    # Transaction broadcasting
├── walletd-hardware/     # Ledger/Trezor support
├── walletd-multisig/     # Multi-signature wallets
├── walletd-staking/      # PoS staking
├── walletd-ffi/          # Mobile FFI bindings
└── docs/                 # Documentation
```

## Building

```bash
# Build all crates
cargo build --release

# Build FFI for iOS
cargo build --release --target aarch64-apple-ios

# Build FFI for Android
cargo build --release --target aarch64-linux-android
```

## Enterprise Features

- **Capitec Integration Ready**: Designed for 24M+ user scale
- **Security Audited**: Comprehensive test coverage
- **Compliance**: KYC/AML integration points
- **High Availability**: Circuit breakers and fallback providers

## License

MIT OR Apache-2.0
