# WalletD Security Audit Report

**Date:** January 2026  
**Auditor:** Claude (Anthropic)  
**Scope:** Full codebase security review  

---

## Executive Summary

| Category | Status | Count |
|----------|--------|-------|
| 🔴 Critical Vulnerabilities | NEEDS FIX | 3 |
| 🟡 Medium Issues | NEEDS FIX | 1 |
| 🟢 Good Practices | VERIFIED | 8 |
| ⚠️ Unmaintained Dependencies | MONITOR | 11 |

**Overall Assessment:** The core cryptographic implementation is sound, using industry-standard libraries (BIP-39, secp256k1, BDK). However, transitive dependencies from Solana SDK contain known vulnerabilities that require updates.

---

## 🔴 Critical Vulnerabilities

### 1. curve25519-dalek Timing Side-Channel (RUSTSEC-2024-0344)

**Severity:** HIGH  
**Affected:** `walletd-broadcast` via Solana SDK  
**Risk:** Timing variability in scalar operations could leak private key bits through side-channel attacks.

**Fix:**
```toml
# In Cargo.toml, try to force newer version:
[patch.crates-io]
curve25519-dalek = { version = ">=4.1.3" }
```

**Note:** This is a transitive dependency from Solana. Monitor for Solana SDK updates that include the fix.

---

### 2. ed25519-dalek Double Public Key Attack (RUSTSEC-2022-0093)

**Severity:** HIGH  
**Affected:** `walletd-broadcast` via Solana SDK  
**Risk:** Allows signature forgery under specific conditions.

**Fix:**
```toml
[patch.crates-io]
ed25519-dalek = { version = ">=2.0" }
```

**Note:** Requires Solana SDK >= 2.4.x which may not be available yet. Consider feature-flagging Solana support until upstream fixes.

---

### 3. ring AES Panic (RUSTSEC-2025-0009)

**Severity:** MEDIUM  
**Affected:** `walletd-broadcast` via ethers → jsonwebtoken  
**Risk:** Denial of Service via panic on certain AES operations.

**Fix:**
```toml
[dependencies]
ethers = "2.0.15"  # Check for version with ring >=0.17.12

# Or patch:
[patch.crates-io]
ring = { version = ">=0.17.12" }
```

---

## 🟡 Code Issues

### 4. Sensitive Data Logging

**Locations:**
- `walletd_icp_cli/src/icp_menu.rs:15` - Prints mnemonic
- `walletd_icp_cli/src/bin/hedera_faucet.rs:17` - Prints private key
- `walletd_icp_cli/src/monero_instant_faucet.rs:17` - Prints seed

**Risk:** Secrets may end up in logs, terminal history, or screenshots.

**Fix:**
```rust
// BEFORE (bad):
println!("Mnemonic phrase:");
println!("{}", wallet.mnemonic_phrase());

// AFTER (good):
println!("Mnemonic phrase: [HIDDEN - use export function]");
// Or use a secure display method:
println!("Mnemonic phrase (verify carefully):");
print_secure_secret(&wallet.mnemonic_phrase())?;  // Shows briefly, clears screen

// Best: Never print, write to encrypted file
wallet.export_mnemonic_to_encrypted_file(path, password)?;
```

---

## 🟢 Verified Good Practices

| Practice | Status | Location |
|----------|--------|----------|
| `#![forbid(unsafe_code)]` | ✅ | Most crates |
| OsRng for entropy | ✅ | `coins/bitcoin/src/lib.rs` |
| BIP-39 compliant mnemonic generation | ✅ | Using `bip39` crate |
| BIP-84 correct derivation path | ✅ | m/84'/0'/0'/0/n |
| Audited crypto libraries | ✅ | k256, secp256k1, bitcoin, bdk |
| No private key storage | ✅ | Keys derived on-demand |
| Input validation on addresses | ✅ | Multiple locations |
| HTTPS for RPC endpoints | ✅ | All hardcoded URLs |

---

## ⚠️ Unmaintained Dependencies

These don't have active security issues but should be monitored:

| Crate | Status | Recommendation |
|-------|--------|----------------|
| `atty` | Unmaintained + unsound | Replace with `is-terminal` |
| `backoff` | Unmaintained | Replace with `backon` or `again` |
| `bincode` | Unmaintained | Continue using, monitor alternatives |
| `derivative` | Unmaintained | Replace with `derive_more` |
| `fxhash` | Unmaintained | Replace with `rustc-hash` |
| `instant` | Unmaintained | Replace with `web-time` |
| `paste` | Unmaintained | Continue using, low risk |
| `rustls-pemfile` | Unmaintained | Update reqwest to latest |

---

## Remediation Priority

### Immediate (Before Production)
1. Apply patches for curve25519-dalek and ed25519-dalek
2. Remove secret printing from CLI tools
3. Update ethers to version with ring fix

### Short-term (2-4 weeks)
1. Replace `atty` with `is-terminal`
2. Replace `backoff` with `backon`
3. Add zeroization to sensitive data structures

### Medium-term (1-3 months)
1. Monitor Solana SDK for security updates
2. Add memory protection for mnemonics
3. Implement secret display confirmation flow

---

## Recommended Cargo.toml Changes

```toml
# Add to root Cargo.toml

[patch.crates-io]
# Security patches (may cause compatibility issues - test thoroughly)
# curve25519-dalek = "4.1.3"
# ed25519-dalek = "2.1"
# ring = "0.17.12"

[dependencies]
# Replace unmaintained crates
is-terminal = "0.4"  # Instead of atty
backon = "0.4"       # Instead of backoff

# Add security helpers
zeroize = { version = "1.7", features = ["derive"] }
secrecy = "0.8"
```

---

## Security Test Suite

A comprehensive security test suite has been created:

**Location:** `/tests/security_tests.rs`

**Coverage:**
- BIP-39 test vector compliance (4 tests)
- BIP-32/44/84 derivation paths (4 tests)
- Entropy quality validation (4 tests)
- Address format validation (3 tests)
- Attack vector protection (5 tests)
- Memory safety patterns (3 tests)
- Integration security (3 tests)

**Run with:**
```bash
cargo test --test security_tests
```

---

## Conclusion

WalletD has a solid cryptographic foundation with proper use of audited libraries. The main concerns are:

1. **Transitive dependency vulnerabilities** from Solana SDK
2. **Secret exposure** in CLI example code
3. **Unmaintained dependencies** that need replacement

The core wallet functionality (key derivation, address generation, transaction signing) follows industry best practices. With the recommended fixes applied, WalletD would meet enterprise security standards.

---

## Appendix: Test Vectors Verified

| Standard | Vectors Tested | Status |
|----------|---------------|--------|
| BIP-39 | 4 official TREZOR vectors | ✅ |
| BIP-32 | Derivation path structure | ✅ |
| BIP-44 | Purpose/coin/account | ✅ |
| BIP-84 | Native SegWit paths | ✅ |
| SLIP-44 | 18 coin type IDs | ✅ |
