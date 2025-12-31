# WalletD Code Quality Improvement Plan

## Executive Summary

This document outlines a comprehensive plan to improve WalletD from "working prototype" to "production-grade SDK" suitable for:
- Enterprise clients (Capitec Bank - 24M users)
- Crates.io publication
- Security audits
- Grant applications

**Current State:** 65K lines, 14 blockchains, ~330 tests
**Target State:** Production-ready, auditable, 80%+ coverage

---

## Phase 1: Static Analysis & Linting (Day 1)

### 1.1 Fix All Clippy Warnings
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Current issues:
- `impl` blocks that can be derived
- Borrowed expressions that implement required traits
- No-effect operations

### 1.2 Enforce Strict Linting
Add to workspace `Cargo.toml`:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

### 1.3 Format Consistency
```bash
cargo fmt --all
```

Add `.rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

---

## Phase 2: Error Handling Standardization (Day 1-2)

### 2.1 Unified Error Type

Create `crates/walletd-error/src/lib.rs`:
```rust
#[derive(Error, Debug)]
pub enum WalletdError {
    #[error("Key error: {0}")]
    Key(#[from] KeyError),
    
    #[error("Address error: {0}")]
    Address(#[from] AddressError),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("{chain}: {message}")]
    Chain { chain: &'static str, message: String },
}

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    #[error("Derivation failed: {0}")]
    DerivationFailed(String),
}
```

### 2.2 Result Type Alias
```rust
pub type Result<T> = std::result::Result<T, WalletdError>;
```

### 2.3 Remove All `unwrap()` and `expect()`
```bash
grep -r "\.unwrap()" --include="*.rs" | wc -l
grep -r "\.expect(" --include="*.rs" | wc -l
```

Replace with:
- `?` operator
- `.ok_or(Error)?`
- `.map_err(|e| Error::from(e))?`

---

## Phase 3: Test Coverage (Day 2-4)

### 3.1 Current Coverage Analysis
| Crate | Current | Target |
|-------|---------|--------|
| walletd_bitcoin | 25% | 80% |
| walletd_monero | 34% | 80% |
| walletd_ethereum | ~60% | 80% |
| walletd_ton | ~70% | 80% |
| walletd_sui | ~70% | 80% |
| walletd_aptos | ~70% | 80% |
| walletd_arbitrum | ~70% | 80% |

### 3.2 Test Categories to Add

#### Unit Tests (per crate)
- [ ] Key generation (random, mnemonic, private key import)
- [ ] Address derivation (all formats)
- [ ] Amount conversions (edge cases, overflow)
- [ ] Signature generation and verification
- [ ] Serialization/deserialization

#### Property-Based Tests (proptest)
```rust
proptest! {
    #[test]
    fn amount_roundtrip(amount in 0u64..u64::MAX) {
        let a = TonAmount::from_nano(amount);
        assert_eq!(a.nano(), amount);
    }
    
    #[test]
    fn address_roundtrip(bytes in prop::array::uniform32(0u8..)) {
        let addr = TonAddress::new(0, bytes);
        let friendly = addr.to_friendly(TonNetwork::Mainnet, true);
        let parsed = TonAddress::from_friendly(&friendly).unwrap();
        assert_eq!(addr, parsed);
    }
}
```

#### Integration Tests
- [ ] Real network queries (testnet)
- [ ] Transaction broadcast (testnet)
- [ ] Balance checking
- [ ] Nonce management

#### Fuzzing Tests
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = TonAddress::from_friendly(
        &String::from_utf8_lossy(data)
    );
});
```

### 3.3 Coverage Tooling
```bash
# Install
cargo install cargo-tarpaulin

# Run
cargo tarpaulin --workspace --out Html

# CI integration
cargo tarpaulin --workspace --fail-under 80
```

---

## Phase 4: Security Hardening (Day 3-5)

### 4.1 Dependency Audit
```bash
cargo install cargo-audit
cargo audit

cargo install cargo-deny
cargo deny check
```

Add `deny.toml`:
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]

[bans]
multiple-versions = "warn"
```

### 4.2 Zeroize Sensitive Data
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PrivateKey([u8; 32]);

impl Drop for TonWallet {
    fn drop(&mut self) {
        self.signing_key.zeroize();
    }
}
```

### 4.3 Constant-Time Operations
```rust
use subtle::ConstantTimeEq;

fn verify_signature(expected: &[u8], actual: &[u8]) -> bool {
    expected.ct_eq(actual).into()
}
```

### 4.4 Input Validation
```rust
impl TonAddress {
    pub fn from_friendly(addr: &str) -> Result<Self> {
        // Length check
        if addr.len() > 48 {
            return Err(TonError::InvalidAddress("Too long".into()));
        }
        
        // Character validation
        if !addr.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '-' || c == '_') {
            return Err(TonError::InvalidAddress("Invalid characters".into()));
        }
        
        // ... rest of parsing
    }
}
```

### 4.5 Memory Safety
- No `unsafe` code (already enforced with `#![forbid(unsafe_code)]`)
- Bounds checking on all array access
- No raw pointer manipulation

---

## Phase 5: Documentation (Day 4-6)

### 5.1 API Documentation
Every public item needs:
```rust
/// Creates a new TON wallet from a mnemonic phrase.
///
/// # Arguments
///
/// * `mnemonic` - A 24-word BIP-39 mnemonic phrase
/// * `network` - Target network (Mainnet or Testnet)
///
/// # Returns
///
/// Returns `Ok(TonWallet)` on success, or `Err(TonError)` if:
/// - Mnemonic is not 24 words
/// - Words are not in BIP-39 wordlist
/// - Key derivation fails
///
/// # Example
///
/// ```rust
/// use walletd_ton::{TonWallet, TonNetwork};
///
/// let wallet = TonWallet::from_mnemonic(
///     "abandon abandon ... art",
///     TonNetwork::Mainnet
/// )?;
/// ```
///
/// # Security
///
/// The mnemonic should be stored securely and never logged.
pub fn from_mnemonic(mnemonic: &str, network: TonNetwork) -> Result<Self>
```

### 5.2 Architecture Documentation
Create `docs/ARCHITECTURE.md`:
- Crate dependency graph
- Data flow diagrams
- Key derivation paths per chain
- Transaction signing flow

### 5.3 Security Documentation
Create `docs/SECURITY.md`:
- Threat model
- Key management best practices
- Known limitations
- Responsible disclosure policy

### 5.4 Integration Guides
- `docs/guides/BITCOIN.md`
- `docs/guides/ETHEREUM.md`
- `docs/guides/TON.md`
- etc.

---

## Phase 6: CI/CD Pipeline (Day 5-7)

### 6.1 GitHub Actions Workflow

`.github/workflows/ci.yml`:
```yaml
name: CI

on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-Dwarnings"

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace --all-features

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --all-features
      
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --workspace --fail-under 80

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --workspace --all-targets -- -D warnings

  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo doc --workspace --no-deps
        env:
          RUSTDOCFLAGS: "-Dwarnings"
```

### 6.2 Pre-commit Hooks
`.pre-commit-config.yaml`:
```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all --
        language: system
        types: [rust]
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --workspace -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false
```

---

## Phase 7: Code Consistency (Day 6-8)

### 7.1 Standardize Wallet Interface

Every wallet crate should implement:
```rust
pub trait Wallet {
    type Address;
    type Amount;
    type Signature;
    type Error;
    
