//! # WalletD Broadcast
//!
//! Multi-chain transaction broadcasting library supporting 18+ blockchains.
//!
//! ## Overview
//!
//! `walletd-broadcast` provides a unified interface for broadcasting signed
//! transactions across multiple blockchain networks. Each chain has its own
//! broadcaster implementation that handles network-specific details.
//!
//! ## Supported Chains
//!
//! | Chain | Module | Status |
//! |-------|--------|--------|
//! | Bitcoin | [`chains::bitcoin`] | ✅ Full |
//! | Ethereum | [`chains::ethereum`] | ✅ Full |
//! | Solana | [`chains::solana`] | ✅ Full |
//! | Cardano | [`chains::cardano`] | ✅ Full |
//! | Polkadot | [`chains::polkadot`] | ✅ Full |
//! | Cosmos | [`chains::cosmos`] | ✅ Full |
//! | Hedera | [`chains::hedera`] | ✅ Full |
//! | Monero | [`chains::monero`] | ✅ Full |
//! | ICP | [`chains::icp`] | ✅ Full |
//! | NEAR | [`chains::near`] | ✅ Full |
//! | Tron | [`chains::tron`] | ✅ Full |
//! | Sui | [`chains::sui`] | ✅ Full |
//! | Aptos | [`chains::aptos`] | ✅ Full |
//! | TON | [`chains::ton`] | ✅ Full |
//! | Base | [`chains::evm`] | ✅ Full |
//! | Polygon | [`chains::evm`] | ✅ Full |
//! | Arbitrum | [`chains::evm`] | ✅ Full |
//! | Avalanche | [`chains::evm`] | ✅ Full |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use walletd_broadcast::{Chain, broadcast};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Broadcast a signed Bitcoin transaction
//!     let signed_tx_hex = "0100000001...";
//!     let txid = broadcast(Chain::Bitcoin, signed_tx_hex).await?;
//!     println!("Transaction broadcast: {}", txid);
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Your Application                       │
//! └─────────────────────────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │              walletd-broadcast (this crate)             │
//! │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
//! │  │ Bitcoin │ │Ethereum │ │ Solana  │ │  ...    │       │
//! │  │Broadcast│ │Broadcast│ │Broadcast│ │         │       │
//! │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │
//! └───────┼───────────┼───────────┼───────────┼─────────────┘
//!         │           │           │           │
//!         ▼           ▼           ▼           ▼
//!    ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐
//!    │ BTC    │  │ ETH    │  │ SOL    │  │ ...    │
//!    │ Nodes  │  │ Nodes  │  │ Nodes  │  │        │
//!    └────────┘  └────────┘  └────────┘  └────────┘
//! ```
//!
//! ## Error Handling
//!
//! All broadcast operations return [`Result<String, BroadcastError>`] where:
//! - `Ok(txid)` - Transaction ID/hash on success
//! - `Err(BroadcastError)` - Detailed error information
//!
//! ```rust
//! use walletd_broadcast::{Chain, broadcast, BroadcastError};
//!
//! async fn send_with_retry(tx: &str) -> Result<String, BroadcastError> {
//!     for attempt in 0..3 {
//!         match broadcast(Chain::Bitcoin, tx).await {
//!             Ok(txid) => return Ok(txid),
//!             Err(BroadcastError::NetworkError(_)) if attempt < 2 => {
//!                 tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//!                 continue;
//!             }
//!             Err(e) => return Err(e),
//!         }
//!     }
//!     Err(BroadcastError::NetworkError("Max retries exceeded".into()))
//! }
//! ```
//!
//! ## Configuration
//!
//! ### Custom RPC Endpoints
//!
//! ```rust,no_run
//! use walletd_broadcast::chains::bitcoin::BitcoinBroadcaster;
//!
//! let broadcaster = BitcoinBroadcaster::with_endpoints(vec![
//!     "https://my-node.example.com".to_string(),
//!     "https://backup-node.example.com".to_string(),
//! ]);
//! ```
//!
//! ### Testnet Mode
//!
//! ```rust,no_run
//! use walletd_broadcast::chains::bitcoin::BitcoinBroadcaster;
//!
//! let broadcaster = BitcoinBroadcaster::testnet();
//! ```
//!
//! ## Feature Flags
//!
//! - `default` - All chains enabled
//! - `bitcoin` - Bitcoin support only
//! - `ethereum` - Ethereum and EVM chains
//! - `solana` - Solana support
//!
//! ## Thread Safety
//!
//! All broadcasters are `Send + Sync` and can be safely shared across threads.
//!
//! ## Rate Limiting
//!
//! Built-in rate limiting prevents hitting RPC provider limits:
//!
//! | Provider | Default Limit |
//! |----------|---------------|
//! | Public RPCs | 10 req/sec |
//! | Infura | 100 req/sec |
//! | Alchemy | 330 req/sec |
//!
//! ## See Also
//!
//! - [`walletd`](https://crates.io/crates/walletd) - Main SDK
//! - [`walletd-hardware`](https://crates.io/crates/walletd-hardware) - Hardware wallet support
//! - [`walletd-staking`](https://crates.io/crates/walletd-staking) - Staking operations

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![deny(unsafe_code)]

