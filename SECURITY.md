# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.2.x   | :white_check_mark: |
| 1.1.x   | :white_check_mark: |
| 1.0.x   | :x:                |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

1. **DO NOT** create a public GitHub issue for security vulnerabilities
2. Email security concerns to: [security@walletd.dev](mailto:security@walletd.dev)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (if available)

### What to Expect

- **Acknowledgment**: We'll acknowledge receipt within 48 hours
- **Initial Assessment**: Within 7 days, we'll provide an initial assessment
- **Resolution Timeline**: Critical issues targeted for fix within 30 days
- **Credit**: With your permission, we'll credit you in the security advisory

## Known Security Issues

### Tracked Dependencies

The following transitive dependency issues are tracked and monitored:

| Advisory | Crate | Status | Risk | Notes |
|----------|-------|--------|------|-------|
| RUSTSEC-2025-0137 | ruint 1.17.0 | Awaiting upstream fix | LOW | `reciprocal_mg10` unsoundness - not used in WalletD code paths. Transitive from alloy. |

### Unmaintained Dependencies (Non-Security)

These are informational warnings about unmaintained crates, not security vulnerabilities:

| Crate | Source | Status |
|-------|--------|--------|
| backoff | ic-agent, hedera | Awaiting upstream update |
| derivative | alloy (ark-ff) | Awaiting upstream update |
| fxhash | bdk (sled) | Awaiting BDK update |
| instant | ic-agent, bdk | Awaiting upstream update |
| paste | candid, alloy | Awaiting upstream update |
| rustls-pemfile | reqwest, tonic | Awaiting upstream update |
| serde_cbor | ic-agent | Awaiting IC-Agent update |

Run `cargo audit` to see current status. Configuration: `.cargo/audit.toml`

### Security Measures in v1.2.0

This version includes critical security fixes:

- **ethers→alloy migration**: Eliminates ring 0.16.20 vulnerability (RUSTSEC-2025-0009)
- **ed25519-dalek 2.2**: Fixes Double Public Key Signing Oracle Attack (RUSTSEC-2022-0093)
- **curve25519-dalek 4.1**: Fixes timing variability in Scalar operations (RUSTSEC-2024-0344)
- **rustls via BDK 0.30**: Fixes infinite loop vulnerability (RUSTSEC-2024-0336)
- **ic-agent 0.44**: Updated ring/rustls dependencies

### Security Measures in v1.1.0

- **H-01**: Proper cryptographic key zeroization using `ZeroizingSigningKey` wrapper
- **H-02**: Constant-time comparison functions to prevent timing attacks (`ct_eq`, `ct_eq_32`, `ct_eq_64`)
- **M-01**: Secure file permissions (0o600) for configuration files
- **M-03**: Clear warnings on test mnemonics in example code
- **M-04**: Debug logging gated behind feature flags
- **M-05**: Safe integer arithmetic in amount calculations

### Cryptographic Dependencies

We use well-audited cryptographic libraries:

| Library | Version | Purpose |
|---------|---------|---------|
| ed25519-dalek | 2.2+ | Ed25519 signatures |
| curve25519-dalek | 4.1+ | Curve operations |
| secp256k1 | 0.27+ | ECDSA for Bitcoin/Ethereum |
| ring | 0.17+ | TLS/cryptographic operations |
| sha2 | 0.10+ | SHA-256/SHA-512 |
| subtle | 2.5+ | Constant-time operations |
| zeroize | 1.8+ | Secure memory cleanup |
| rand | 0.8+ | Cryptographic randomness (OsRng) |

### Best Practices for Users

1. **Never use test mnemonics** ("abandon abandon...") for real funds
2. **Secure your configuration files** - they may contain sensitive data
3. **Keep dependencies updated** - run `cargo audit` regularly
4. **Use hardware wallets** for production environments when possible
5. **Enable 2FA** on any services integrated with WalletD

### Audit History

| Date | Version | Auditor | Findings |
|------|---------|---------|----------|
| Dec 2025 | 1.2.0 | Internal Security Review | 6 Critical dependency vulnerabilities fixed, ethers→alloy migration |
| Dec 2025 | 1.1.0 | Internal Pen Test | 2 High, 4 Medium (all fixed) |

## Bug Bounty

We currently do not have a formal bug bounty program, but we greatly appreciate responsible disclosure and will acknowledge security researchers who help improve WalletD.
