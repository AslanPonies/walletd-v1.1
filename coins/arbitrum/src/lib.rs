//! # WalletD Arbitrum
//!
//! Arbitrum L2 blockchain support for the WalletD SDK.
//!
//! Arbitrum is an Optimistic Rollup Layer 2 solution for Ethereum with:
//! - Fast, cheap transactions
//! - Full EVM compatibility
//! - Native ETH for gas
//!
//! ## Example
//!
//! ```rust,ignore
//! use walletd_arbitrum::{ArbitrumWallet, NetworkConfig};
//!
//! // Create wallet from private key
//! let wallet = ArbitrumWallet::from_private_key(
//!     "0x...",
//!     NetworkConfig::mainnet().chain_id
//! )?;
//!
//! println!("Address: {}", wallet.address());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod config;
mod wallet;

pub use config::*;
pub use wallet::*;
