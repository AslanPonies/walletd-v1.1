//! # WalletD Cardano
//!
//! Cardano (ADA) wallet support for the WalletD SDK.
//!
//! ## Features
//!
//! - Create and manage Cardano wallets
//! - Enterprise and Base address generation
//! - Ed25519 signing and verification
//! - Mainnet, Preview, and Preprod testnet support
//!
//! ## Example
//!
//! ```rust,no_run
//! use walletd_cardano::CardanoWallet;
//!
//! fn main() {
//!     // Create a new mainnet wallet
//!     let wallet = CardanoWallet::mainnet().unwrap();
//!     
//!     // Get address (bech32 encoded)
//!     println!("Address: {}", wallet.address());
//!     
//!     // Sign a message
//!     let signature = wallet.sign(b"Hello Cardano!");
//!     println!("Signature: {}", hex::encode(&signature));
//! }
//! ```
//!
//! ## Address Types
//!
//! Cardano supports several address types:
//! - **Enterprise**: Payment key only (no staking rewards)
//! - **Base**: Payment + staking key (can receive staking rewards)
//! - **Pointer**: Payment + stake pool pointer
//! - **Reward**: Staking rewards address
//!
//! This crate currently supports Enterprise addresses by default.
//!
//! ## Note on UTXO Model
//!
//! Cardano uses a UTXO (Unspent Transaction Output) model similar to Bitcoin,
//! rather than an account model like Ethereum. Transaction building requires
//! selecting UTXOs and calculating change outputs.

pub mod address;
pub mod config;
pub mod error;
pub mod wallet;

pub use address::CardanoAddress;
pub use config::{
    NetworkConfig, AddressType,
    CARDANO_MAINNET, CARDANO_TESTNET,
    MAINNET_NETWORK_ID, TESTNET_NETWORK_ID,
    LOVELACE_PER_ADA, MIN_UTXO_LOVELACE,
};
pub use error::CardanoError;
pub use wallet::CardanoWallet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_network_id() {
        assert_eq!(MAINNET_NETWORK_ID, 1);
    }

    #[test]
    fn test_testnet_network_id() {
        assert_eq!(TESTNET_NETWORK_ID, 0);
    }

    #[test]
    fn test_lovelace_per_ada() {
        assert_eq!(LOVELACE_PER_ADA, 1_000_000);
    }

    #[test]
    fn test_create_wallet() {
        let wallet = CardanoWallet::mainnet();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_create_address() {
        let pubkey = [1u8; 32];
        let addr = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID);
        assert!(addr.is_ok());
    }
}
