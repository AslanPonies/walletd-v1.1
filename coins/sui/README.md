# walletd_sui

[![Crates.io](https://img.shields.io/crates/v/walletd_sui.svg)](https://crates.io/crates/walletd_sui)
[![Documentation](https://docs.rs/walletd_sui/badge.svg)](https://docs.rs/walletd_sui)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

SUI blockchain support for the WalletD SDK.

## Features

- 🔑 **Ed25519 Keys** - Native SUI cryptographic operations
- 📬 **Address Generation** - Standard SUI address format (0x + 64 hex)
- ✍️ **Transaction Signing** - Sign SUI transactions with intent messages
- 🌲 **HD Derivation** - BIP-44 path `m/44'/784'/account'/0'/index'` via SLIP-10
- 🔄 **Network Support** - Mainnet, Testnet, Devnet, Localnet

## Quick Start

```rust
use walletd_sui::{SuiWallet, SuiNetwork, SuiAmount};

// Create from mnemonic
let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
let wallet = SuiWallet::from_mnemonic(mnemonic, SuiNetwork::Mainnet)?;

println!("Address: {}", wallet.address());
// Address: 0x...

// Create random wallet
let random_wallet = SuiWallet::new(SuiNetwork::Testnet);

// Amount handling
let amount = SuiAmount::from_sui(1.5);
println!("MIST: {}", amount.mist());  // 1500000000
```

## Address Derivation

SUI uses Ed25519 keys derived via SLIP-10 with BIP-44 paths:

```
m/44'/784'/account'/0'/address_index'
```

Where:
- `784` is the SUI coin type
- All path components are hardened

```rust
// Multiple accounts from same mnemonic
let wallet0 = SuiWallet::from_mnemonic_with_path(mnemonic, SuiNetwork::Mainnet, 0, 0)?;
let wallet1 = SuiWallet::from_mnemonic_with_path(mnemonic, SuiNetwork::Mainnet, 0, 1)?;
let wallet2 = SuiWallet::from_mnemonic_with_path(mnemonic, SuiNetwork::Mainnet, 1, 0)?;
```

## Transaction Signing

```rust
let wallet = SuiWallet::from_mnemonic(mnemonic, SuiNetwork::Mainnet)?;

// Sign transaction bytes
let tx_bytes = vec![/* BCS-serialized transaction */];
let signature = wallet.sign_transaction(&tx_bytes)?;

// Get signature for RPC submission
let sig_base64 = signature.to_base64();
```

## Networks

| Network | RPC URL | Faucet |
|---------|---------|--------|
| Mainnet | `fullnode.mainnet.sui.io:443` | ❌ |
| Testnet | `fullnode.testnet.sui.io:443` | ✅ |
| Devnet | `fullnode.devnet.sui.io:443` | ✅ |
| Localnet | `127.0.0.1:9000` | ❌ |

```rust
let network = SuiNetwork::Testnet;
println!("RPC: {}", network.rpc_url());
println!("Faucet: {:?}", network.faucet_url());
```

## Amount Handling

SUI uses MIST as the base unit (1 SUI = 10^9 MIST):

```rust
// From SUI
let a = SuiAmount::from_sui(1.5);
assert_eq!(a.mist(), 1_500_000_000);

// From MIST
let b = SuiAmount::from_mist(1_000_000_000);
assert_eq!(b.sui(), 1.0);

// Arithmetic
let sum = a + b;
let diff = a - b;

// Safe arithmetic
let safe_sum = a.checked_add(b);
let safe_diff = a.checked_sub(b);
```

## Keystore Export

Export wallet in SUI keystore format:

```rust
let wallet = SuiWallet::from_mnemonic(mnemonic, SuiNetwork::Mainnet)?;
let keystore = wallet.to_keystore();
// Base64 encoded: flag || private_key || public_key
```

## Features

| Feature | Description |
|---------|-------------|
| `default` | Core wallet functionality |
| `async-runtime` | Tokio async support |
| `rpc` | HTTP client for RPC calls |

## Security Notes

⚠️ **Handle private keys with care:**

- `private_key()` and `private_key_hex()` expose sensitive data
- Use `to_keystore()` for secure export
- Never log or transmit private keys

## License

MIT OR Apache-2.0
