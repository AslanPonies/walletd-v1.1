# walletd_ton

[![Crates.io](https://img.shields.io/crates/v/walletd_ton.svg)](https://crates.io/crates/walletd_ton)
[![Documentation](https://docs.rs/walletd_ton/badge.svg)](https://docs.rs/walletd_ton)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

TON (The Open Network) blockchain support for the WalletD SDK.

## Features

- 🔑 **Ed25519 Keys** - Native TON cryptographic operations
- 📬 **Address Formats** - Raw and user-friendly (base64 with flags)
- ✍️ **Transaction Signing** - Sign TON messages
- 🔄 **Wallet v4r2** - Standard wallet contract support
- 🌐 **Network Support** - Mainnet and Testnet

## Quick Start

```rust
use walletd_ton::{TonWallet, TonNetwork, TonAmount};

// Create from mnemonic (24 words)
let mnemonic = "abandon abandon abandon ... art"; // 24 words
let wallet = TonWallet::from_mnemonic(mnemonic, TonNetwork::Mainnet)?;

println!("Address: {}", wallet.address_friendly());
// EQ... (bounceable address)

// Create random wallet
let random_wallet = TonWallet::new(TonNetwork::Testnet);

// Amount handling (9 decimals)
let amount = TonAmount::from_ton(1.5);
println!("nanoTON: {}", amount.nano());  // 1500000000
```

## Address Formats

TON uses two address formats:

### Raw Format
```
workchain:hash
0:1234567890abcdef...
```

### User-Friendly Format (base64)
```rust
use walletd_ton::{TonAddress, TonNetwork};

let addr = TonAddress::new(0, [0x12; 32]);

// Bounceable (for smart contracts)
let bounceable = addr.to_bounceable();
// EQ...

// Non-bounceable (for wallets)
let non_bounceable = addr.to_non_bounceable();
// UQ...

// Parse any format
let parsed = "EQAbc...xyz".parse::<TonAddress>()?;
```

### Address Flags

| Network | Bounceable | Flag |
|---------|------------|------|
| Mainnet | Yes | 0x11 |
| Mainnet | No | 0x51 |
| Testnet | Yes | 0x91 |
| Testnet | No | 0xD1 |

## Mnemonic Derivation

TON uses a custom key derivation (not standard BIP-39):

- **PBKDF2** with HMAC-SHA512
- **Salt**: "TON default seed" (no password) or "TON fast seed version" (with password)
- **Iterations**: 100,000 (no password) or 1 (with password)
- **24 words** from BIP-39 wordlist

```rust
// Without password
let wallet = TonWallet::from_mnemonic(mnemonic, TonNetwork::Mainnet)?;

// With password
let wallet = TonWallet::from_mnemonic_with_password(
    mnemonic, 
    "my_password",
    TonNetwork::Mainnet
)?;
```

## Transaction Signing

```rust
let wallet = TonWallet::from_mnemonic(mnemonic, TonNetwork::Mainnet)?;

// Sign arbitrary data
let signature = wallet.sign(b"Hello, TON!");

// Sign message for external message
let message = vec![/* serialized message */];
let sig = wallet.sign_message(&message);

println!("Signature: {}", sig.signature_hex());
println!("Public key: {}", sig.public_key_hex());
```

## Networks

| Network | API Endpoint |
|---------|-------------|
| Mainnet | toncenter.com/api/v2/jsonRPC |
| Testnet | testnet.toncenter.com/api/v2/jsonRPC |

```rust
let network = TonNetwork::Mainnet;
println!("API: {}", network.api_endpoint());
println!("Explorer: {}", network.explorer_url());
```

## Amount Handling

TON uses nanoTON as the base unit (1 TON = 10^9 nanoTON):

```rust
// From TON
let a = TonAmount::from_ton(1.5);
assert_eq!(a.nano(), 1_500_000_000);

// From nanoTON
let b = TonAmount::from_nano(1_000_000_000);
assert_eq!(b.ton(), 1.0);

// Arithmetic
let sum = a + b;
let diff = a - b;

// Safe arithmetic
let safe_sum = a.checked_add(b);
let safe_diff = a.checked_sub(b);
```

## Why TON?

- **900M+ Telegram users** - Massive potential user base
- **Gaming ecosystem** - Mini apps and games integration
- **Fast & cheap** - High throughput, low fees
- **Sharding** - Infinite scalability design

## Integration with WalletD

```toml
[dependencies]
walletd = { version = "0.3", features = ["ton"] }
```

```rust
use walletd::ton::{TonWallet, TonNetwork};

let wallet = TonWallet::new(TonNetwork::Mainnet);
```

## Security Notes

⚠️ **Handle private keys with care:**

- `private_key()` and `private_key_hex()` expose sensitive data
- Never log or transmit private keys
- Use secure storage for key material

## License

MIT OR Apache-2.0