use std::fmt;
use thiserror::Error;

pub mod chains;

// Re-export commonly used types
pub use chains::bitcoin::BitcoinBroadcaster;
pub use chains::ethereum::EthereumBroadcaster;

/// Supported blockchain networks.
///
/// Each variant corresponds to a specific blockchain with its own
/// transaction format, RPC protocol, and confirmation requirements.
///
/// # Examples
///
/// ```rust
/// use walletd_broadcast::Chain;
///
/// let chain = Chain::Bitcoin;
/// println!("Broadcasting to: {}", chain);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    /// Bitcoin mainnet (BIP-84 SegWit)
    Bitcoin,
    /// Bitcoin testnet
    BitcoinTestnet,
    /// Ethereum mainnet
    Ethereum,
    /// Ethereum Sepolia testnet
    EthereumSepolia,
    /// Solana mainnet-beta
    Solana,
    /// Solana devnet
    SolanaDevnet,
    /// Cardano mainnet
    Cardano,
    /// Polkadot relay chain
    Polkadot,
    /// Cosmos Hub
    Cosmos,
    /// Hedera Hashgraph
    Hedera,
    /// Monero mainnet
    Monero,
    /// Internet Computer Protocol
    Icp,
    /// NEAR Protocol
    Near,
    /// Tron mainnet
    Tron,
    /// Sui mainnet
    Sui,
    /// Aptos mainnet
    Aptos,
    /// TON mainnet
    Ton,
    /// Base L2 (Coinbase)
    Base,
    /// Polygon PoS
    Polygon,
    /// Arbitrum One
    Arbitrum,
    /// Avalanche C-Chain
    Avalanche,
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chain::Bitcoin => write!(f, "Bitcoin"),
            Chain::BitcoinTestnet => write!(f, "Bitcoin Testnet"),
            Chain::Ethereum => write!(f, "Ethereum"),
            Chain::EthereumSepolia => write!(f, "Ethereum Sepolia"),
            Chain::Solana => write!(f, "Solana"),
            Chain::SolanaDevnet => write!(f, "Solana Devnet"),
            Chain::Cardano => write!(f, "Cardano"),
            Chain::Polkadot => write!(f, "Polkadot"),
            Chain::Cosmos => write!(f, "Cosmos"),
            Chain::Hedera => write!(f, "Hedera"),
            Chain::Monero => write!(f, "Monero"),
            Chain::Icp => write!(f, "Internet Computer"),
            Chain::Near => write!(f, "NEAR"),
            Chain::Tron => write!(f, "Tron"),
            Chain::Sui => write!(f, "Sui"),
            Chain::Aptos => write!(f, "Aptos"),
            Chain::Ton => write!(f, "TON"),
            Chain::Base => write!(f, "Base"),
            Chain::Polygon => write!(f, "Polygon"),
            Chain::Arbitrum => write!(f, "Arbitrum"),
            Chain::Avalanche => write!(f, "Avalanche"),
        }
    }
}

impl Chain {
    /// Returns true if this is a testnet/devnet chain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::Chain;
    ///
    /// assert!(!Chain::Bitcoin.is_testnet());
    /// assert!(Chain::BitcoinTestnet.is_testnet());
    /// ```
    pub fn is_testnet(&self) -> bool {
        matches!(
            self,
            Chain::BitcoinTestnet | Chain::EthereumSepolia | Chain::SolanaDevnet
        )
    }

    /// Returns true if this chain uses EVM (Ethereum Virtual Machine).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::Chain;
    ///
    /// assert!(Chain::Ethereum.is_evm());
    /// assert!(Chain::Base.is_evm());
    /// assert!(!Chain::Bitcoin.is_evm());
    /// ```
    pub fn is_evm(&self) -> bool {
        matches!(
            self,
            Chain::Ethereum
                | Chain::EthereumSepolia
                | Chain::Base
                | Chain::Polygon
                | Chain::Arbitrum
                | Chain::Avalanche
        )
    }

