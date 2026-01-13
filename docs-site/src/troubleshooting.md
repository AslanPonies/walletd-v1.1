# Troubleshooting

> **Fix common issues fast.** Every error, explained with solutions.

---

## Quick Diagnosis

| Symptom | Likely Cause | Jump To |
|---------|--------------|---------|
| "Invalid mnemonic" | Wrong word count or typo | [Mnemonic Errors](#mnemonic-errors) |
| "Network error" | RPC unreachable | [Network Errors](#network-errors) |
| "Insufficient funds" | Balance too low for tx + fee | [Transaction Errors](#transaction-errors) |
| Transaction stuck | Gas too low | [Stuck Transactions](#stuck-transactions) |
| Wrong address format | Network mismatch | [Address Errors](#address-errors) |
| Build fails | Missing dependencies | [Build Errors](#build-errors) |

---

## Mnemonic Errors

### "InvalidMnemonic"

**Symptoms:**
```
Error: InvalidMnemonic
```

**Causes & Solutions:**

| Cause | Solution |
|-------|----------|
| Wrong word count | Must be exactly 12 or 24 words |
| Typo in a word | Check against [BIP-39 word list](https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt) |
| Extra spaces | Trim whitespace, single space between words |
| Wrong language | Use English word list |

**Debug:**
```rust
fn validate_mnemonic(phrase: &str) -> Result<(), String> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    
    // Check word count
    if words.len() != 12 && words.len() != 24 {
        return Err(format!(
            "Expected 12 or 24 words, got {}", 
            words.len()
        ));
    }
    
    // Check each word (simplified)
    let valid_words = include_str!("bip39-english.txt")
        .lines()
        .collect::<Vec<_>>();
    
    for (i, word) in words.iter().enumerate() {
        if !valid_words.contains(word) {
            return Err(format!(
                "Word {} '{}' is not in BIP-39 list. Did you mean '{}'?",
                i + 1,
                word,
                find_similar(word, &valid_words)
            ));
        }
    }
    
    Ok(())
}
```

---

## Network Errors

### "NetworkError: connection refused"

**Symptoms:**
```
Error: NetworkError("connection refused")
Error: NetworkError("timeout")
Error: NetworkError("DNS resolution failed")
```

**Solutions:**

1. **Check internet connection**
   ```bash
   curl -I https://blockstream.info/api
   ```

2. **Try alternative RPC**
   ```rust
   // Instead of
   let rpc = "https://eth.llamarpc.com";
   // Try
   let rpc = "https://rpc.ankr.com/eth";
   // Or
   let rpc = "https://cloudflare-eth.com";
   ```

3. **Check if RPC is down**
   - Bitcoin: [blockstream.info/api](https://blockstream.info/api)
   - Ethereum: [etherscan.io/apis](https://etherscan.io/apis)

4. **Handle timeouts gracefully**
   ```rust
   use tokio::time::timeout;
   
   let result = timeout(
       Duration::from_secs(30),
       get_balance(&address, &rpc)
   ).await;
   
   match result {
       Ok(Ok(balance)) => println!("Balance: {}", balance),
       Ok(Err(e)) => eprintln!("RPC error: {}", e),
       Err(_) => eprintln!("Request timed out - try another RPC"),
   }
   ```

### "Rate limited"

**Symptoms:**
```
Error: RpcError(429)
Error: "Too many requests"
```

**Solutions:**

1. **Add delays between requests**
   ```rust
   for address in addresses {
       let balance = get_balance(&address, &rpc).await?;
       tokio::time::sleep(Duration::from_millis(100)).await;
   }
   ```

2. **Use a paid RPC provider**
   - [Infura](https://infura.io)
   - [Alchemy](https://alchemy.com)
   - [QuickNode](https://quicknode.com)

3. **Implement exponential backoff**
   ```rust
   async fn with_retry<T, F, Fut>(f: F, max_retries: u32) -> Result<T, Error>
   where
       F: Fn() -> Fut,
       Fut: Future<Output = Result<T, Error>>,
   {
       let mut delay = 1;
       for attempt in 0..max_retries {
           match f().await {
               Ok(result) => return Ok(result),
               Err(e) if e.is_rate_limit() => {
                   tokio::time::sleep(Duration::from_secs(delay)).await;
                   delay *= 2;  // Exponential backoff
               }
               Err(e) => return Err(e),
           }
       }
       Err(Error::MaxRetriesExceeded)
   }
   ```

---

## Transaction Errors

### "InsufficientFunds"

**Symptoms:**
```
Error: InsufficientFunds
Error: "insufficient funds for transfer"
```

**Cause**: Balance < amount + fees

**Debug:**
```rust
async fn check_if_sendable(
    address: &str,
    amount: u64,
    rpc: &str,
) -> Result<bool, Error> {
    let balance: u64 = get_balance(address, rpc).await?.parse()?;
    let fee = estimate_fee(rpc).await?;
    
    println!("Balance:  {} sats", balance);
    println!("Amount:   {} sats", amount);
    println!("Est. fee: {} sats", fee);
    println!("Total:    {} sats", amount + fee);
    println!("Shortfall: {} sats", (amount + fee).saturating_sub(balance));
    
    Ok(balance >= amount + fee)
}
```

### "Dust amount"

**Symptoms:**
```
Error: "Output below dust threshold"
```

**Cause**: Sending less than minimum viable amount

**Solution:**
```rust
const DUST_THRESHOLD: u64 = 546;  // Bitcoin

fn validate_amount(amount: u64) -> Result<(), Error> {
    if amount < DUST_THRESHOLD {
        return Err(Error::DustAmount(format!(
            "Amount {} is below dust threshold {}",
            amount, DUST_THRESHOLD
        )));
    }
    Ok(())
}
```

---

## Stuck Transactions

### Bitcoin: Transaction not confirming

**Symptoms:**
- Transaction in mempool for hours/days
- 0 confirmations

**Diagnosis:**
```rust
// Check current fee market
let fees = get_fee_estimates(&rpc).await?;
println!("Current fees:");
println!("  Fast (10 min): {} sat/vB", fees.fastest);
println!("  Medium (30 min): {} sat/vB", fees.half_hour);
println!("  Slow (1 hour): {} sat/vB", fees.hour);

// Check your transaction's fee
let tx = get_transaction(&txid, &rpc).await?;
println!("Your tx fee: {} sat/vB", tx.fee_rate);

if tx.fee_rate < fees.hour {
    println!("⚠️ Fee too low - transaction may be stuck");
}
```

**Solutions:**

1. **Wait** - Mempool clears during low activity (weekends, nights)

2. **RBF (Replace-By-Fee)** - If enabled
   ```rust
   let new_txid = bump_fee(&old_txid, new_fee_rate, &rpc).await?;
   ```

3. **CPFP (Child-Pays-For-Parent)** - Spend the unconfirmed output
   ```rust
   // Create new tx spending unconfirmed output with high fee
   ```

### Ethereum: Transaction pending

**Diagnosis:**
```rust
let nonce = get_nonce(&address, &rpc).await?;
let pending_nonce = get_pending_nonce(&address, &rpc).await?;

if pending_nonce > nonce {
    println!("You have {} pending transactions", pending_nonce - nonce);
}
```

**Solutions:**

1. **Speed up** - Send same tx with higher gas
   ```rust
   let pending = get_pending_tx(&address, &rpc).await?;
   send_transaction_with_nonce(
       &from, &to, &value,
       pending.nonce,
       pending.gas_price * 2,  // Double gas
       &key, &rpc
   ).await?;
   ```

2. **Cancel** - Send 0 ETH to yourself with same nonce
   ```rust
   send_transaction_with_nonce(
       &from, &from,  // To self
       "0",           // Zero value
       stuck_nonce,
       high_gas_price,
       &key, &rpc
   ).await?;
   ```

---

## Address Errors

### "Invalid address"

**Symptoms:**
```
Error: InvalidAddress
Error: "Invalid checksum"
```

**Causes:**

| Issue | Example | Fix |
|-------|---------|-----|
| Missing prefix | `1234abcd...` | Add `0x` for Ethereum |
| Wrong length | `0x123` | Must be 42 chars (ETH) |
| Bad characters | `0xGGGG...` | Only hex (0-9, a-f) |
| Wrong network | `bc1q...` on testnet | Use `tb1q...` for testnet |
| Failed checksum | Mixed case wrong | Use checksummed address |

**Validation:**
```rust
fn validate_address(address: &str, chain: &str) -> Result<(), String> {
    match chain {
        "BTC" => {
            if !address.starts_with("bc1") && 
               !address.starts_with("1") && 
               !address.starts_with("3") {
                return Err("Invalid Bitcoin address prefix".into());
            }
        }
        "ETH" => {
            if !address.starts_with("0x") {
                return Err("Ethereum address must start with 0x".into());
            }
            if address.len() != 42 {
                return Err(format!(
                    "Ethereum address must be 42 chars, got {}", 
                    address.len()
                ));
            }
        }
        _ => {}
    }
    Ok(())
}
```

---

## Build Errors

### "OpenSSL not found"

**macOS:**
```bash
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
export PKG_CONFIG_PATH=$(brew --prefix openssl)/lib/pkgconfig
```

**Ubuntu:**
```bash
sudo apt-get install libssl-dev pkg-config
```

### "secp256k1 build failed"

**macOS:**
```bash
brew install automake autoconf libtool
```

**Ubuntu:**
```bash
sudo apt-get install build-essential automake autoconf libtool
```

### "Can't find crate"

```bash
# Clear cargo cache and rebuild
cargo clean
cargo build
```

---

## Common Mistakes

### 1. Using mainnet mnemonic on testnet

```rust
// ❌ Wrong - mainnet address on testnet RPC
let addr = derive_address(mnemonic, WalletMode::Mainnet)?;
get_balance(&addr, "https://blockstream.info/testnet/api").await?;

// ✅ Correct - match mode to RPC
let addr = derive_address(mnemonic, WalletMode::Testnet)?;
get_balance(&addr, "https://blockstream.info/testnet/api").await?;
```

### 2. Hardcoding gas prices

```rust
// ❌ Wrong - gas prices change constantly
let gas_price = 20_000_000_000; // 20 gwei

// ✅ Correct - always estimate
let gas = get_gas_price(&rpc).await?;
let gas_price = gas.base_fee + gas.priority_fee;
```

### 3. Not handling async properly

```rust
// ❌ Wrong - blocking in async context
let balance = futures::executor::block_on(get_balance(&addr, &rpc));

// ✅ Correct - use async/await
let balance = get_balance(&addr, &rpc).await?;
```

### 4. Ignoring errors

```rust
// ❌ Wrong - silent failure
let balance = get_balance(&addr, &rpc).await.unwrap_or_default();

// ✅ Correct - handle errors
let balance = match get_balance(&addr, &rpc).await {
    Ok(b) => b,
    Err(e) => {
        eprintln!("Failed to get balance: {}", e);
        return Err(e.into());
    }
};
```

---

## Still Stuck?

1. **Check GitHub Issues**: [github.com/AslanPonies/walletd-v1.1/issues](https://github.com/AslanPonies/walletd-v1.1/issues)
2. **Search closed issues** - your problem may be solved
3. **Open new issue** with:
   - WalletD version
   - Rust version (`rustc --version`)
   - OS and version
   - Full error message
   - Minimal reproduction code

---

## Debug Mode

Enable verbose logging:

```rust
// At start of main()
std::env::set_var("RUST_LOG", "walletd=debug");
env_logger::init();
```

Or via environment:
```bash
RUST_LOG=walletd=debug cargo run
```
