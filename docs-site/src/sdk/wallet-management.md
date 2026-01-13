# Wallet Management

## Creating Wallets
```rust
let manager = WalletManager::new(WalletMode::Testnet)?;
let wallet = manager.create_wallet(Chain::Bitcoin)?;
```