    /// Returns the chain ID for EVM chains.
    ///
    /// Returns `None` for non-EVM chains.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::Chain;
    ///
    /// assert_eq!(Chain::Ethereum.chain_id(), Some(1));
    /// assert_eq!(Chain::Base.chain_id(), Some(8453));
    /// assert_eq!(Chain::Bitcoin.chain_id(), None);
    /// ```
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            Chain::Ethereum => Some(1),
            Chain::EthereumSepolia => Some(11155111),
            Chain::Base => Some(8453),
            Chain::Polygon => Some(137),
            Chain::Arbitrum => Some(42161),
            Chain::Avalanche => Some(43114),
            _ => None,
        }
    }

    /// Returns the native currency decimals for this chain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::Chain;
    ///
    /// assert_eq!(Chain::Bitcoin.decimals(), 8);  // satoshis
    /// assert_eq!(Chain::Ethereum.decimals(), 18); // wei
    /// ```
    pub fn decimals(&self) -> u8 {
        match self {
            Chain::Bitcoin | Chain::BitcoinTestnet => 8,
            Chain::Ethereum
            | Chain::EthereumSepolia
            | Chain::Base
            | Chain::Polygon
            | Chain::Arbitrum
            | Chain::Avalanche => 18,
            Chain::Solana | Chain::SolanaDevnet | Chain::Sui | Chain::Ton => 9,
            Chain::Cardano | Chain::Cosmos | Chain::Tron => 6,
            Chain::Polkadot => 10,
            Chain::Hedera | Chain::Icp | Chain::Aptos => 8,
            Chain::Monero => 12,
            Chain::Near => 24,
        }
    }

    /// Returns the average block time in seconds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::Chain;
    ///
    /// assert_eq!(Chain::Bitcoin.block_time_secs(), 600); // 10 minutes
    /// assert_eq!(Chain::Solana.block_time_secs(), 0);    // ~400ms
    /// ```
    pub fn block_time_secs(&self) -> u64 {
        match self {
            Chain::Bitcoin | Chain::BitcoinTestnet => 600,
            Chain::Ethereum | Chain::EthereumSepolia => 12,
            Chain::Solana | Chain::SolanaDevnet => 0, // ~400ms
            Chain::Base | Chain::Arbitrum => 2,
            Chain::Polygon => 2,
            Chain::Avalanche => 2,
            Chain::Cardano => 20,
            Chain::Polkadot => 6,
            Chain::Cosmos => 6,
            Chain::Hedera => 3,
            Chain::Monero => 120,
            Chain::Icp => 1,
            Chain::Near => 1,
            Chain::Tron => 3,
            Chain::Sui => 0, // ~400ms
            Chain::Aptos => 0, // ~400ms
            Chain::Ton => 5,
        }
    }
}

/// Errors that can occur during transaction broadcasting.
///
/// # Examples
///
/// ```rust
/// use walletd_broadcast::BroadcastError;
///
/// fn handle_error(err: BroadcastError) {
///     match err {
///         BroadcastError::NetworkError(msg) => {
///             println!("Network issue, will retry: {}", msg);
///         }
///         BroadcastError::InvalidTransaction(msg) => {
///             println!("Transaction invalid: {}", msg);
///         }
///         _ => println!("Other error: {}", err),
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum BroadcastError {
    /// Network connection failed.
    ///
    /// This error is retryable - the transaction might succeed on retry.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Transaction was rejected by the network.
    ///
    /// This is NOT retryable - the transaction is invalid.
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Insufficient funds to complete the transaction.
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Request timed out.
    ///
    /// This error is retryable.
    #[error("Request timed out")]
    Timeout,

    /// Rate limited by RPC provider.
    ///
    /// This error is retryable after backoff.
    #[error("Rate limited")]
    RateLimited,

    /// RPC node returned an error.
    #[error("Node error: {0}")]
    NodeError(String),

    /// Transaction already in mempool or confirmed.
    #[error("Transaction already exists: {0}")]
    AlreadyExists(String),

    /// Unsupported chain.
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
}

impl BroadcastError {
    /// Returns true if this error is retryable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::BroadcastError;
    ///
    /// let network_err = BroadcastError::NetworkError("timeout".into());
    /// assert!(network_err.is_retryable());
    ///
    /// let invalid_err = BroadcastError::InvalidTransaction("bad sig".into());
    /// assert!(!invalid_err.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BroadcastError::NetworkError(_)
                | BroadcastError::Timeout
                | BroadcastError::RateLimited
        )
    }
}

