# WalletD API Documentation

WalletD is a unified multi-chain cryptocurrency wallet SDK built in Rust. This document provides comprehensive API documentation for all supported blockchains.

## Table of Contents

- [Overview](#overview)
- [Supported Chains](#supported-chains)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Traits](#core-traits)
- [Chain-Specific APIs](#chain-specific-apis)
  - [Ethereum](#ethereum-api)
  - [Bitcoin](#bitcoin-api)
  - [Solana](#solana-api)
  - [Base L2](#base-l2-api)
  - [ERC-20 Tokens](#erc-20-api)
  - [ICP (Internet Computer)](#icp-api)
  - [Hedera](#hedera-api)
  - [Monero](#monero-api)
- [Error Handling](#error-handling)
- [Security Best Practices](#security-best-practices)

---

## Overview

WalletD provides a consistent API across multiple blockchain networks, making it easy to build applications that support multiple cryptocurrencies. Key features include:

- **Unified Traits**: Common interfaces for all wallets (`Wallet`, `Transferable`, `Syncable`)
- **HD Wallet Support**: BIP-32/39/44 hierarchical deterministic wallets
- **Type Safety**: Strong Rust types for amounts, addresses, and transactions
- **Async/Await**: Full async support for network operations
- **Security**: Zeroize-on-drop for sensitive data

---

## Supported Chains

| Chain | Package | Status | Network Operations |
|-------|---------|--------|-------------------|
| Ethereum | `walletd_ethereum` | ✅ Production | Yes |
| Bitcoin | `walletd_bitcoin` | ✅ Production | Yes |
| Solana | `walletd_solana` | ✅ Production | Yes |
| Base L2 | `walletd_base` | ✅ Production | Yes |
| ERC-20 | `walletd_erc20` | ✅ Production | Yes |
| ICP | `walletd_icp` | ✅ Production | Yes |
| Hedera | `walletd_hedera` | ✅ Production | Yes |
| Monero | `walletd_monero` | ✅ Production | Yes |
| SUI | `walletd_sui` | 🚧 Planned | - |
| Prasaga | `walletd_prasaga_avio` | ✅ Beta | Yes |

---

## Installation

Add WalletD to your `Cargo.toml`:

```toml
[dependencies]
# Core traits
walletd-traits = "0.1"

# Individual chains (pick what you need)
walletd_ethereum = "0.2"
walletd_bitcoin = "0.2"
walletd_solana = "0.2"
walletd_base = "0.2"
walletd_erc20 = "0.1"
walletd_icp = "0.1"
walletd_hedera = "0.2"
walletd_monero = "0.1"
```

---

## Quick Start

### Creating an Ethereum Wallet

```rust
use walletd_ethereum::EthereumWallet;

#[tokio::main]
async fn main() {
    // Generate a new random wallet
    let wallet = EthereumWallet::new(Network::Mainnet);
    
    println!("Address: {}", wallet.address());
    println!("Private Key: {}", wallet.private_key());
    
    // Connect to a provider
    wallet.connect_provider("https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY");
    
    // Get balance
    let balance = wallet.get_balance().await.unwrap();
    println!("Balance: {} ETH", balance.eth());
}
```

### Creating a Bitcoin Wallet from Mnemonic

```rust
use walletd_bitcoin::{BitcoinWallet, AddressType};
use bdk::keys::bip39::Mnemonic;

let mnemonic = Mnemonic::parse("your twelve word mnemonic phrase here")?;

let wallet = BitcoinWallet::builder()
    .mnemonic(mnemonic)
    .network_type(Network::Bitcoin)
    .address_format(AddressType::P2wpkh)  // Native SegWit
    .build()?;

println!("Address: {}", wallet.receive_address()?);
```

---

## Core Traits

All wallets implement the unified traits from `walletd-traits`:

### Wallet Trait

```rust
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Returns the primary address
    fn address(&self) -> String;
    
    /// Returns the current balance
    async fn balance(&self) -> WalletResult<Amount>;
    
    /// Returns the network
    fn network(&self) -> &Network;
    
    /// Returns currency symbol (e.g., "ETH", "BTC")
    fn currency_symbol(&self) -> &str;
    
    /// Returns decimal places
    fn decimals(&self) -> u8;
}
```

### Transferable Trait

```rust
#[async_trait]
pub trait Transferable: Wallet {
    /// Send funds to an address
    async fn transfer(&self, to: &str, amount: Amount) -> WalletResult<TxHash>;
    
    /// Estimate transaction fee
    async fn estimate_fee(&self, to: &str, amount: Amount) -> WalletResult<Amount>;
}
```

### Amount Type

```rust
// Create amounts
let eth = Amount::from_human(1.5, 18);     // 1.5 ETH
let btc = Amount::from_human(0.001, 8);    // 0.001 BTC
let wei = Amount::from_smallest_unit(1_000_000_000_000_000_000, 18);  // 1 ETH in wei

// Convert amounts
println!("{}", eth.human_readable());       // 1.5
println!("{}", eth.smallest_unit());        // 1500000000000000000
```

---

## Chain-Specific APIs

### Ethereum API

```rust
use walletd_ethereum::{EthereumWallet, EthereumAmount, Network};

// Create wallet
let wallet = EthereumWallet::new(Network::Mainnet);
let wallet = EthereumWallet::from_private_key("0x...", Network::Mainnet)?;

// Amount utilities
let amount = EthereumAmount::from_eth(1.5);
let amount = EthereumAmount::from_wei(U256::from(1_000_000_000_000_000_000u128));
let amount = EthereumAmount::from_gwei(21_000);

println!("{} ETH", amount.eth());
println!("{} Gwei", amount.gwei());
println!("{} Wei", amount.wei());

// Connect and query
wallet.connect_provider("https://...")?;
let balance = wallet.get_balance().await?;
let nonce = wallet.get_nonce().await?;

// Send transaction
let tx_hash = wallet.send_transaction(to_address, amount).await?;
```

### Bitcoin API

```rust
use walletd_bitcoin::{BitcoinWallet, BitcoinWalletBuilder, AddressType};
use bdk::bitcoin::Network;

// Create from mnemonic
let wallet = BitcoinWallet::builder()
    .mnemonic(mnemonic)
    .network_type(Network::Bitcoin)     // or Testnet, Regtest
    .address_format(AddressType::P2wpkh) // Native SegWit
    .build()?;

// Access wallet properties
let address = wallet.receive_address()?;       // Get receive address
let next = wallet.next_address()?;             // Get next address (increments)
let network = wallet.network()?;
let coin_type = wallet.coin_type_id()?;        // 0 for mainnet, 1 for testnet
let hd_purpose = wallet.default_hd_purpose()?; // BIP44/49/84

// Get balance (requires blockchain sync)
let balance = wallet.balance().await?;
println!("Confirmed: {} sats", balance.confirmed);
println!("Pending: {} sats", balance.trusted_pending);

// Transfer (requires blockchain)
let txid = wallet.transfer(&blockchain, amount_sats, "bc1q...").await?;
```

### Solana API

```rust
use walletd_solana::{SolanaClient, SolanaAccount};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};

// Create client
let client = SolanaClient::new("https://api.mainnet-beta.solana.com").await?;
let client = SolanaClient::new_with_commitment(url, CommitmentConfig::finalized()).await?;

// Create account from keypair
let keypair = Keypair::new();
let account = SolanaAccount::new_from_bytes(keypair.to_bytes())?;
println!("Pubkey: {}", account.pubkey());

// Query balance
let balance = client.get_balance(&pubkey).await?;  // Returns lamports

// Get account info
let account_info = client.get_account(&pubkey).await?;

// Transfer SOL
let confirmed = client.transfer(from_keypair, to_pubkey, lamports).await?;

// Request airdrop (devnet only)
let result = client.request_airdrop(pubkey).await?;
```

### Base L2 API

```rust
use walletd_base::BaseWallet;

// Create wallet
let wallet = BaseWallet::new(Network::Mainnet);  // Chain ID 8453
let wallet = BaseWallet::new(Network::Sepolia);  // Chain ID 84532

let wallet = BaseWallet::from_private_key("0x...", Network::Mainnet)?;

// Connect provider
wallet.connect_provider("https://mainnet.base.org")?;

// Get balance
let balance = wallet.get_balance().await?;  // Returns U256 in wei

// Send transaction
let tx_hash = wallet.send_transaction(to, amount).await?;
```

### ERC-20 API

```rust
use walletd_erc20::{UsdcAdapter, Erc20Adapter};

// Create USDC adapter (mainnet by default)
let usdc = UsdcAdapter::mainnet();
let usdc = UsdcAdapter::new(custom_contract_address);

// Query token info
let name = usdc.name(rpc_url).await?;         // "USD Coin"
let symbol = usdc.symbol(rpc_url).await?;     // "USDC"
let decimals = usdc.decimals(rpc_url).await?; // 6

// Query balances
let supply = usdc.total_supply(rpc_url).await?;
let balance = usdc.balance_of(rpc_url, owner_address).await?;
let allowance = usdc.allowance(rpc_url, owner, spender).await?;

// Function selectors
// name()       -> 0x06fdde03
// symbol()     -> 0x95d89b41
// decimals()   -> 0x313ce567
// totalSupply() -> 0x18160ddd
// balanceOf(address) -> 0x70a08231
// allowance(address,address) -> 0xdd62ed3e
```

### ICP API

```rust
use walletd_icp::{IcpWallet, Principal, Agent, HDNetworkType};

// Create wallet from principal
let principal = Principal::anonymous();
let wallet = IcpWallet::from_principal(principal, HDNetworkType::MainNet);

// Access properties
let address = wallet.address();      // Account ID (hex string)
let principal = wallet.principal();

// Convert principal to account ID
let account_id = IcpWallet::principal_to_account_id(&principal);

// Create transaction
let tx = wallet.create_transaction(to_principal, amount, Some(memo))?;

// Transfer (requires agent)
let agent = Agent::builder()
    .with_url("https://ic0.app")
    .build()?;
let block_height = wallet.transfer(&agent, to, amount, memo).await?;

// Get balance
let balance = wallet.get_balance(&agent).await?;
```

### Hedera API

```rust
use walletd_hedera::{RealHederaWallet, HederaClient};
use hedera::Hbar;

// Create wallet
let mut wallet = RealHederaWallet::new("testnet")?;

// Keys are generated automatically
println!("Public Key: {}", wallet.public_key);

// Initialize with existing account
wallet.account_id = Some("0.0.12345".to_string());
wallet.init_with_existing_account().await?;

// Create new testnet account
let account_id = wallet.create_testnet_account(Hbar::new(10)).await?;

// Get balance
let balance = wallet.get_balance().await?;  // Returns HBAR as f64

// Send HBAR
let tx_id = wallet.send_hbar("0.0.67890", 5.0).await?;
```

### Monero API

```rust
use walletd_monero::{
    MoneroWallet, MoneroAmount, MoneroPrivateKeys, MoneroPublicKeys,
    Address, AddressType, SubaddressIndex, Network,
};

// Create wallet from HD key
let wallet = MoneroWallet::from_hd_key(&hd_key, AddressType::Standard)?;

// Access properties
let address = wallet.public_address();
let network = wallet.network();
let private_keys = wallet.private_keys();

// Create keys from seed
let private_keys = MoneroPrivateKeys::from_seed(&seed_bytes)?;
let public_keys = MoneroPublicKeys::from_private_keys(&private_keys);

// Create address
let address = Address::new(&Network::Mainnet, &public_keys, &AddressType::Standard)?;

// Parse address from string
let address = Address::from_str("49zf2PF7...")?;

// Create subaddress
let index = SubaddressIndex::new(0, 1);  // Account 0, Subaddress 1
let subaddress_keys = SubaddressKeys::new(&private_keys, &index)?;

// Monero amounts (12 decimal places)
let amount = MoneroAmount::from_xmr(1.5);
let amount = MoneroAmount::from_piconero(1_500_000_000_000);

println!("{} XMR", amount.as_XMR());
println!("{} piconero", amount.as_piconero());

// Arithmetic
let total = amount_a + amount_b;
let diff = amount_a - amount_b;
let scaled = amount * 1.5;
```

---

## Error Handling

All chain implementations use the `WalletError` type for consistent error handling:

```rust
pub enum WalletError {
    InvalidAddress(String),
    InsufficientBalance { have: Amount, need: Amount },
    TransactionFailed(String),
    NetworkError(String),
    KeyError(String),
    NotSynced,
    NotSupported(String),
    Other(String),
}
```

Example error handling:

```rust
match wallet.transfer(to, amount).await {
    Ok(tx_hash) => println!("Success: {}", tx_hash),
    Err(WalletError::InsufficientBalance { have, need }) => {
        println!("Need {} but only have {}", need, have);
    }
    Err(WalletError::NetworkError(msg)) => {
        println!("Network issue: {}", msg);
    }
    Err(e) => println!("Error: {}", e),
}
```

---

## Security Best Practices

### 1. Secure Key Storage

```rust
use zeroize::Zeroize;

// Private keys are zeroized on drop
let mut private_key = wallet.export_private()?;
// ... use key ...
private_key.zeroize();  // Explicitly clear memory
```

### 2. Never Log Sensitive Data

```rust
// BAD: logs private key
log::info!("Wallet: {:?}", wallet);

// GOOD: only log public info
log::info!("Address: {}", wallet.address());
```

### 3. Validate Addresses

```rust
// Always validate addresses before sending
if !is_valid_ethereum_address(&to_address) {
    return Err(WalletError::InvalidAddress(to_address));
}
```

### 4. Use Testnets for Development

```rust
// Development
let wallet = EthereumWallet::new(Network::Sepolia);

// Production
let wallet = EthereumWallet::new(Network::Mainnet);
```

### 5. Handle Network Errors Gracefully

```rust
let balance = match wallet.get_balance().await {
    Ok(b) => b,
    Err(WalletError::NetworkError(_)) => {
        // Retry or use cached value
        cached_balance
    }
    Err(e) => return Err(e),
};
```

---

## Testing

Run the test suite:

```bash
# All tests
cargo test --workspace

# Specific chain
cargo test -p walletd_ethereum
cargo test -p walletd_bitcoin

# Integration tests (require network)
cargo test --workspace -- --ignored
```

---

## License

MIT OR Apache-2.0

---

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
