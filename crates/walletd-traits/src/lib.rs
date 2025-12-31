//! # WalletD Traits
//!
//! This crate provides unified traits for the WalletD multi-chain wallet SDK.
//! All chain-specific wallet implementations should implement these traits
//! to ensure a consistent API across different blockchains.
//!
//! ## Core Traits
//!
//! - [`Wallet`] - Basic wallet functionality (address, balance)
//! - [`Transferable`] - Send funds to another address
//! - [`Syncable`] - Sync wallet state with blockchain
//! - [`HDWallet`] - Hierarchical deterministic wallet support
//! - [`TokenWallet`] - Token/asset support (ERC-20, SPL, etc.)
//!
//! ## Example
//!
//! ```ignore
//! use walletd_traits::prelude::*;
//!
//! async fn check_balance<W: Wallet>(wallet: &W) -> Result<Amount, WalletError> {
//!     wallet.balance().await
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a blockchain amount with arbitrary precision.
///
/// This type wraps the smallest unit of a cryptocurrency (e.g., wei, satoshi, lamport)
/// and provides methods for conversion to human-readable units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount {
    /// The value in the smallest unit of the currency
    pub value: u128,
    /// Number of decimal places for the currency (e.g., 18 for ETH, 8 for BTC)
    pub decimals: u8,
}

impl Amount {
    /// Creates a new Amount from the smallest unit value
    pub fn from_smallest_unit(value: u128, decimals: u8) -> Self {
        Self { value, decimals }
    }

    /// Creates a new Amount from a human-readable value
    pub fn from_human(value: f64, decimals: u8) -> Self {
        let multiplier = 10u128.pow(decimals as u32);
        let smallest = (value * multiplier as f64) as u128;
        Self { value: smallest, decimals }
    }

    /// Returns the value in the smallest unit
    pub fn smallest_unit(&self) -> u128 {
        self.value
    }

    /// Returns the value in human-readable form
    pub fn human_readable(&self) -> f64 {
        let divisor = 10u128.pow(self.decimals as u32) as f64;
        self.value as f64 / divisor
    }

    /// Returns zero amount with the specified decimals
    pub fn zero(decimals: u8) -> Self {
        Self { value: 0, decimals }
    }

    /// Checks if the amount is zero
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.precision$}", self.human_readable(), precision = self.decimals as usize)
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self { value: 0, decimals: 18 }
    }
}

/// Represents the status of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction has been confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction status is unknown
    Unknown,
}

/// Represents a transaction hash/ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(pub String);

impl TxHash {
    /// Creates a new TxHash from a string
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Returns the hash as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TxHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TxHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Represents a blockchain network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    /// Network name (e.g., "mainnet", "testnet", "sepolia")
    pub name: String,
    /// Chain ID (for EVM chains)
    pub chain_id: Option<u64>,
    /// Whether this is a testnet
    pub is_testnet: bool,
}

impl Network {
    /// Creates a mainnet network
    pub fn mainnet(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chain_id: None,
            is_testnet: false,
        }
    }

    /// Creates a testnet network
    pub fn testnet(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            chain_id: None,
            is_testnet: true,
        }
    }

    /// Sets the chain ID
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }
}

/// Common wallet errors
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Insufficient balance for transaction
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance {
        /// Available balance
        have: Amount,
        /// Required balance
        need: Amount,
    },

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Network/RPC error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Key/signing error
    #[error("Key error: {0}")]
    KeyError(String),

    /// Wallet not synced
    #[error("Wallet not synced")]
    NotSynced,

    /// Feature not supported
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type for wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

/// Basic wallet trait - provides core wallet functionality
///
/// All wallet implementations must implement this trait to provide
/// basic functionality like getting the address and checking balance.
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Returns the primary address of the wallet
    fn address(&self) -> String;

    /// Returns the current balance of the wallet
    async fn balance(&self) -> WalletResult<Amount>;

    /// Returns the network this wallet is connected to
    fn network(&self) -> &Network;

    /// Returns the currency symbol (e.g., "ETH", "BTC", "SOL")
    fn currency_symbol(&self) -> &str;

    /// Returns the number of decimal places for the currency
    fn decimals(&self) -> u8;
}

/// Trait for wallets that can send transactions
#[async_trait]
pub trait Transferable: Wallet {
    /// Transfers funds to another address
    ///
    /// # Arguments
    /// * `to` - The recipient address
    /// * `amount` - The amount to send
    ///
    /// # Returns
    /// The transaction hash on success
    async fn transfer(&self, to: &str, amount: Amount) -> WalletResult<TxHash>;