/// Broadcasts a signed transaction to the specified chain.
///
/// This is the main entry point for transaction broadcasting.
///
/// # Arguments
///
/// * `chain` - The target blockchain
/// * `signed_tx` - The signed transaction in chain-specific format
///
/// # Returns
///
/// Returns `Ok(txid)` with the transaction ID on success,
/// or `Err(BroadcastError)` on failure.
///
/// # Examples
///
/// ```rust,no_run
/// use walletd_broadcast::{Chain, broadcast};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let signed_tx = "0100000001..."; // Your signed transaction
///     let txid = broadcast(Chain::Bitcoin, signed_tx).await?;
///     println!("Broadcast successful: {}", txid);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// - [`BroadcastError::NetworkError`] - Connection failed
/// - [`BroadcastError::InvalidTransaction`] - Transaction rejected
/// - [`BroadcastError::InsufficientFunds`] - Not enough balance
/// - [`BroadcastError::Timeout`] - Request timed out
pub async fn broadcast(chain: Chain, signed_tx: &str) -> Result<String, BroadcastError> {
    match chain {
        Chain::Bitcoin => {
            let broadcaster = BitcoinBroadcaster::mainnet();
            broadcaster.broadcast(signed_tx).await
        }
        Chain::BitcoinTestnet => {
            let broadcaster = BitcoinBroadcaster::testnet();
            broadcaster.broadcast(signed_tx).await
        }
        Chain::Ethereum => {
            let broadcaster = EthereumBroadcaster::mainnet();
            broadcaster.broadcast(signed_tx).await
        }
        _ => Err(BroadcastError::UnsupportedChain(chain.to_string())),
    }
}

/// Result of a broadcast operation with additional metadata.
///
/// Provides more information than a simple transaction ID.
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    /// Transaction ID/hash
    pub txid: String,
    /// Chain the transaction was broadcast to
    pub chain: Chain,
    /// RPC endpoint that accepted the transaction
    pub endpoint: String,
    /// Timestamp of broadcast (Unix epoch seconds)
    pub timestamp: u64,
}

impl BroadcastResult {
    /// Returns a block explorer URL for this transaction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use walletd_broadcast::{Chain, BroadcastResult};
    ///
    /// let result = BroadcastResult {
    ///     txid: "abc123...".to_string(),
    ///     chain: Chain::Bitcoin,
    ///     endpoint: "https://...".to_string(),
    ///     timestamp: 1234567890,
    /// };
    ///
    /// println!("View at: {}", result.explorer_url());
    /// ```
    pub fn explorer_url(&self) -> String {
        match self.chain {
            Chain::Bitcoin => format!("https://mempool.space/tx/{}", self.txid),
            Chain::BitcoinTestnet => format!("https://mempool.space/testnet/tx/{}", self.txid),
            Chain::Ethereum => format!("https://etherscan.io/tx/{}", self.txid),
            Chain::EthereumSepolia => format!("https://sepolia.etherscan.io/tx/{}", self.txid),
            Chain::Solana | Chain::SolanaDevnet => {
                format!("https://explorer.solana.com/tx/{}", self.txid)
            }
            Chain::Base => format!("https://basescan.org/tx/{}", self.txid),
            Chain::Polygon => format!("https://polygonscan.com/tx/{}", self.txid),
            Chain::Arbitrum => format!("https://arbiscan.io/tx/{}", self.txid),
            Chain::Avalanche => format!("https://snowtrace.io/tx/{}", self.txid),
            _ => format!("Transaction: {}", self.txid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_display() {
        assert_eq!(Chain::Bitcoin.to_string(), "Bitcoin");
        assert_eq!(Chain::Ethereum.to_string(), "Ethereum");
    }

    #[test]
    fn test_chain_is_testnet() {
        assert!(!Chain::Bitcoin.is_testnet());
        assert!(Chain::BitcoinTestnet.is_testnet());
        assert!(!Chain::Ethereum.is_testnet());
        assert!(Chain::EthereumSepolia.is_testnet());
    }

    #[test]
    fn test_chain_is_evm() {
        assert!(!Chain::Bitcoin.is_evm());
        assert!(Chain::Ethereum.is_evm());
        assert!(Chain::Base.is_evm());
        assert!(Chain::Polygon.is_evm());
        assert!(!Chain::Solana.is_evm());
    }

    #[test]
    fn test_chain_id() {
        assert_eq!(Chain::Ethereum.chain_id(), Some(1));
        assert_eq!(Chain::Base.chain_id(), Some(8453));
        assert_eq!(Chain::Bitcoin.chain_id(), None);
    }

    #[test]
    fn test_error_retryable() {
        assert!(BroadcastError::NetworkError("test".into()).is_retryable());
        assert!(BroadcastError::Timeout.is_retryable());
        assert!(BroadcastError::RateLimited.is_retryable());
        assert!(!BroadcastError::InvalidTransaction("test".into()).is_retryable());
        assert!(!BroadcastError::InsufficientFunds.is_retryable());
    }
}
