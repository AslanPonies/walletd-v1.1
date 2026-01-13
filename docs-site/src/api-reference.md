# API Reference

> **Complete function reference for all chains.**

---

## Universal Functions

Every chain implements these core functions:

### derive_address

```rust
pub fn derive_address(mnemonic: &str, mode: WalletMode) -> Result<String>
```

Derives blockchain address from BIP-39 mnemonic.

### get_balance

```rust
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String>
```

Returns balance in smallest unit (satoshis, wei, lamports, etc.)

### send_transaction

```rust
pub async fn send_transaction(
    from: &str,
    to: &str,
    amount: impl ToString,
    rpc_url: &str,
) -> Result<String>
```

Creates, signs, and broadcasts a transaction.

---

## Chain-Specific Details

| Chain | Path | Unit | Decimals |
|-------|------|------|----------|
| Bitcoin | m/84'/0'/0'/0/0 | satoshi | 8 |
| Ethereum | m/44'/60'/0'/0/0 | wei | 18 |
| Solana | m/44'/501'/0'/0' | lamport | 9 |
| Polygon | m/44'/60'/0'/0/0 | wei | 18 |
| Arbitrum | m/44'/60'/0'/0/0 | wei | 18 |
| Base | m/44'/60'/0'/0/0 | wei | 18 |
| Avalanche | m/44'/60'/0'/0/0 | wei | 18 |
| Cardano | m/1852'/1815'/0'/0/0 | lovelace | 6 |
| Polkadot | m/44'/354'/0'/0'/0' | planck | 10 |
| Cosmos | m/44'/118'/0'/0/0 | uatom | 6 |
| NEAR | m/44'/397'/0' | yoctoNEAR | 24 |
| Tron | m/44'/195'/0'/0/0 | sun | 6 |
| Sui | m/44'/784'/0'/0'/0' | MIST | 9 |
| Aptos | m/44'/637'/0'/0'/0' | octa | 8 |
| TON | m/44'/607'/0' | nanoton | 9 |
| ICP | m/44'/223'/0'/0/0 | e8s | 8 |
| Hedera | m/44'/3030'/0'/0/0 | tinybar | 8 |
| Monero | custom | atomic | 12 |

---

## Error Types

```rust
pub enum WalletError {
    InvalidMnemonic,
    DerivationFailed(String),
    NetworkError(String),
    InvalidAddress,
    InsufficientFunds,
    TransactionFailed(String),
}
```

---

## Full Rustdoc

Generate complete API docs:

```bash
cargo doc --open --no-deps
```