    /// Estimates the fee for a transfer
    ///
    /// # Arguments
    /// * `to` - The recipient address  
    /// * `amount` - The amount to send
    ///
    /// # Returns
    /// The estimated fee
    async fn estimate_fee(&self, to: &str, amount: Amount) -> WalletResult<Amount>;
}

/// Trait for wallets that can sync with the blockchain
#[async_trait]
pub trait Syncable: Wallet {
    /// Syncs the wallet state with the blockchain
    async fn sync(&mut self) -> WalletResult<()>;

    /// Returns true if the wallet is synced
    fn is_synced(&self) -> bool;

    /// Returns the last sync timestamp (Unix epoch seconds)
    fn last_synced(&self) -> Option<u64>;
}

/// Trait for HD (Hierarchical Deterministic) wallets
pub trait HDWallet: Wallet {
    /// Returns the derivation path used by this wallet
    fn derivation_path(&self) -> &str;

    /// Derives a new address at the given index
    fn derive_address(&self, index: u32) -> WalletResult<String>;

    /// Returns all derived addresses
    fn addresses(&self) -> Vec<String>;
}

/// Trait for wallets that support tokens (ERC-20, SPL, etc.)
#[async_trait]
pub trait TokenWallet: Wallet {
    /// Token information
    type TokenInfo: Send + Sync;

    /// Returns the balance of a specific token
    async fn token_balance(&self, token_address: &str) -> WalletResult<Amount>;

    /// Transfers tokens to another address
    async fn transfer_token(
        &self,
        token_address: &str,
        to: &str,
        amount: Amount,
    ) -> WalletResult<TxHash>;

    /// Returns information about a token
    async fn token_info(&self, token_address: &str) -> WalletResult<Self::TokenInfo>;
}

/// Trait for wallets that support message signing
#[async_trait]
pub trait Signable: Wallet {
    /// Signs a message with the wallet's private key
    async fn sign_message(&self, message: &[u8]) -> WalletResult<Vec<u8>>;

    /// Verifies a signed message
    async fn verify_message(&self, message: &[u8], signature: &[u8], address: &str) -> WalletResult<bool>;
}

/// Trait for wallets that can export/import
pub trait Exportable: Wallet {
    /// Exports the wallet's public information (safe to share)
    fn export_public(&self) -> WalletResult<String>;

    /// Exports the wallet's private key (DANGER: keep secure!)
    fn export_private(&self) -> WalletResult<String>;
}

/// Transaction builder for constructing complex transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder {
    /// Recipient address
    pub to: Option<String>,
    /// Amount to send
    pub amount: Option<Amount>,
    /// Transaction data/memo
    pub data: Option<Vec<u8>>,
    /// Gas limit (for EVM chains)
    pub gas_limit: Option<u64>,
    /// Gas price (for EVM chains)
    pub gas_price: Option<Amount>,
    /// Nonce override
    pub nonce: Option<u64>,
}

