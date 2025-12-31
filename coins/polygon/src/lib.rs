//! # WalletD Polygon
//!
//! Polygon (POL/MATIC) wallet support for the WalletD SDK.
//!
//! ## Features
//!
//! - Create and manage Polygon wallets
//! - Send and receive POL (formerly MATIC)
//! - EIP-1559 transaction support
//! - Mainnet and Amoy testnet support
//!
//! ## Example
//!
//! ```rust,no_run
//! use walletd_polygon::PolygonWallet;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a new mainnet wallet
//!     let mut wallet = PolygonWallet::mainnet().unwrap();
//!     
//!     // Connect to RPC
//!     wallet.connect_mainnet().unwrap();
//!     
//!     // Get address
//!     println!("Address: {}", wallet.address());
//!     
//!     // Get balance
//!     let balance = wallet.get_balance_pol().await.unwrap();
//!     println!("Balance: {} POL", balance);
//! }
//! ```

pub mod config;
pub mod error;
pub mod rpc;
pub mod transaction;
pub mod wallet;

pub use config::{NetworkConfig, POLYGON_MAINNET, POLYGON_AMOY, POLYGON_MAINNET_CHAIN_ID, POLYGON_AMOY_CHAIN_ID};
pub use error::PolygonError;
pub use rpc::PolygonRpcClient;
pub use transaction::PolygonTransaction;
pub use wallet::PolygonWallet;

// Re-export alloy primitives for convenience
pub use alloy::primitives::{Address, U256};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_mainnet_chain_id() {
        assert_eq!(POLYGON_MAINNET_CHAIN_ID, 137);
    }

    #[test]
    fn test_polygon_amoy_chain_id() {
        assert_eq!(POLYGON_AMOY_CHAIN_ID, 80002);
    }

    #[test]
    fn test_create_wallet() {
        let wallet = PolygonWallet::mainnet();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_create_transaction() {
        let tx = PolygonTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64),
            137,
        );
        assert_eq!(tx.chain_id, 137);
    }
}
