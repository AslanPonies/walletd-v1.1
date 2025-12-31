//! # WalletD Ethereum Library
//!
//! Provides a wallet implementation for Ethereum and blockchain-specific functionality.
//! 
//! This library uses the [alloy](https://github.com/alloy-rs/alloy) framework for Ethereum interactions.
//!
//! ## Quickstart Guide
//!
//! Use the [EthereumWallet] struct as a good starting point to access the functionalities for Ethereum that walletD provides.
//!
//! Each [EthereumWallet] is associated with one public address.
//!
//! ### Import from Seed
//!
//! Here's how you can import an Ethereum wallet based on a mnemonic. We will use the `mut` keyword to make the [ethereum wallet][EthereumWallet] mutable so that we can modify `ethereum_wallet` later.
//! ```
//! use walletd_ethereum::prelude::*;
//!
//! # fn ethereum() -> Result<(), walletd_ethereum::Error> {
//! let mnemonic_phrase = "joy tail arena mix other envelope diary achieve short nest true vocal";
//! let mnemonic = Mnemonic::parse(mnemonic_phrase).unwrap();
//! let mut ethereum_wallet = EthereumWallet::builder().mnemonic(mnemonic).build()?;
//! let public_address = ethereum_wallet.public_address();
//! println!("ethereum wallet public address: {}", public_address);
//! # Ok(())
//! # }
//! ```
//! We see that by default the Ethereum wallet uses the derivation path "m/44'/60'/0'/0/" corresponding to BIP44 for the purpose value and 60' corresponding to the coin type for Ethereum.
//!
//! ### Using EthClient to Access Blockchain Data
//! The [EthClient] can be used to access blockchain data such as details of a transaction given a tx hash, the current block number, or the current gas price.
//! All methods take an RPC URL string directly.
//! ```no_run
//! # use walletd_ethereum::prelude::*;
//! # async fn ethereum() -> Result<(), walletd_ethereum::Error> {
//! let tx_hash: B256 = "0xe4216d69bf935587b82243e68189de7ade0aa5b6f70dd0de8636b8d643431c0b".parse().unwrap();
//! let rpc_url = "https://eth.llamarpc.com";
//! let tx = EthClient::get_transaction_data_from_tx_hash(rpc_url, tx_hash).await?;
//! let block_number = EthClient::current_block_number(rpc_url).await?;
//! let gas_price = EthClient::gas_price(rpc_url).await?;
//! println!("transaction data: {:?}", tx);
//! Ok(())
//! # }
//! ```
//!
//! ### Balance of Ethereum Wallet on Blockchain
//! You can find the balance of any address on the blockchain.
//! ```no_run
//! # use walletd_ethereum::prelude::*;
//! # async fn ethereum() -> Result<(), walletd_ethereum::Error> {
//! let mnemonic_phrase = "mandate rude write gather vivid inform leg swift usual early bamboo element";
//! let address: Address = "0xFf7FD50BF684eb853787179cc9c784b55Ac68699".parse().unwrap();
//! let mnemonic = Mnemonic::parse(mnemonic_phrase).unwrap();
//! let ethereum_wallet = EthereumWallet::builder().mnemonic(mnemonic).build()?;
//! let rpc_url = "https://eth.llamarpc.com";
//! let balance = EthClient::balance(rpc_url, address).await?;
//! println!("ethereum wallet balance: {} ETH, ({} wei)", balance.eth(), balance.wei());
//! # Ok(())
//! # }
//! ```
//!
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;

mod ethclient;
pub use ethclient::EthClient;
mod ethereum_amount;
pub use ethereum_amount::EthereumAmount;
mod ethereum_wallet;
pub use ethereum_wallet::{EthereumWallet, EthereumWalletBuilder};
mod error;
pub use error::Error;
pub use alloy;
pub mod prelude;

// Unified traits implementation
mod traits_impl;
pub use traits_impl::ConnectedEthereumWallet;

/// Re-export walletd-traits for convenience
pub use walletd_traits;

/// Represents the format of an Ethereum address (checksummed or non-checksummed)
#[derive(Default, Debug, Clone, Copy)]
pub enum EthereumFormat {
    #[default]
    /// Checksummed is the checksummed format of an Ethereum address where the case of each letter is mixed using the checksum algorithm
    /// This is the default format for this enum
    Checksummed,
    /// NonChecksummed is the non-checksummed format of an Ethereum address where the letters are all lowercase
    NonChecksummed,
}

impl fmt::Display for EthereumFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EthereumFormat::Checksummed => write!(f, "Checksummed"),
            EthereumFormat::NonChecksummed => write!(f, "NonChecksummed"),
        }
    }
}
