# WalletD Security Audit Preparation

## Overview

This document outlines security considerations, threat models, and audit preparation for WalletD - a multi-chain cryptocurrency wallet SDK.

## Supported Chains (14 total)

| Chain | Crypto Scheme | Key Derivation | Status |
|-------|--------------|----------------|--------|
| Bitcoin | secp256k1 | BIP32/44/84 | Production |
| Ethereum | secp256k1 | BIP32/44 | Production |
| Solana | Ed25519 | BIP32/44 | Production |
| Monero | Ed25519 | Monero-specific | Production |
| Hedera | Ed25519 | BIP32/44 | Production |
| ICP | Ed25519 | ICP-specific | Production |
| TON | Ed25519 | Custom PBKDF2 | Production |
| SUI | Ed25519 | BIP32/44 | Production |
| Arbitrum | secp256k1 | EVM-compatible | Production |
| Base | secp256k1 | EVM-compatible | Production |
| Aptos | Ed25519 | BIP32/44 | Production |
| Prasaga | secp256k1 | Custom | Beta |

## Threat Model

### Assets to Protect

1. **Private Keys** - Most critical asset
   - Must never be logged or exposed
   - Should be zeroized after use
   - Must use secure random generation

2. **Mnemonics** - Recovery phrases
   - 12 or 24 word BIP-39 phrases
   - Must validate checksum
   - Must support all BIP-39 languages

3. **Transaction Data** - Financial operations
   - Must prevent double-spending
   - Must validate amounts before signing
   - Must prevent address substitution

### Threat Actors

1. **Remote Attackers** - Network-based attacks
   - Man-in-the-middle on RPC connections
   - DNS hijacking of RPC endpoints
   - Malicious RPC server responses

2. **Local Attackers** - Access to device
   - Memory dumping for key extraction
   - Side-channel timing attacks
   - File system access to stored keys

3. **Supply Chain** - Dependency attacks
   - Compromised crate dependencies
   - Typosquatting attacks
   - Malicious updates

## Security Controls

### Key Management

```rust
// Keys are generated using secure random sources
use rand::rngs::OsRng;

// BIP-39 mnemonics use CSPRNG
let mnemonic = Mnemonic::generate(&mut OsRng, 24)?;

// Private keys derived using BIP-32
let master = ExtendedPrivKey::new_master(network, &seed)?;
```

### Memory Safety

- All private key types should implement `Zeroize`
- Sensitive data cleared on drop
- No logging of key material

### Input Validation

| Input | Validation |
|-------|-----------|
| Mnemonic | Word count, checksum, word list |
| Address | Format, checksum, network match |
| Amount | Non-negative, precision, overflow |
| Derivation Path | Format, hardened markers, depth |

## Test Coverage

### Unit Tests

| Crate | Tests | Coverage |
|-------|-------|----------|
| Bitcoin | 62 | ~70% |
| Ethereum | 87 | ~75% |
| Hedera | 27 | ~60% |
| TON | 32 | ~65% |
| HD Key | 15 | ~80% |
| Total | 400+ | ~60% |

### Edge Case Tests

- All zeros private key (rejected)
- All ones private key
- Boundary values (secp256k1 order)
- Maximum amounts
- Dust thresholds
- Invalid checksums

### Fuzz Testing

Fuzz targets implemented:
- `fuzz_mnemonic` - BIP-39 parsing
- `fuzz_hd_derivation` - HD path handling
- `fuzz_eth_amount` - Amount arithmetic

Run fuzzing:
```bash
cargo +nightly fuzz run fuzz_mnemonic -- -max_total_time=3600
```

## Known Limitations

1. **No HSM Support** - Keys stored in software only
2. **No Hardware Wallet** - Ledger/Trezor not integrated
3. **Network Trust** - RPC endpoints must be trusted
4. **No MPC** - Single-key signatures only

## Audit Scope Recommendations

### High Priority

1. **Key Derivation** - `key_manager/hd_key/`
   - BIP-32 implementation
   - Path parsing
   - Hardened derivation

2. **Signing Operations** - `coins/*/src/*_wallet.rs`
   - Transaction construction
   - Signature generation
   - Nonce handling

3. **Amount Handling** - `coins/*/src/*_amount.rs`
   - Overflow prevention
   - Precision handling
   - Conversion accuracy

### Medium Priority

4. **Address Generation** - All chain modules
   - Checksum calculation
   - Format validation
   - Network detection

5. **RPC Communication** - Network modules
   - TLS verification
   - Response validation
   - Error handling

### Lower Priority

6. **Configuration** - Config modules
   - Secure defaults
   - Environment handling
   - Secret management

## Dependency Audit

Critical dependencies to review:

```toml
# Cryptography
bip39 = "2.0"           # Mnemonic handling
secp256k1 = "0.29"      # ECDSA signatures
ed25519-dalek = "2.1"   # Ed25519 signatures
sha2 = "0.10"           # SHA-256
ripemd = "0.1"          # RIPEMD-160

# Blockchain
bitcoin = "0.32"        # Bitcoin primitives
alloy = "1.2"           # Ethereum primitives
solana-sdk = "1.18"     # Solana primitives
```

## Pre-Audit Checklist

- [x] All tests passing
- [x] Fuzz targets created
- [x] Edge cases documented
- [x] No panics in parsing code
- [x] Dependency versions pinned
- [ ] cargo-audit clean (1 unmaintained dep: serde_cbor via ic-agent)
- [ ] No unsafe code (or justified)
- [ ] Logging sanitized

## Known Dependency Issues

1. **serde_cbor** (RUSTSEC-2021-0127) - Unmaintained
   - Used by: ic-agent (ICP integration)
   - Risk: Low - library still functional
   - Mitigation: Monitor for replacement in ic-agent updates

## Contact

For security issues: security@walletd.io
