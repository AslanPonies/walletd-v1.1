# WalletD Security Documentation

## Overview

WalletD is a multi-chain cryptocurrency wallet SDK handling sensitive cryptographic material. This document outlines security considerations, best practices, and known limitations.

## Threat Model

### Assets Protected
1. **Private Keys** - Ed25519/secp256k1 signing keys
2. **Mnemonic Phrases** - 12/24 word recovery phrases
3. **Signatures** - Transaction authorization
4. **Addresses** - Account identifiers (less sensitive)

### Threat Actors
1. **Local Attacker** - Access to device memory
2. **Network Attacker** - Man-in-the-middle on RPC calls
3. **Supply Chain** - Compromised dependencies
4. **User Error** - Misuse of API

### Attack Vectors
| Vector | Mitigation | Status |
|--------|------------|--------|
| Memory dumping | Zeroization on drop | ⚠️ Partial |
| Timing attacks | Constant-time comparison | ✅ Done |
| RPC interception | HTTPS only | ✅ Done |
| Dependency vulns | cargo-audit | ✅ Done |
| Unsafe code | `#![forbid(unsafe_code)]` | ✅ Done |

## Security Measures Implemented

### 1. No Unsafe Code
All crates use `#![forbid(unsafe_code)]` to prevent memory safety issues.

```rust
#![forbid(unsafe_code)]
```

### 2. Dependency Auditing
Run `cargo audit` before each release to check for known vulnerabilities.

```bash
cargo audit
cargo deny check
```

### 3. Input Validation
All user inputs are validated before processing:

```rust
impl TonAddress {
    pub fn from_friendly(addr: &str) -> Result<Self, TonError> {
        // Length validation
        if addr.len() > 48 {
            return Err(TonError::InvalidAddress("Too long".into()));
        }
        
        // Character validation
        if !addr.chars().all(|c| is_valid_base64_char(c)) {
            return Err(TonError::InvalidAddress("Invalid characters".into()));
        }
        
        // Checksum validation
        // ...
    }
}
```

### 4. Error Handling
No panics in production code. All errors are propagated via `Result<T, E>`.

```rust
// Bad
let key = bytes.try_into().unwrap();

// Good
let key = bytes.try_into().map_err(|_| KeyError::InvalidLength)?;
```

### 5. Constant-Time Operations
Signature verification uses constant-time comparison:

```rust
use subtle::ConstantTimeEq;

fn verify_signature(expected: &[u8], actual: &[u8]) -> bool {
    expected.ct_eq(actual).into()
}
```

## Security Measures TODO

### 1. Zeroization
Private key material should be zeroed when dropped:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
struct PrivateKey([u8; 32]);
```

**Status:** Not yet implemented for all wallet types.

### 2. Memory Locking
Consider using `mlock()` to prevent private keys from being swapped to disk:

```rust
// Platform-specific, requires careful implementation
```

**Status:** Not implemented.

### 3. Secure Random
All key generation uses `rand::thread_rng()` which is cryptographically secure on supported platforms.

## Known Limitations

### 1. Single-Signature Only
Current implementation does not support multi-signature wallets.

### 2. No Hardware Wallet Support
No integration with Ledger/Trezor devices.

### 3. Plaintext Keys in Memory
Private keys exist as plaintext in memory during wallet lifetime.

### 4. No Key Derivation Hardening
Some derivation paths use non-hardened indices where hardened would be safer.

### 5. Test Vectors
Not all chains have comprehensive test vectors from official sources.

## Secure Usage Guidelines

### DO
- ✅ Store mnemonics in secure storage (OS keychain, encrypted files)
- ✅ Use testnet for development
- ✅ Validate all addresses before sending
- ✅ Keep dependencies updated
- ✅ Use strong, unique passwords for mnemonic encryption

### DON'T
- ❌ Log private keys or mnemonics
- ❌ Transmit keys over unencrypted channels
- ❌ Use example/test keys in production
- ❌ Ignore security audit findings
- ❌ Deploy without testing on testnet first

## Security Testing

### Fuzzing
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz address parsing
    let _ = TonAddress::from_friendly(&String::from_utf8_lossy(data));
});
```

### Property Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn address_roundtrip(bytes in prop::array::uniform32(any::<u8>())) {
        let addr = TonAddress::new(0, bytes);
        let encoded = addr.to_friendly(TonNetwork::Mainnet, true);
        let decoded = TonAddress::from_friendly(&encoded).unwrap();
        prop_assert_eq!(addr, decoded);
    }
}
```

## Vulnerability Disclosure

If you discover a security vulnerability, please report it responsibly:

1. **Email:** security@walletd.dev (replace with actual)
2. **Do NOT** open a public GitHub issue
3. Allow 90 days for fix before public disclosure

## Audit Status

| Component | Last Audit | Auditor | Status |
|-----------|-----------|---------|--------|
| walletd-core | - | - | Not audited |
| walletd_bitcoin | - | - | Not audited |
| walletd_ethereum | - | - | Not audited |
| walletd_ton | - | - | Not audited |

**Recommendation:** Conduct a professional security audit before production use with significant funds.

## References

- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [Rust Secure Coding Guidelines](https://anssi-fr.github.io/rust-guide/)
- [BIP-39 Mnemonic Security](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
