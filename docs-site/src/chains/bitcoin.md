# Bitcoin Deep Dive

> **The complete guide to Bitcoin in WalletD** — From basics to production-ready code.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Address Types Explained](#address-types-explained)
3. [Complete API Reference](#complete-api-reference)
4. [Transaction Building](#transaction-building)
5. [Fee Estimation](#fee-estimation)
6. [UTXO Management](#utxo-management)
7. [Replace-By-Fee (RBF)](#replace-by-fee-rbf)
8. [Multi-Signature Wallets](#multi-signature-wallets)
9. [Watch-Only Wallets](#watch-only-wallets)
10. [Testnet Development](#testnet-development)
11. [Production Checklist](#production-checklist)
12. [Common Patterns](#common-patterns)
13. [Troubleshooting](#troubleshooting)

---

## Quick Start

```rust
use walletd::bitcoin::{derive_address, get_balance, send_transaction, get_transactions};
use walletd::types::WalletMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = "your 24 word mnemonic phrase here";
    
    // 1. Derive a SegWit address
    let address = derive_address(mnemonic, WalletMode::Mainnet)?;
    println!("Address: {}", address);  // bc1q...
    
    // 2. Check balance
    let balance = get_balance(&address, "https://blockstream.info/api").await?;
    println!("Balance: {} sats", balance);
    
    // 3. Get transaction history
    let txs = get_transactions(&address, "https://blockstream.info/api").await?;
    for tx in txs {
        println!("{}: {} sats", tx.txid, tx.amount);
    }
    
    Ok(())
}
```

---

## Address Types Explained

Bitcoin has evolved through multiple address formats. **WalletD defaults to Native SegWit (bc1q) for optimal fees.**

### Comparison Table

| Type | Prefix | Example | Avg Fee | Use Case |
|------|--------|---------|---------|----------|
| **Legacy (P2PKH)** | `1` | `1BvBMSEYstWetq...` | High | Compatibility |
| **Script (P2SH)** | `3` | `3J98t1WpEZ73CN...` | Medium | Multisig, wrapped SegWit |
| **Native SegWit (P2WPKH)** | `bc1q` | `bc1qar0srrr7xfk...` | **Low** ✓ | **Recommended** |
| **Taproot (P2TR)** | `bc1p` | `bc1pmzfrwwndsqm...` | Lowest | Privacy, complex scripts |

### Why SegWit?

```
Legacy transaction:    ~226 bytes → ~22,600 sats fee @ 100 sat/vB
SegWit transaction:    ~141 vbytes → ~14,100 sats fee @ 100 sat/vB
                                     ─────────────────
                                     37% fee savings
```

### Derivation Paths

| Standard | Path | Address Type | When to Use |
|----------|------|--------------|-------------|
| BIP-44 | `m/44'/0'/0'/0/0` | Legacy (1...) | Exchange compatibility |
| BIP-49 | `m/49'/0'/0'/0/0` | Wrapped SegWit (3...) | Legacy system support |
| **BIP-84** | `m/84'/0'/0'/0/0` | **Native SegWit (bc1q...)** | **Default - use this** |
| BIP-86 | `m/86'/0'/0'/0/0` | Taproot (bc1p...) | Advanced privacy |

### Testnet Prefixes

| Type | Mainnet | Testnet |
|------|---------|---------|
| Legacy | `1` | `m` or `n` |
| Script | `3` | `2` |
| SegWit | `bc1q` | `tb1q` |
| Taproot | `bc1p` | `tb1p` |

---

## Complete API Reference

### derive_address

Derives a Bitcoin address from a BIP-39 mnemonic.

```rust
pub fn derive_address(mnemonic: &str, mode: WalletMode) -> Result<String>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `mnemonic` | `&str` | BIP-39 mnemonic (12 or 24 words) |
| `mode` | `WalletMode` | `Mainnet`, `Testnet`, or `Demo` |

**Returns:** Native SegWit address (`bc1q...` or `tb1q...`)

**Example:**

```rust
// Mainnet
let addr = derive_address(mnemonic, WalletMode::Mainnet)?;
assert!(addr.starts_with("bc1q"));

// Testnet  
let addr = derive_address(mnemonic, WalletMode::Testnet)?;
assert!(addr.starts_with("tb1q"));

// Derive multiple addresses (different accounts)
for i in 0..5 {
    let addr = derive_address_at_index(mnemonic, WalletMode::Mainnet, i)?;
    println!("Address {}: {}", i, addr);
}
```

**Errors:**

| Error | Cause | Fix |
|-------|-------|-----|
| `InvalidMnemonic` | Wrong word count or invalid words | Check for 12/24 words, verify spelling |
| `DerivationFailed` | Cryptographic error | Ensure mnemonic is valid BIP-39 |

---

### get_balance

Fetches the confirmed balance for an address.

```rust
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `address` | `&str` | Any valid Bitcoin address |
| `rpc_url` | `&str` | Esplora-compatible API endpoint |

**Returns:** Balance in **satoshis** as a string

**RPC Endpoints:**

| Network | Provider | URL | Rate Limit |
|---------|----------|-----|------------|
| Mainnet | Blockstream | `https://blockstream.info/api` | None |
| Mainnet | Mempool.space | `https://mempool.space/api` | 10/sec |
| Testnet | Blockstream | `https://blockstream.info/testnet/api` | None |
| Testnet | Mempool.space | `https://mempool.space/testnet/api` | 10/sec |

**Example:**

```rust
let balance_sats = get_balance(
    "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq",
    "https://blockstream.info/api"
).await?;

// Convert to BTC
let sats: u64 = balance_sats.parse()?;
let btc = sats as f64 / 100_000_000.0;
println!("{:.8} BTC", btc);
```

**Errors:**

| Error | Cause | Fix |
|-------|-------|-----|
| `NetworkError` | RPC unreachable | Check internet, try another endpoint |
| `InvalidAddress` | Malformed address | Validate address format |
| `RpcError(404)` | Address never used | This is valid - balance is 0 |

---

### send_transaction

Creates, signs, and broadcasts a Bitcoin transaction.

```rust
pub async fn send_transaction(
    from_address: &str,
    to_address: &str,
    amount_sats: u64,
    rpc_url: &str,
) -> Result<String>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `from_address` | `&str` | Sender's address (must have UTXOs) |
| `to_address` | `&str` | Recipient's address (any format) |
| `amount_sats` | `u64` | Amount in satoshis |
| `rpc_url` | `&str` | API endpoint |

**Returns:** Transaction ID (txid)

**Example:**

```rust
// Send 0.001 BTC (100,000 sats)
let txid = send_transaction(
    "bc1qsender...",
    "bc1qrecipient...",
    100_000,
    "https://blockstream.info/api"
).await?;

println!("Sent! Track at: https://mempool.space/tx/{}", txid);
```

**Fee Handling:**

- Fees are **automatically estimated** based on current mempool
- Default target: 6-block confirmation (~1 hour)
- For custom fees, use `send_transaction_with_fee()`

---

### get_transactions

Retrieves transaction history for an address.

```rust
pub async fn get_transactions(
    address: &str,
    rpc_url: &str,
) -> Result<Vec<TransactionInfo>>
```

**Returns:**

```rust
pub struct TransactionInfo {
    pub txid: String,           // Transaction ID
    pub amount: i64,            // Satoshis (+received, -sent)
    pub fee: u64,               // Fee paid (if sender)
    pub confirmations: u32,     // 0 = unconfirmed
    pub timestamp: u64,         // Unix timestamp
    pub block_height: Option<u64>,
}
```

**Example:**

```rust
let txs = get_transactions(&address, &rpc).await?;

for tx in txs {
    let direction = if tx.amount > 0 { "↓ IN" } else { "↑ OUT" };
    let btc = (tx.amount.abs() as f64) / 100_000_000.0;
    println!("{} {:.8} BTC - {}", direction, btc, tx.txid);
}
```

---

## Transaction Building

### Simple Send

```rust
// Send entire balance minus fee
let balance: u64 = get_balance(&from, &rpc).await?.parse()?;
let fee = estimate_fee(&rpc, 1).await?;  // 1 input
let amount = balance - fee;

let txid = send_transaction(&from, &to, amount, &rpc).await?;
```

### Multi-Output Transaction

```rust
use walletd::bitcoin::build_transaction;

let outputs = vec![
    ("bc1qalice...", 50_000),   // 50k sats to Alice
    ("bc1qbob...", 30_000),     // 30k sats to Bob
    ("bc1qcarol...", 20_000),   // 20k sats to Carol
];

let txid = build_transaction()
    .from(&my_address)
    .outputs(outputs)
    .fee_rate(10)  // sat/vB
    .broadcast(&rpc)
    .await?;
```

### OP_RETURN Data

```rust
// Embed data in transaction (up to 80 bytes)
let txid = build_transaction()
    .from(&my_address)
    .to(&recipient, 10_000)
    .op_return(b"WalletD was here!")
    .broadcast(&rpc)
    .await?;
```

---

## Fee Estimation

### Current Fee Rates

```rust
use walletd::bitcoin::get_fee_estimates;

let fees = get_fee_estimates("https://mempool.space/api").await?;

println!("Next block:  {} sat/vB", fees.fastest);    // ~10 min
println!("3 blocks:    {} sat/vB", fees.half_hour);  // ~30 min
println!("6 blocks:    {} sat/vB", fees.hour);       // ~1 hour
println!("Low priority:{} sat/vB", fees.economy);    // ~1 day
```

### Calculate Transaction Fee

```rust
// Estimate fee for a transaction
fn estimate_tx_fee(num_inputs: usize, num_outputs: usize, fee_rate: u64) -> u64 {
    // SegWit transaction size estimation
    let base_size = 10;  // Version + locktime
    let input_size = 68; // Per input (SegWit)
    let output_size = 31; // Per P2WPKH output
    
    let vsize = base_size + (num_inputs * input_size) + (num_outputs * output_size);
    (vsize as u64) * fee_rate
}

// Example: 2 inputs, 2 outputs @ 20 sat/vB
let fee = estimate_tx_fee(2, 2, 20);
println!("Estimated fee: {} sats", fee);  // ~3,960 sats
```

### Fee Priority Guide

| Priority | Target | Typical Rate | When to Use |
|----------|--------|--------------|-------------|
| Urgent | Next block | 50-200 sat/vB | Time-critical |
| Normal | 3 blocks | 20-50 sat/vB | Default |
| Low | 6+ blocks | 10-20 sat/vB | Non-urgent |
| Economy | 1 day | 1-10 sat/vB | No rush |

---

## UTXO Management

### What's a UTXO?

Bitcoin doesn't have "balances" - it has Unspent Transaction Outputs (UTXOs).

```
Your "balance" = Sum of all UTXOs you control

Example:
  UTXO 1: 0.5 BTC (from tx abc123)
  UTXO 2: 0.3 BTC (from tx def456)
  UTXO 3: 0.2 BTC (from tx ghi789)
  ─────────────────────────────────
  Balance: 1.0 BTC
```

### List UTXOs

```rust
use walletd::bitcoin::get_utxos;

let utxos = get_utxos(&address, &rpc).await?;

for utxo in &utxos {
    println!("{}:{} - {} sats", 
        utxo.txid, 
        utxo.vout, 
        utxo.value
    );
}
```

### UTXO Consolidation

Consolidate many small UTXOs when fees are low:

```rust
// Check if consolidation makes sense
let utxos = get_utxos(&address, &rpc).await?;
let fees = get_fee_estimates(&rpc).await?;

if fees.economy < 5 && utxos.len() > 10 {
    println!("Good time to consolidate {} UTXOs", utxos.len());
    
    let txid = consolidate_utxos(&address, &rpc).await?;
    println!("Consolidated to single UTXO: {}", txid);
}
```

### Dust Threshold

UTXOs below ~546 sats are considered "dust" and may not be spendable economically:

```rust
const DUST_THRESHOLD: u64 = 546;

let utxos = get_utxos(&address, &rpc).await?;
let dust_utxos: Vec<_> = utxos.iter()
    .filter(|u| u.value < DUST_THRESHOLD)
    .collect();

if !dust_utxos.is_empty() {
    println!("Warning: {} dust UTXOs ({} sats total)",
        dust_utxos.len(),
        dust_utxos.iter().map(|u| u.value).sum::<u64>()
    );
}
```

---

## Replace-By-Fee (RBF)

Speed up stuck transactions by replacing them with higher fees.

### Enable RBF

```rust
let txid = build_transaction()
    .from(&address)
    .to(&recipient, amount)
    .rbf(true)  // Enable replacement
    .broadcast(&rpc)
    .await?;
```

### Bump Fee

```rust
use walletd::bitcoin::bump_fee;

// Original transaction stuck in mempool
let original_txid = "abc123...";

// Bump fee to 50 sat/vB
let new_txid = bump_fee(original_txid, 50, &rpc).await?;
println!("Replaced {} with {}", original_txid, new_txid);
```

---

## Multi-Signature Wallets

### Create 2-of-3 Multisig

```rust
use walletd::bitcoin::multisig::{MultisigConfig, create_multisig_address};

// Three participants
let pubkeys = vec![
    "02abc...",  // Alice
    "02def...",  // Bob  
    "02ghi...",  // Carol
];

// 2 of 3 required to spend
let config = MultisigConfig {
    threshold: 2,
    pubkeys: pubkeys,
};

let address = create_multisig_address(&config, WalletMode::Mainnet)?;
println!("Multisig address: {}", address);  // bc1q... (P2WSH)
```

### Sign Multisig Transaction

```rust
use walletd::bitcoin::multisig::{create_multisig_tx, add_signature};

// Step 1: Create unsigned transaction
let mut tx = create_multisig_tx(&config, &to, amount)?;
println!("Share this with signers: {}", tx.to_hex());

// Step 2: Alice signs
let alice_sig = alice_wallet.sign(&tx)?;
add_signature(&mut tx, &alice_sig, 0)?;

// Step 3: Bob signs  
let bob_sig = bob_wallet.sign(&tx)?;
add_signature(&mut tx, &bob_sig, 1)?;

// Step 4: Threshold met - broadcast
assert!(tx.is_complete());  // 2 of 3 signed
let txid = broadcast(&tx, &rpc).await?;
```

---

## Watch-Only Wallets

Monitor addresses without private keys:

```rust
use walletd::bitcoin::watch_only::WatchOnlyWallet;

// Create watch-only wallet from xpub
let xpub = "xpub6CUG...";
let wallet = WatchOnlyWallet::from_xpub(xpub)?;

// Generate receiving addresses
for i in 0..5 {
    let addr = wallet.get_address(i)?;
    println!("Address {}: {}", i, addr);
}

// Monitor for incoming transactions
let balance = wallet.get_total_balance(&rpc).await?;
let txs = wallet.get_all_transactions(&rpc).await?;
```

---

## Testnet Development

### 1. Get Testnet Coins

| Faucet | URL | Amount |
|--------|-----|--------|
| Coinfaucet | [coinfaucet.eu/btc-testnet](https://coinfaucet.eu/en/btc-testnet/) | 0.01 tBTC |
| Bitcoin Testnet | [bitcoinfaucet.uo1.net](https://bitcoinfaucet.uo1.net/) | 0.001 tBTC |
| Mempool | [mempool.space/testnet/faucet](https://mempool.space/testnet/faucet) | 0.01 tBTC |

### 2. Use Testnet Mode

```rust
// All functions work the same, just change the mode
let address = derive_address(mnemonic, WalletMode::Testnet)?;
assert!(address.starts_with("tb1q"));

let balance = get_balance(&address, "https://blockstream.info/testnet/api").await?;
```

### 3. Testnet Block Explorers

- [mempool.space/testnet](https://mempool.space/testnet)
- [blockstream.info/testnet](https://blockstream.info/testnet)

---

## Production Checklist

Before going live:

- [ ] **Key Security**: Mnemonic stored securely (HSM, hardware wallet, or encrypted)
- [ ] **Address Validation**: Validate all addresses before sending
- [ ] **Amount Validation**: Check for overflow, dust, and sufficient balance
- [ ] **Fee Estimation**: Use real-time fee estimates, not hardcoded values
- [ ] **Error Handling**: Handle all error cases gracefully
- [ ] **Rate Limiting**: Respect RPC provider limits
- [ ] **Monitoring**: Log all transactions for audit trail
- [ ] **Backup**: Test wallet recovery from mnemonic
- [ ] **Testing**: Full test coverage on testnet first

```rust
// Production-ready send function
async fn safe_send(
    from: &str,
    to: &str, 
    amount: u64,
    rpc: &str,
) -> Result<String, WalletError> {
    // 1. Validate addresses
    if !is_valid_address(from) || !is_valid_address(to) {
        return Err(WalletError::InvalidAddress);
    }
    
    // 2. Check balance
    let balance: u64 = get_balance(from, rpc).await?.parse()?;
    let fee = estimate_fee(rpc, 1, 2).await?;
    
    if amount + fee > balance {
        return Err(WalletError::InsufficientFunds);
    }
    
    // 3. Check dust
    if amount < 546 {
        return Err(WalletError::DustAmount);
    }
    
    // 4. Send with logging
    let txid = send_transaction(from, to, amount, rpc).await?;
    log::info!("Sent {} sats from {} to {}: {}", amount, from, to, txid);
    
    Ok(txid)
}
```

---

## Common Patterns

### Convert Satoshis ↔ BTC

```rust
const SATS_PER_BTC: f64 = 100_000_000.0;

fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / SATS_PER_BTC
}

fn btc_to_sats(btc: f64) -> u64 {
    (btc * SATS_PER_BTC) as u64
}

// Format for display
fn format_btc(sats: u64) -> String {
    format!("{:.8} BTC", sats_to_btc(sats))
}
```

### Validate Address

```rust
fn is_valid_bitcoin_address(address: &str, network: WalletMode) -> bool {
    match network {
        WalletMode::Mainnet => {
            address.starts_with("bc1q") ||  // SegWit
            address.starts_with("bc1p") ||  // Taproot
            address.starts_with("1") ||     // Legacy
            address.starts_with("3")        // Script
        }
        WalletMode::Testnet => {
            address.starts_with("tb1q") ||
            address.starts_with("tb1p") ||
            address.starts_with("m") ||
            address.starts_with("n") ||
            address.starts_with("2")
        }
        WalletMode::Demo => true,
    }
}
```

### Wait for Confirmation

```rust
use tokio::time::{sleep, Duration};

async fn wait_for_confirmation(
    txid: &str,
    rpc: &str,
    target_confirmations: u32,
) -> Result<u32, WalletError> {
    loop {
        let status = get_transaction_status(txid, rpc).await?;
        
        if status.confirmations >= target_confirmations {
            return Ok(status.confirmations);
        }
        
        println!("Waiting... {} confirmations", status.confirmations);
        sleep(Duration::from_secs(30)).await;
    }
}

// Usage
let confs = wait_for_confirmation(&txid, &rpc, 6).await?;
println!("Transaction confirmed with {} confirmations", confs);
```

---

## Troubleshooting

### "Insufficient funds" but I have balance

**Cause**: Balance exists but UTXOs are too small after fees.

```rust
// Check if you can actually spend
let balance: u64 = get_balance(&addr, &rpc).await?.parse()?;
let fee = estimate_fee(&rpc, 1, 2).await?;
let dust = 546;

let spendable = balance.saturating_sub(fee + dust);
println!("Spendable: {} sats", spendable);
```

### Transaction stuck in mempool

**Cause**: Fee too low for current mempool conditions.

```rust
// Option 1: Wait it out (may take days)
// Option 2: RBF bump fee
let new_txid = bump_fee(&old_txid, higher_fee_rate, &rpc).await?;

// Option 3: CPFP (Child Pays For Parent)
// Spend the unconfirmed output with a high fee
```

### Invalid address error

**Cause**: Address format doesn't match network.

```rust
// Common mistake: Using mainnet address on testnet
let addr = derive_address(mnemonic, WalletMode::Mainnet)?;
// ❌ This will fail:
get_balance(&addr, "https://blockstream.info/testnet/api").await?;

// ✅ Fix: Match address to network
let addr = derive_address(mnemonic, WalletMode::Testnet)?;
get_balance(&addr, "https://blockstream.info/testnet/api").await?;
```

### Rate limited by RPC

**Cause**: Too many requests.

```rust
// Add delays between requests
for addr in addresses {
    let balance = get_balance(&addr, &rpc).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// Or use a dedicated RPC provider
```

---

## Resources

- [Bitcoin Developer Guide](https://developer.bitcoin.org/devguide/)
- [BIP-32 HD Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP-39 Mnemonic](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
- [BIP-84 SegWit](https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki)
- [Mempool.space](https://mempool.space) - Best block explorer
- [Blockstream Esplora API](https://github.com/Blockstream/esplora/blob/master/API.md)
