# Security Guide

> **Best practices for production deployments.**

---

## Mnemonic Security

### DO ✅

- Store on paper in secure location
- Use hardware wallet for large holdings
- Test recovery before depositing
- Use passphrase (25th word) for extra security

### DON'T ❌

- Store digitally (photos, cloud, notes)
- Share with anyone
- Enter on websites
- Commit to version control

---

## Code Security

```rust
// ❌ NEVER hardcode secrets
let mnemonic = "abandon abandon...";

// ✅ Use environment variables
let mnemonic = std::env::var("WALLET_MNEMONIC")?;

// ✅ Or secure key management
let mnemonic = vault::get_secret("wallet/mnemonic")?;
```

---

## Network Security

```rust
// ❌ Never use HTTP
let rpc = "http://node.example.com";

// ✅ Always HTTPS
let rpc = "https://node.example.com";
```

---

## Validation Checklist

Before any transaction:

- [ ] Validate recipient address format
- [ ] Verify amount is not dust
- [ ] Check sufficient balance for amount + fee
- [ ] Confirm with user before broadcasting

---

## Audit Status

| Component | Status |
|-----------|--------|
| Key derivation | ✅ Uses audited bip32/bip39 |
| Signing | ✅ Uses audited k256/ed25519 |
| Memory safety | ✅ Pure Rust |
| External audit | 🔄 In progress |
