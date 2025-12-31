//! # WalletD Avalanche
//!
//! Avalanche (AVAX) C-Chain wallet support for the WalletD SDK.
//!
//! ## Features
//!
//! - Create and manage Avalanche C-Chain wallets
//! - Send and receive AVAX
//! - EIP-1559 transaction support
//! - Mainnet and Fuji testnet support
//!
//! ## Example
//!
//! ```rust,no_run
//! use walletd_avalanche::AvalancheWallet;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new mainnet wallet
//!     let mut wallet = AvalancheWallet::mainnet().unwrap();
//!     
//!     // Connect to RPC
//!     wallet.connect_mainnet().unwrap();
//!     
//!     // Get address
//!     println!("Address: {}", wallet.address());
//!     
//!     // Get balance
//!     let balance = wallet.get_balance_avax().await.unwrap();
//!     println!("Balance: {} AVAX", balance);
//! }
//! ```
//!
//! ## Chain Support
//!
//! This crate supports the Avalanche C-Chain (Contract Chain), which is
//! EVM-compatible. The P-Chain (Platform) and X-Chain (Exchange) are not
//! currently supported.

pub mod config;
pub mod error;
pub mod rpc;
pub mod transaction;
pub mod wallet;

pub use config::{
    NetworkConfig, ChainType,
    AVALANCHE_MAINNET, AVALANCHE_FUJI,
    AVALANCHE_MAINNET_CHAIN_ID, AVALANCHE_FUJI_CHAIN_ID
};
pub use error::AvalancheError;
pub use rpc::AvalancheRpcClient;
pub use transaction::AvalancheTransaction;
pub use wallet::AvalancheWallet;

// Re-export alloy primitives for convenience
pub use alloy::primitives::{Address, U256};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avalanche_mainnet_chain_id() {
        assert_eq!(AVALANCHE_MAINNET_CHAIN_ID, 43114);
    }

    #[test]
    fn test_avalanche_fuji_chain_id() {
        assert_eq!(AVALANCHE_FUJI_CHAIN_ID, 43113);
    }

    #[test]
    fn test_create_wallet() {
        let wallet = AvalancheWallet::mainnet();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_create_transaction() {
        let tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64),
            43114,
        );
        assert_eq!(tx.chain_id, 43114);
    }

    #[test]
    fn test_chain_type() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_type, ChainType::CChain);
    }
}