    // Creation
    fn new(network: Network) -> Self;
    fn from_mnemonic(mnemonic: &str, network: Network) -> Result<Self, Self::Error>;
    fn from_private_key(key: &[u8], network: Network) -> Result<Self, Self::Error>;
    
    // Properties
    fn address(&self) -> &Self::Address;
    fn public_key(&self) -> &[u8];
    fn network(&self) -> Network;
    
    // Operations
    fn sign(&self, message: &[u8]) -> Self::Signature;
}
```

### 7.2 Standardize Amount Type

Every chain amount should implement:
```rust
pub trait Amount: 
    Copy + Clone + Default + 
    Add + Sub + 
    PartialOrd + Ord + 
    Display + Debug +
    Serialize + Deserialize
{
    fn zero() -> Self;
    fn from_base_units(units: u64) -> Self;
    fn to_base_units(&self) -> u64;
    fn decimals() -> u8;
    fn symbol() -> &'static str;
    fn checked_add(&self, other: Self) -> Option<Self>;
    fn checked_sub(&self, other: Self) -> Option<Self>;
}
```

### 7.3 Standardize Address Type

Every chain address should implement:
```rust
pub trait Address: 
    Clone + PartialEq + Eq + Hash +
    Display + FromStr +
    Serialize + Deserialize
{
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
    fn to_string(&self) -> String;
}
```

---

## Phase 8: Performance Optimization (Day 7-9)

### 8.1 Benchmarks
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_address_derivation(c: &mut Criterion) {
    c.bench_function("ton_address_from_pubkey", |b| {
        let pubkey = [0u8; 32];
        b.iter(|| TonAddress::from_ed25519_pubkey(&pubkey))
    });
}

fn bench_signing(c: &mut Criterion) {
    let wallet = TonWallet::new(TonNetwork::Mainnet);
    let message = [0u8; 1000];
    
    c.bench_function("ton_sign_1kb", |b| {
        b.iter(|| wallet.sign(&message))
    });
}

criterion_group!(benches, bench_address_derivation, bench_signing);
criterion_main!(benches);
```

### 8.2 Avoid Allocations
```rust
// Before (allocates)
fn to_hex(&self) -> String {
    format!("0x{}", hex::encode(&self.0))
}

// After (reuses buffer)
fn write_hex(&self, buf: &mut String) {
    buf.push_str("0x");
    for byte in &self.0 {
        write!(buf, "{:02x}", byte).unwrap();
    }
}
```

### 8.3 Lazy Initialization
```rust
use once_cell::sync::Lazy;

static CRC: Lazy<Crc<u16>> = Lazy::new(|| Crc::<u16>::new(&CRC_16_XMODEM));
```

---

## Phase 9: API Ergonomics (Day 8-10)

### 9.1 Builder Pattern
```rust
let wallet = TonWallet::builder()
    .network(TonNetwork::Mainnet)
    .mnemonic("abandon ...")
    .password("optional")
    .wallet_id(698983191)
    .build()?;
```

### 9.2 Fluent API
```rust
let tx = wallet
    .transfer()
    .to("EQ...")
    .amount(TonAmount::from_ton(1.5))
    .comment("Payment")
    .build()?;
```

### 9.3 Sensible Defaults
```rust
impl Default for TonNetwork {
    fn default() -> Self {
        TonNetwork::Mainnet
    }
}
```

### 9.4 Into/From Implementations
```rust
impl From<u64> for TonAmount {
    fn from(nano: u64) -> Self {
        TonAmount::from_nano(nano)
    }
}

impl From<TonAmount> for u64 {
    fn from(amount: TonAmount) -> Self {
        amount.nano()
    }
}
```

---

## Implementation Priority

### Week 1 (Critical)
1. ✅ Phase 1: Static Analysis (2 hours)
2. ✅ Phase 2: Error Handling (4 hours)
3. ✅ Phase 4.1-4.2: Security Basics (2 hours)
4. ✅ Phase 6: CI/CD Pipeline (3 hours)

### Week 2 (Important)
5. Phase 3: Test Coverage (8 hours)
6. Phase 5: Documentation (6 hours)
7. Phase 7: Code Consistency (4 hours)

### Week 3 (Nice-to-have)
8. Phase 8: Performance (4 hours)
9. Phase 9: API Ergonomics (4 hours)
10. Phase 4.3-4.5: Advanced Security (4 hours)

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Test Coverage | ~50% | 80%+ |
| Clippy Warnings | ~20 | 0 |
| Doc Coverage | ~30% | 100% |
| Unsafe Code | 0 | 0 |
| Security Vulns | Unknown | 0 |
| Build Time | ~3min | <2min |

---

## Quick Wins (Do Now)

```bash
# 1. Fix clippy warnings
cargo clippy --fix --workspace --allow-dirty

# 2. Format all code
cargo fmt --all

# 3. Audit dependencies
cargo audit

# 4. Generate docs
cargo doc --workspace --no-deps --open
```
