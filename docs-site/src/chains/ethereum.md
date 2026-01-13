# Ethereum Deep Dive

> **Complete Ethereum guide** — From basic transfers to ERC-20 tokens and gas optimization.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Complete API Reference](#complete-api-reference)
3. [Gas & Fees Explained](#gas--fees-explained)
4. [ERC-20 Tokens](#erc-20-tokens)
5. [Transaction Types](#transaction-types)
6. [Nonce Management](#nonce-management)
7. [Smart Contract Interaction](#smart-contract-interaction)
8. [L2 Networks](#l2-networks)
9. [ENS Resolution](#ens-resolution)
10. [Production Patterns](#production-patterns)
11. [Troubleshooting](#troubleshooting)

---

## Quick Start

```rust
use walletd::ethereum::{derive_address, get_balance, send_transaction};
use walletd::types::WalletMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = "your 24 word mnemonic here";
    
    // 1. Derive address
    let address = derive_address(mnemonic, WalletMode::Mainnet)?;
    println!("Address: {}", address);  // 0x...
    
    // 2. Check balance
    let balance_wei = get_balance(&address, "https://eth.llamarpc.com").await?;
    let eth = wei_to_eth(&balance_wei);
    println!("Balance: {:.6} ETH", eth);
    
    // 3. Send ETH
    let tx_hash = send_transaction(
        &from_address,
        &to_address,
        "1000000000000000000",  // 1 ETH in wei
        &private_key,
        "https://eth.llamarpc.com"
    ).await?;
    
    println!("TX: https://etherscan.io/tx/{}", tx_hash);
    
    Ok(())
}

fn wei_to_eth(wei: &str) -> f64 {
    wei.parse::<u128>().unwrap_or(0) as f64 / 1e18
}
```

---

## Complete API Reference

### derive_address

```rust
pub fn derive_address(mnemonic: &str, mode: WalletMode) -> Result<String>
```

**Derivation Path:** `m/44'/60'/0'/0/0`

**Returns:** Checksummed address (EIP-55)

```rust
let address = derive_address(mnemonic, WalletMode::Mainnet)?;
// → "0x742d35Cc6634C0532925a3b844Bc9e7595f8c2a1"
//    ^^-- Note mixed case (checksum)

// Same address works on all EVM chains!
// Mainnet, Goerli, Sepolia, Polygon, Arbitrum, Base, etc.
```

---

### get_balance

```rust
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String>
```

**Returns:** Balance in **wei** as a string (1 ETH = 10^18 wei)

```rust
let wei = get_balance(&address, "https://eth.llamarpc.com").await?;

// Convert to ETH
let wei_u128: u128 = wei.parse()?;
let eth = wei_u128 as f64 / 1_000_000_000_000_000_000.0;
println!("{:.6} ETH", eth);
```

**Free RPC Endpoints:**

| Network | URL | Chain ID |
|---------|-----|----------|
| Mainnet | `https://eth.llamarpc.com` | 1 |
| Mainnet | `https://rpc.ankr.com/eth` | 1 |
| Mainnet | `https://cloudflare-eth.com` | 1 |
| Sepolia | `https://rpc.sepolia.org` | 11155111 |
| Sepolia | `https://ethereum-sepolia.blockpi.network/v1/rpc/public` | 11155111 |

---

### send_transaction

```rust
pub async fn send_transaction(
    from: &str,
    to: &str,
    amount_wei: &str,
    private_key: &str,
    rpc_url: &str,
) -> Result<String>
```

**Features:**
- ✅ EIP-1559 transactions (Type 2)
- ✅ Automatic gas estimation
- ✅ Automatic nonce management
- ✅ Chain ID auto-detection

```rust
// Send 0.1 ETH
let tx_hash = send_transaction(
    "0xYourAddress...",
    "0xRecipient...",
    "100000000000000000",  // 0.1 ETH
    "0xYourPrivateKey...",
    "https://eth.llamarpc.com"
).await?;
```

---

## Gas & Fees Explained

### The Gas Model

```
Transaction Cost = Gas Used × Gas Price

Where:
- Gas Used: Computational units consumed (e.g., 21,000 for ETH transfer)
- Gas Price: Price per unit in gwei (fluctuates with demand)
```

### EIP-1559 Fees (Post-London)

```
Total Fee = Gas Used × (Base Fee + Priority Fee)

- Base Fee: Set by protocol, burned (destroyed)
- Priority Fee: Tip to validator, you set this
- Max Fee: Your maximum willingness to pay
```

```rust
use walletd::ethereum::{get_gas_price, send_transaction_eip1559};

// Get current gas prices
let gas = get_gas_price("https://eth.llamarpc.com").await?;
println!("Base Fee: {} gwei", gas.base_fee / 1e9);
println!("Priority Fee: {} gwei", gas.priority_fee / 1e9);

// Send with custom gas settings
let tx_hash = send_transaction_eip1559(
    &from,
    &to,
    amount,
    &key,
    21_000,                    // gas_limit
    gas.base_fee * 2,         // max_fee (2x buffer for base fee increase)
    2_000_000_000,            // priority_fee (2 gwei tip)
    &rpc
).await?;
```

### Gas Limits by Operation

| Operation | Typical Gas | At 30 gwei |
|-----------|-------------|------------|
| ETH transfer | 21,000 | ~$1.50 |
| ERC-20 transfer | 65,000 | ~$4.50 |
| ERC-20 approve | 45,000 | ~$3.15 |
| Uniswap swap | 150,000-300,000 | ~$15-30 |
| NFT mint | 100,000-200,000 | ~$10-20 |
| Contract deploy | 500,000+ | ~$50+ |

### Real-Time Gas Tracking

```rust
// Monitor gas prices
loop {
    let gas = get_gas_price(&rpc).await?;
    let eth_transfer_cost = 21_000.0 * (gas.base_fee + gas.priority_fee) / 1e18;
    
    println!("ETH transfer cost: ${:.2}", eth_transfer_cost * eth_price);
    
    if eth_transfer_cost < 1.0 {
        println!("Low gas! Good time to transact.");
        break;
    }
    
    tokio::time::sleep(Duration::from_secs(60)).await;
}
```

---

## ERC-20 Tokens

### Check Token Balance

```rust
use walletd::ethereum::erc20;

// USDC on mainnet
let usdc_contract = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
let balance = erc20::get_balance(usdc_contract, &wallet_address, &rpc).await?;

// USDC has 6 decimals
let usdc_amount = balance.parse::<u64>()? as f64 / 1_000_000.0;
println!("{:.2} USDC", usdc_amount);
```

### Common Token Contracts (Mainnet)

| Token | Address | Decimals |
|-------|---------|----------|
| USDC | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | 6 |
| USDT | `0xdAC17F958D2ee523a2206206994597C13D831ec7` | 6 |
| DAI | `0x6B175474E89094C44Da98b954EesacDeE97AE4` | 18 |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | 18 |
| WBTC | `0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599` | 8 |
| LINK | `0x514910771AF9Ca656af840dff83E8264EcF986CA` | 18 |
| UNI | `0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984` | 18 |

### Transfer Tokens

```rust
// Approve + Transfer pattern
// Step 1: Approve spender (if needed)
let approve_hash = erc20::approve(
    usdc_contract,
    spender_address,
    amount,
    &private_key,
    &rpc
).await?;

// Step 2: Transfer
let transfer_hash = erc20::transfer(
    usdc_contract,
    recipient,
    amount,
    &private_key,
    &rpc
).await?;
```

### Get Token Info

```rust
let info = erc20::get_token_info(contract, &rpc).await?;
println!("Name: {}", info.name);        // "USD Coin"
println!("Symbol: {}", info.symbol);    // "USDC"
println!("Decimals: {}", info.decimals); // 6
println!("Total Supply: {}", info.total_supply);
```

---

## Transaction Types

### Type 0 (Legacy)

```rust
// Pre-EIP-1559, still supported
let tx = LegacyTransaction {
    nonce: 0,
    gas_price: 30_000_000_000,  // 30 gwei
    gas_limit: 21_000,
    to: recipient,
    value: amount,
    data: vec![],
};
```

### Type 2 (EIP-1559) - Recommended

```rust
// Modern transaction type
let tx = Eip1559Transaction {
    chain_id: 1,
    nonce: 0,
    max_priority_fee_per_gas: 2_000_000_000,  // 2 gwei tip
    max_fee_per_gas: 100_000_000_000,         // 100 gwei max
    gas_limit: 21_000,
    to: recipient,
    value: amount,
    data: vec![],
    access_list: vec![],
};
```

---

## Nonce Management

### What's a Nonce?

Every transaction has a sequential nonce starting at 0. Transactions must be mined in order.

```
Address: 0xABC...
├── Nonce 0: ✅ Confirmed
├── Nonce 1: ✅ Confirmed
├── Nonce 2: ⏳ Pending
├── Nonce 3: 🔒 Queued (waiting for 2)
└── Nonce 4: 🔒 Queued (waiting for 2, 3)
```

### Get Current Nonce

```rust
use walletd::ethereum::get_nonce;

// Next nonce to use
let nonce = get_nonce(&address, &rpc).await?;
println!("Next nonce: {}", nonce);

// Pending nonce (includes mempool)
let pending_nonce = get_pending_nonce(&address, &rpc).await?;
```

### Handle Nonce Gaps

```rust
// Problem: Transaction stuck, blocking all future txs
// Solution: Replace with same nonce

let stuck_nonce = 5;
let tx_hash = send_transaction_with_nonce(
    &from,
    &to,
    "0",  // Can send 0 ETH to self
    stuck_nonce,
    high_gas_price,  // Higher than stuck tx
    &key,
    &rpc
).await?;
```

### Parallel Transaction Sending

```rust
// Send multiple transactions without waiting
let base_nonce = get_nonce(&address, &rpc).await?;

let futures: Vec<_> = (0..5).map(|i| {
    let nonce = base_nonce + i;
    send_transaction_with_nonce(&from, &to, amount, nonce, &key, &rpc)
}).collect();

let results = futures::future::join_all(futures).await;
```

---

## Smart Contract Interaction

### Read Contract (Free)

```rust
use walletd::ethereum::contract;

// Call view/pure function
let result = contract::call(
    contract_address,
    "balanceOf(address)",
    &[wallet_address],
    &rpc
).await?;
```

### Write Contract (Costs Gas)

```rust
// Call state-changing function
let tx_hash = contract::send(
    contract_address,
    "transfer(address,uint256)",
    &[recipient, amount],
    &private_key,
    &rpc
).await?;
```

### ABI Encoding

```rust
use walletd::ethereum::abi;

// Encode function call
let data = abi::encode_function_call(
    "transfer(address,uint256)",
    &[
        abi::Address(recipient),
        abi::Uint256(amount),
    ]
)?;

// Decode return value
let balance: U256 = abi::decode_output("uint256", &result)?;
```

---

## L2 Networks

All EVM L2s use the same API - just change the RPC URL.

### Network Configuration

| Network | RPC | Chain ID | Native Token |
|---------|-----|----------|--------------|
| Base | `https://mainnet.base.org` | 8453 | ETH |
| Arbitrum | `https://arb1.arbitrum.io/rpc` | 42161 | ETH |
| Optimism | `https://mainnet.optimism.io` | 10 | ETH |
| Polygon | `https://polygon-rpc.com` | 137 | MATIC |
| Avalanche | `https://api.avax.network/ext/bc/C/rpc` | 43114 | AVAX |

### Multi-Chain Balance Check

```rust
let networks = vec![
    ("Ethereum", "https://eth.llamarpc.com", 18, "ETH"),
    ("Base", "https://mainnet.base.org", 18, "ETH"),
    ("Polygon", "https://polygon-rpc.com", 18, "MATIC"),
    ("Arbitrum", "https://arb1.arbitrum.io/rpc", 18, "ETH"),
];

for (name, rpc, decimals, symbol) in networks {
    let balance = get_balance(&address, rpc).await?;
    let amount = balance.parse::<u128>()? as f64 / 10f64.powi(decimals);
    println!("{}: {:.6} {}", name, amount, symbol);
}
```

### Bridging Considerations

```rust
// Same address on all EVM chains
let address = derive_address(mnemonic, WalletMode::Mainnet)?;

// This address works on:
// - Ethereum mainnet
// - Base
// - Polygon  
// - Arbitrum
// - Optimism
// - Avalanche C-Chain
// - Any EVM chain!
```

---

## ENS Resolution

### Resolve ENS Name

```rust
use walletd::ethereum::ens;

// vitalik.eth → 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
let address = ens::resolve("vitalik.eth", &rpc).await?;
println!("Address: {}", address);
```

### Reverse Lookup

```rust
// 0xd8dA6BF2... → vitalik.eth
let name = ens::reverse_lookup(&address, &rpc).await?;
if let Some(ens_name) = name {
    println!("ENS: {}", ens_name);
}
```

### Safe Send with ENS

```rust
async fn send_to(recipient: &str, amount: &str, key: &str, rpc: &str) -> Result<String> {
    // Resolve ENS if needed
    let to_address = if recipient.ends_with(".eth") {
        ens::resolve(recipient, rpc).await?
    } else {
        recipient.to_string()
    };
    
    send_transaction(&from, &to_address, amount, key, rpc).await
}

// Usage
send_to("vitalik.eth", "1000000000000000000", &key, &rpc).await?;
```

---

## Production Patterns

### Unit Conversion Helpers

```rust
const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;
const WEI_PER_GWEI: u128 = 1_000_000_000;

fn eth_to_wei(eth: f64) -> u128 {
    (eth * WEI_PER_ETH as f64) as u128
}

fn wei_to_eth(wei: u128) -> f64 {
    wei as f64 / WEI_PER_ETH as f64
}

fn gwei_to_wei(gwei: f64) -> u128 {
    (gwei * WEI_PER_GWEI as f64) as u128
}

fn wei_to_gwei(wei: u128) -> f64 {
    wei as f64 / WEI_PER_GWEI as f64
}

// Format for display
fn format_eth(wei: u128) -> String {
    format!("{:.6} ETH", wei_to_eth(wei))
}
```

### Address Validation

```rust
fn is_valid_eth_address(address: &str) -> bool {
    // Check format
    if !address.starts_with("0x") {
        return false;
    }
    if address.len() != 42 {
        return false;
    }
    
    // Check hex characters
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

// Checksum validation (EIP-55)
fn is_checksummed(address: &str) -> bool {
    use sha3::{Digest, Keccak256};
    
    let addr_lower = &address[2..].to_lowercase();
    let hash = hex::encode(Keccak256::digest(addr_lower.as_bytes()));
    
    for (i, c) in address[2..].chars().enumerate() {
        if c.is_alphabetic() {
            let should_upper = hash.chars().nth(i).unwrap() >= '8';
            if should_upper != c.is_uppercase() {
                return false;
            }
        }
    }
    true
}
```

### Retry Logic

```rust
async fn send_with_retry(
    from: &str,
    to: &str,
    amount: &str,
    key: &str,
    rpc: &str,
    max_retries: u32,
) -> Result<String> {
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match send_transaction(from, to, amount, key, rpc).await {
            Ok(hash) => return Ok(hash),
            Err(e) => {
                eprintln!("Attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
                
                // Exponential backoff
                let delay = 2u64.pow(attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

---

## Troubleshooting

### "Insufficient funds for gas"

**Cause**: Not enough ETH to cover gas fees.

```rust
// Check if you can afford the transaction
let balance: u128 = get_balance(&address, &rpc).await?.parse()?;
let gas = get_gas_price(&rpc).await?;
let gas_cost = 21_000 * (gas.base_fee + gas.priority_fee);
let total_needed = amount + gas_cost;

if balance < total_needed {
    println!("Need {} wei more", total_needed - balance);
}
```

### "Nonce too low"

**Cause**: Transaction with this nonce already confirmed.

```rust
// Get correct nonce
let nonce = get_pending_nonce(&address, &rpc).await?;
// Use this nonce for your transaction
```

### "Replacement transaction underpriced"

**Cause**: Trying to replace pending tx with lower gas.

```rust
// Get pending transaction gas price
let pending = get_pending_transaction(&address, &rpc).await?;

// New gas must be at least 10% higher
let new_gas = (pending.gas_price as f64 * 1.1) as u128;
```

### Transaction stuck pending

```rust
// Option 1: Wait (can take hours during congestion)

// Option 2: Speed up (replace with higher gas)
let pending = get_pending_transaction(&address, &rpc).await?;
let speed_up_hash = send_transaction_with_nonce(
    &from,
    &pending.to,
    &pending.value,
    pending.nonce,
    pending.gas_price * 2,  // Double the gas
    &key,
    &rpc
).await?;

// Option 3: Cancel (send 0 to yourself with same nonce)
let cancel_hash = send_transaction_with_nonce(
    &from,
    &from,  // Send to self
    "0",    // Zero value
    pending.nonce,
    pending.gas_price * 2,
    &key,
    &rpc
).await?;
```

---

## Testnet Faucets

| Network | Faucet | Amount |
|---------|--------|--------|
| Sepolia | [sepoliafaucet.com](https://sepoliafaucet.com) | 0.5 ETH/day |
| Sepolia | [Alchemy](https://sepoliafaucet.com) | 0.5 ETH/day |
| Goerli | [goerlifaucet.com](https://goerlifaucet.com) | 0.25 ETH |

---

## Resources

- [Ethereum Docs](https://ethereum.org/developers)
- [EIP-1559 Explained](https://eips.ethereum.org/EIPS/eip-1559)
- [ERC-20 Standard](https://eips.ethereum.org/EIPS/eip-20)
- [Etherscan](https://etherscan.io) - Block Explorer
- [eth.llamarpc.com](https://eth.llamarpc.com) - Free RPC
