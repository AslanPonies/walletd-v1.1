# Why WalletD?

> **How WalletD compares to alternatives.**

---

## vs Building From Scratch

| Aspect | From Scratch | WalletD |
|--------|--------------|---------|
| Time to first transaction | 2-4 weeks | 1 hour |
| Chains supported | 1 (per effort) | 18 out of the box |
| Learning curve | High (per chain) | One API |
| Maintenance burden | You own it all | We maintain it |
| Security audits | Your responsibility | Shared effort |

---

## vs JavaScript Libraries

| Feature | ethers.js + bitcoinjs | WalletD |
|---------|----------------------|---------|
| Language | JavaScript | Rust |
| Type safety | Runtime errors | Compile-time |
| Performance | Good | Excellent |
| Multi-chain | Manual integration | Unified API |
| Memory safety | GC-dependent | Guaranteed |

**When to use JS**: Browser wallets, quick prototypes
**When to use WalletD**: Backend services, exchanges, high-security apps

---

## vs Hardware Wallet SDKs

| Feature | Ledger SDK | WalletD |
|---------|------------|---------|
| Software wallets | ❌ | ✅ |
| Hardware support | Ledger only | Ledger + Trezor |
| Hot wallet | ❌ | ✅ |
| Multi-chain | Limited | 18 chains |

**WalletD + Hardware**: Use both! WalletD integrates with Ledger/Trezor.

---

## Feature Comparison

| Feature | WalletD | ethers.js | bitcoinjs | solana-web3 |
|---------|---------|-----------|-----------|-------------|
| Bitcoin | ✅ | ❌ | ✅ | ❌ |
| Ethereum | ✅ | ✅ | ❌ | ❌ |
| Solana | ✅ | ❌ | ❌ | ✅ |
| L2 chains | ✅ | ✅ | ❌ | ❌ |
| Hardware wallets | ✅ | Partial | ❌ | ❌ |
| Multisig | ✅ | Limited | ✅ | ❌ |
| Staking | ✅ | ❌ | ❌ | ✅ |
| Unified API | ✅ | ❌ | ❌ | ❌ |

---

## Migration Guide

### From ethers.js

```javascript
// ethers.js
const wallet = ethers.Wallet.fromMnemonic(mnemonic);
const balance = await provider.getBalance(wallet.address);
```

```rust
// WalletD
let address = ethereum::derive_address(mnemonic, WalletMode::Mainnet)?;
let balance = ethereum::get_balance(&address, &rpc).await?;
```

### From bitcoinjs

```javascript
// bitcoinjs
const root = bip32.fromSeed(seed);
const child = root.derivePath("m/84'/0'/0'/0/0");
const address = bitcoin.payments.p2wpkh({ pubkey: child.publicKey }).address;
```

```rust
// WalletD  
let address = bitcoin::derive_address(mnemonic, WalletMode::Mainnet)?;
```

---

## When NOT to Use WalletD

- **Browser-only apps**: Use ethers.js or viem
- **Single-chain focus**: Native SDK may have more features
- **Extreme customization**: Roll your own for full control

---

## Summary

Use WalletD when you need:
- ✅ Multi-chain support from day one
- ✅ Production-grade reliability
- ✅ Type-safe Rust backend
- ✅ Unified error handling
- ✅ Hardware wallet integration