impl TransactionBuilder {
    /// Creates a new transaction builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the recipient address
    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to = Some(address.into());
        self
    }

    /// Sets the amount to send
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Sets the transaction data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the gas limit
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Sets the gas price
    pub fn gas_price(mut self, price: Amount) -> Self {
        self.gas_price = Some(price);
        self
    }

    /// Sets the nonce
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        Amount, HDWallet, Network, Signable, Syncable, TokenWallet, Transferable,
        TransactionBuilder, TransactionStatus, TxHash, Wallet, WalletError, WalletResult,
        Exportable,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Amount Tests
    // ============================================================================

    #[test]
    fn test_amount_conversion() {
        // 1 ETH = 1e18 wei
        let amount = Amount::from_human(1.0, 18);
        assert_eq!(amount.smallest_unit(), 1_000_000_000_000_000_000);
        assert!((amount.human_readable() - 1.0).abs() < 0.0001);

        // 1 BTC = 1e8 satoshi
        let btc = Amount::from_human(1.0, 8);
        assert_eq!(btc.smallest_unit(), 100_000_000);
    }

    #[test]
    fn test_amount_zero() {
        let zero = Amount::zero(18);
        assert!(zero.is_zero());
        assert_eq!(zero.smallest_unit(), 0);
    }

    #[test]
    fn test_amount_from_smallest_unit() {
        let amount = Amount::from_smallest_unit(1_000_000, 6);
        assert_eq!(amount.smallest_unit(), 1_000_000);
        assert_eq!(amount.decimals, 6);
        assert!((amount.human_readable() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_amount_small_values() {
        // 0.000001 ETH
        let amount = Amount::from_human(0.000001, 18);
        assert_eq!(amount.smallest_unit(), 1_000_000_000_000);
    }

    #[test]
    fn test_amount_large_values() {
        // 1 million ETH
        let amount = Amount::from_human(1_000_000.0, 18);
        assert!(amount.smallest_unit() > 0);
    }

    #[test]
    fn test_amount_display() {
        let amount = Amount::from_human(1.5, 18);
        let display = format!("{}", amount);
        assert!(display.contains("1.5"));
    }

    #[test]
    fn test_amount_default() {
        let default = Amount::default();
        assert_eq!(default.value, 0);
        assert_eq!(default.decimals, 18);
    }

    #[test]
    fn test_amount_comparison() {
        let a = Amount::from_smallest_unit(100, 18);
        let b = Amount::from_smallest_unit(200, 18);
        
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Amount::from_smallest_unit(100, 18));
    }

    #[test]
    fn test_amount_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(Amount::from_smallest_unit(100, 18));
        set.insert(Amount::from_smallest_unit(100, 18)); // duplicate
        
        assert_eq!(set.len(), 1);
    }

    // ============================================================================
    // TxHash Tests
    // ============================================================================

    #[test]
    fn test_tx_hash() {
        let hash = TxHash::new("0x1234567890abcdef");
        assert_eq!(hash.as_str(), "0x1234567890abcdef");
        assert_eq!(format!("{}", hash), "0x1234567890abcdef");
    }

    #[test]
    fn test_tx_hash_from_string() {
        let hash: TxHash = "0xabcd".to_string().into();
        assert_eq!(hash.as_str(), "0xabcd");
    }

    #[test]
    fn test_tx_hash_from_str() {
        let hash: TxHash = "0xefgh".into();
        assert_eq!(hash.as_str(), "0xefgh");
    }

    #[test]
    fn test_tx_hash_equality() {
        let hash1 = TxHash::new("0x123");
        let hash2 = TxHash::new("0x123");
        let hash3 = TxHash::new("0x456");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_tx_hash_hash() {
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        map.insert(TxHash::new("0x123"), "tx1");
        
        assert_eq!(map.get(&TxHash::new("0x123")), Some(&"tx1"));
    }

    // ============================================================================
    // Network Tests
    // ============================================================================

    #[test]
    fn test_network() {
        let mainnet = Network::mainnet("ethereum").with_chain_id(1);
        assert!(!mainnet.is_testnet);
        assert_eq!(mainnet.chain_id, Some(1));

        let testnet = Network::testnet("sepolia").with_chain_id(11155111);
        assert!(testnet.is_testnet);
    }

    #[test]
    fn test_network_mainnet() {
        let network = Network::mainnet("Bitcoin");
        assert_eq!(network.name, "Bitcoin");
        assert!(!network.is_testnet);
        assert!(network.chain_id.is_none());
    }

    #[test]
    fn test_network_testnet() {
        let network = Network::testnet("Goerli");
        assert_eq!(network.name, "Goerli");
        assert!(network.is_testnet);
    }

    #[test]
    fn test_network_with_chain_id() {
        let network = Network::mainnet("Polygon")
            .with_chain_id(137);
        
        assert_eq!(network.chain_id, Some(137));
    }

    // ============================================================================
    // TransactionStatus Tests
    // ============================================================================

    #[test]
    fn test_transaction_status_variants() {
        let pending = TransactionStatus::Pending;
        let confirmed = TransactionStatus::Confirmed;
        let failed = TransactionStatus::Failed;
        let unknown = TransactionStatus::Unknown;
        
        assert_ne!(pending, confirmed);
        assert_ne!(confirmed, failed);
        assert_ne!(failed, unknown);
    }

    #[test]
    fn test_transaction_status_equality() {
        assert_eq!(TransactionStatus::Pending, TransactionStatus::Pending);
        assert_eq!(TransactionStatus::Confirmed, TransactionStatus::Confirmed);
    }

    // ============================================================================
    // WalletError Tests
    // ============================================================================

    #[test]
    fn test_wallet_error_display() {
        let err = WalletError::InvalidAddress("bad address".to_string());
        assert!(err.to_string().contains("Invalid address"));
        assert!(err.to_string().contains("bad address"));
    }

    #[test]
    fn test_wallet_error_insufficient_balance() {
        let err = WalletError::InsufficientBalance {
            have: Amount::from_human(1.0, 18),
            need: Amount::from_human(2.0, 18),
        };
        let msg = err.to_string();
        assert!(msg.contains("Insufficient balance"));
    }

    #[test]
    fn test_wallet_error_transaction_failed() {
        let err = WalletError::TransactionFailed("gas too low".to_string());
        assert!(err.to_string().contains("Transaction failed"));
    }

    #[test]
    fn test_wallet_error_network() {
        let err = WalletError::NetworkError("connection refused".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_wallet_error_key() {
        let err = WalletError::KeyError("invalid private key".to_string());
        assert!(err.to_string().contains("Key error"));
    }

    #[test]
    fn test_wallet_error_not_synced() {
        let err = WalletError::NotSynced;
        assert!(err.to_string().contains("not synced"));
    }

    #[test]
    fn test_wallet_error_not_supported() {
        let err = WalletError::NotSupported("multi-sig".to_string());
        assert!(err.to_string().contains("Not supported"));
    }

    #[test]
    fn test_wallet_error_other() {
        let err = WalletError::Other("something went wrong".to_string());
        assert!(err.to_string().contains("something went wrong"));
    }

    // ============================================================================
    // TransactionBuilder Tests
    // ============================================================================

    #[test]
    fn test_transaction_builder() {
        let tx = TransactionBuilder::new()
            .to("0x1234...")
            .amount(Amount::from_human(1.0, 18))
            .gas_limit(21000)
            .nonce(5);

        assert_eq!(tx.to, Some("0x1234...".to_string()));
        assert_eq!(tx.gas_limit, Some(21000));
        assert_eq!(tx.nonce, Some(5));
    }

    #[test]
    fn test_transaction_builder_default() {
        let tx = TransactionBuilder::default();
        
        assert!(tx.to.is_none());
        assert!(tx.amount.is_none());
        assert!(tx.data.is_none());
        assert!(tx.gas_limit.is_none());
        assert!(tx.gas_price.is_none());
        assert!(tx.nonce.is_none());
    }

    #[test]
    fn test_transaction_builder_with_data() {
        let data = vec![0x01, 0x02, 0x03];
        let tx = TransactionBuilder::new()
            .data(data.clone());
        
        assert_eq!(tx.data, Some(data));
    }

    #[test]
    fn test_transaction_builder_with_gas_price() {
        let gas_price = Amount::from_smallest_unit(20_000_000_000, 18); // 20 gwei
        let tx = TransactionBuilder::new()
            .gas_price(gas_price);
        
        assert!(tx.gas_price.is_some());
    }

    #[test]
    fn test_transaction_builder_chaining() {
        let tx = TransactionBuilder::new()
            .to("0x123")
            .amount(Amount::from_human(1.0, 18))
            .gas_limit(21000)
            .gas_price(Amount::from_smallest_unit(20_000_000_000, 18))
            .nonce(0)
            .data(vec![]);
        
        assert!(tx.to.is_some());
        assert!(tx.amount.is_some());
        assert!(tx.gas_limit.is_some());
        assert!(tx.gas_price.is_some());
        assert!(tx.nonce.is_some());
        assert!(tx.data.is_some());
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_amount_serialization() {
        let amount = Amount::from_human(1.5, 18);
        let json = serde_json::to_string(&amount).unwrap();
        let deserialized: Amount = serde_json::from_str(&json).unwrap();
        
        assert_eq!(amount, deserialized);
    }

    #[test]
    fn test_tx_hash_serialization() {
        let hash = TxHash::new("0x1234");
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: TxHash = serde_json::from_str(&json).unwrap();
        
        assert_eq!(hash, deserialized);
    }

    #[test]
    fn test_network_serialization() {
        let network = Network::mainnet("ethereum").with_chain_id(1);
        let json = serde_json::to_string(&network).unwrap();
        let deserialized: Network = serde_json::from_str(&json).unwrap();
        
        assert_eq!(network, deserialized);
    }

    #[test]
    fn test_transaction_status_serialization() {
        let status = TransactionStatus::Confirmed;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: TransactionStatus = serde_json::from_str(&json).unwrap();
        
        assert_eq!(status, deserialized);
    }
}
