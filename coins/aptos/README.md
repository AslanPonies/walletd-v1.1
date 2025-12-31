# walletd_aptos

[![Crates.io](https://img.shields.io/crates/v/walletd_aptos.svg)](https://crates.io/crates/walletd_aptos)
[![Documentation](https://docs.rs/walletd_aptos/badge.svg)](https://docs.rs/walletd_aptos)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

Aptos blockchain support for the WalletD SDK.

## Features

- 🔑 **Ed25519 Keys** - Native Aptos cryptographic operations
- 📬 **Address Generation** - Standard Aptos address format (0x + 64 hex)
- ✍️ **Transaction Signing** - Sign Aptos transactions
- 🌲 **HD Derivation** - BIP-44 path `m/44'/637'/account'/0'/index'` via SLIP-10
- 🔄 **Network Support** - Mainnet, Testnet, Devnet, Localnet

## Quick Start

```rust
use walletd_aptos::{AptosWallet, AptosNetwork, AptosAmount};

// Create from mnemonic
let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
let wallet = AptosWallet::from_mnemonic(mnemonic, AptosNetwork::Mainnet)?;

println!("Address: {}", wallet.address());
// Address: 0x...

// Create random wallet
let random_wallet = AptosWallet::new(AptosNetwork::Testnet);

// Amount handling (8 decimals)
let amount = AptosAmount::from_apt(1.5);
println!("Octas: {}", amount.octas());  // 150000000
```

## Address Derivation

Aptos uses Ed25519 keys derived via SLIP-10 with BIP-44 paths:

```
m/44'/637'/account'/0'/address_index'
```

Where:
- `637` is the Aptos coin type
- All path components are hardened

```rust
// Multiple accounts from same mnemonic
let wallet0 = AptosWallet::from_mnemonic_with_path(mnemonic, AptosNetwork::Mainnet, 0, 0)?;
let wallet1 = AptosWallet::from_mnemonic_with_path(mnemonic, AptosNetwork::Mainnet, 0, 1)?;
let wallet2 = AptosWallet::from_mnemonic_with_path(mnemonic, AptosNetwork::Mainnet, 1, 0)?;
```

## Transaction Signing

```rust
let wallet = AptosWallet::from_mnemonic(mnemonic, AptosNetwork::Mainnet)?;

// Sign a transaction signing message
let signing_message = vec![/* Transaction signing message */];
let signature = wallet.sign_transaction(&signing_message)?;

// Create authenticator for submission
let authenticator = wallet.create_authenticator(&signature);

// Get signature hex for API calls
println!("Signature: {}", signature.signature_hex());
```

## Networks

| Network | Chain ID | REST API | Faucet |
|---------|----------|----------|--------|
| Mainnet | 1 | fullnode.mainnet.aptoslabs.com | ❌ |
| Testnet | 2 | fullnode.testnet.aptoslabs.com | ✅ |
| Devnet | 3 | fullnode.devnet.aptoslabs.com | ✅ |
| Localnet | 4 | 127.0.0.1:8080 | ❌ |

```rust
let network = AptosNetwork::Testnet;
println!("REST: {}", network.rest_url());
println!("Faucet: {:?}", network.faucet_url());
println!("Explorer: {}", network.explorer_url());
println!("Indexer: {}", network.indexer_url());
```

## Amount Handling

Aptos uses Octas as the base unit (1 APT = 10^8 Octas):

```rust
// From APT
let a = AptosAmount::from_apt(1.5);
assert_eq!(a.octas(), 150_000_000);

// From Octas
let b = AptosAmount::from_octas(100_000_000);
assert_eq!(b.apt(), 1.0);

// Arithmetic
let sum = a + b;
let diff = a - b;

// Safe arithmetic
let safe_sum = a.checked_add(b);
let safe_diff = a.checked_sub(b);
```

## Auth Key

Get the authentication key for account operations:

```rust
let wallet = AptosWallet::from_mnemonic(mnemonic, AptosNetwork::Mainnet)?;

// Auth key = SHA3-256(pubkey || 0x00)
let auth_key = wallet.auth_key();
let auth_key_hex = wallet.auth_key_hex();

// For Ed25519 single-key accounts, auth_key == address
assert_eq!(auth_key, *wallet.address().as_bytes());
```

## Address Formats

Aptos supports both full and short address formats:

```rust
let addr = AptosAddress::from_hex("0x1")?;

// Full format (always 66 chars: 0x + 64 hex)
println!("{}", addr.to_hex());
// 0x0000000000000000000000000000000000000000000000000000000000000001

// Short format (leading zeros omitted)
println!("{}", addr.to_short_hex());
// 0x1
```

## Why Aptos?

- **Move VM** - Safe, resource-oriented smart contracts
- **90% dev growth YoY** - Fastest growing developer ecosystem
- **Gaming/AI focus** - Strong ecosystem for games
- **~1s finality** - Fast transaction confirmation
- **Parallel execution** - High throughput via Block-STM

## Integration with WalletD

```toml
[dependencies]
walletd = { version = "0.3", features = ["aptos"] }

# Or with other Move chains
walletd = { version = "0.3", features = ["move-chains"] }
```

```rust
use walletd::aptos::{AptosWallet, AptosNetwork};

let wallet = AptosWallet::new(AptosNetwork::Mainnet);
```

## Security Notes

⚠️ **Handle private keys with care:**

- `private_key()` and `private_key_hex()` expose sensitive data
- Never log or transmit private keys
- Use secure storage for key material

## License

MIT OR Apache-2.0
