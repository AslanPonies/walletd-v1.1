//! # WalletD Error
//!
//! This crate provides unified error types for the WalletD multi-chain wallet SDK.
//! It consolidates error handling across all supported blockchains into a consistent
//! error hierarchy.
//!
//! ## Error Categories
//!
//! - [`WalletdError`] - Top-level error type
//! - [`ChainError`] - Chain-specific errors
//! - [`CryptoError`] - Cryptographic operation errors
//! - [`NetworkError`] - Network/RPC errors
//! - [`ParseError`] - Parsing and validation errors
//!
//! ## Example
//!
//! ```
//! use walletd_error::{WalletdError, Result};
//!
//! fn validate_address(addr: &str) -> Result<()> {
//!     if addr.len() < 10 {
//!         return Err(WalletdError::InvalidAddress {
//!             address: addr.to_string(),
//!             reason: "Address too short".to_string(),
//!         });
//!     }
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// The main error type for WalletD operations.
///
/// This enum covers all possible errors that can occur during wallet operations
/// across all supported blockchains.
#[derive(Error, Debug)]
pub enum WalletdError {
    // ============ Address Errors ============
    /// Invalid address format or checksum
    #[error("Invalid address '{address}': {reason}")]
    InvalidAddress {
        /// The invalid address
        address: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Address not found or doesn't exist
    #[error("Address not found: {0}")]
    AddressNotFound(String),

    // ============ Balance/Amount Errors ============
    /// Insufficient balance for operation
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance {
        /// Available balance (in smallest unit)
        have: u128,
        /// Required balance (in smallest unit)
        need: u128,
    },

    /// Amount overflow during calculation
    #[error("Amount overflow: {0}")]
    AmountOverflow(String),

    /// Invalid amount format
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    // ============ Transaction Errors ============
    /// Transaction construction failed
    #[error("Failed to build transaction: {0}")]
    TransactionBuildError(String),

    /// Transaction signing failed
    #[error("Failed to sign transaction: {0}")]
    SigningError(String),

    /// Transaction broadcast failed
    #[error("Failed to broadcast transaction: {0}")]
    BroadcastError(String),

    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    /// Transaction failed/reverted
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Transaction timed out
    #[error("Transaction timed out waiting for confirmation")]
    TransactionTimeout,

    // ============ Key/Crypto Errors ============
    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid mnemonic phrase
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    /// Key derivation failed
    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),

    /// Missing private key for signing
    #[error("Missing private key")]
    MissingPrivateKey,

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    // ============ Network Errors ============
    /// RPC connection failed
    #[error("RPC connection failed: {url} - {reason}")]
    RpcConnectionError {
        /// RPC URL
        url: String,
        /// Error reason
        reason: String,
    },

    /// RPC request failed
    #[error("RPC request failed: {method} - {reason}")]
    RpcRequestError {
        /// RPC method name
        method: String,
        /// Error reason
        reason: String,
    },

    /// Network timeout
    #[error("Network timeout after {seconds}s")]
    NetworkTimeout {
        /// Timeout duration
        seconds: u64,
    },

    /// Rate limited by provider
    #[error("Rate limited by provider, retry after {retry_after_secs}s")]
    RateLimited {
        /// Suggested retry delay
        retry_after_secs: u64,
    },

    /// Invalid chain ID
    #[error("Invalid chain ID: expected {expected}, got {got}")]
    ChainIdMismatch {
        /// Expected chain ID
        expected: u64,
        /// Actual chain ID
        got: u64,
    },

    // ============ Contract Errors ============
    /// Contract call failed
    #[error("Contract call failed: {0}")]
    ContractError(String),

    /// Contract not found at address
    #[error("Contract not found at {0}")]
    ContractNotFound(String),

    /// ABI encoding/decoding error
    #[error("ABI error: {0}")]
    AbiError(String),

    // ============ Wallet State Errors ============
    /// Wallet not synced
    #[error("Wallet not synced with blockchain")]
    NotSynced,

    /// Wallet already exists
    #[error("Wallet already exists: {0}")]
    WalletExists(String),

    /// Wallet not found
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    /// Invalid wallet state
    #[error("Invalid wallet state: {0}")]
    InvalidState(String),

    // ============ Parsing Errors ============
    /// Hex decode error
    #[error("Hex decode error: {0}")]
    HexError(String),

    /// JSON parse error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Invalid format
    #[error("Invalid format: {0}")]
    FormatError(String),

    // ============ Chain-Specific Errors ============
    /// Bitcoin-specific error
    #[error("Bitcoin error: {0}")]
    BitcoinError(String),

    /// Ethereum-specific error
    #[error("Ethereum error: {0}")]
    EthereumError(String),

    /// Solana-specific error
    #[error("Solana error: {0}")]
    SolanaError(String),

    /// ICP-specific error
    #[error("ICP error: {0}")]
    IcpError(String),

    /// Monero-specific error
    #[error("Monero error: {0}")]
    MoneroError(String),

    /// Hedera-specific error
    #[error("Hedera error: {0}")]
    HederaError(String),

    /// Base L2-specific error
    #[error("Base error: {0}")]
    BaseError(String),

    // ============ Feature/Support Errors ============
    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    // ============ IO Errors ============
    /// File IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    // ============ Generic ============
    /// Unknown/other error
    #[error("{0}")]
    Other(String),

    /// Wrapped error from external source
    #[error("External error: {message}")]
    External {
        /// Error message
        message: String,
    },
}

/// Convenient Result type using WalletdError
pub type Result<T> = std::result::Result<T, WalletdError>;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Adds context to an error
    fn context(self, ctx: impl Into<String>) -> Result<T>;
    
    /// Adds context using a closure (lazy evaluation)
    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T>;
}

impl<T, E: std::error::Error> ErrorContext<T> for std::result::Result<T, E> {
    fn context(self, ctx: impl Into<String>) -> Result<T> {
        self.map_err(|e| WalletdError::External {
            message: format!("{}: {}", ctx.into(), e),
        })
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.map_err(|e| WalletdError::External {
            message: format!("{}: {}", f(), e),
        })
    }
}

impl<T> ErrorContext<T> for Option<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| WalletdError::Other(ctx.into()))
    }

    fn with_context<F: FnOnce() -> String>(self, f: F) -> Result<T> {
        self.ok_or_else(|| WalletdError::Other(f()))
    }
}

// ============ From implementations for common error types ============

impl From<std::io::Error> for WalletdError {
    fn from(err: std::io::Error) -> Self {
        WalletdError::IoError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for WalletdError {
    fn from(err: std::num::ParseIntError) -> Self {
        WalletdError::FormatError(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for WalletdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        WalletdError::FormatError(err.to_string())
    }
}

impl From<hex::FromHexError> for WalletdError {
    fn from(err: hex::FromHexError) -> Self {
        WalletdError::HexError(err.to_string())
    }
}

/// Error codes for programmatic error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ErrorCode {
    /// Unknown error
    Unknown = 0,
    /// Invalid address
    InvalidAddress = 1001,
    /// Address not found
    AddressNotFound = 1002,
    /// Insufficient balance
    InsufficientBalance = 2001,
    /// Amount overflow
    AmountOverflow = 2002,
    /// Transaction build error
    TransactionBuildError = 3001,
    /// Signing error
    SigningError = 3002,
    /// Broadcast error
    BroadcastError = 3003,
    /// Transaction not found
    TransactionNotFound = 3004,
    /// Transaction failed
    TransactionFailed = 3005,
    /// RPC connection error
    RpcConnectionError = 4001,
    /// RPC request error
    RpcRequestError = 4002,
    /// Network timeout
    NetworkTimeout = 4003,
    /// Rate limited
    RateLimited = 4004,
    /// Contract error
    ContractError = 5001,
    /// Not synced
    NotSynced = 6001,
    /// Not supported
    NotSupported = 9001,
}

impl WalletdError {
    /// Returns the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            WalletdError::InvalidAddress { .. } => ErrorCode::InvalidAddress,
            WalletdError::AddressNotFound(_) => ErrorCode::AddressNotFound,
            WalletdError::InsufficientBalance { .. } => ErrorCode::InsufficientBalance,
            WalletdError::AmountOverflow(_) => ErrorCode::AmountOverflow,
            WalletdError::TransactionBuildError(_) => ErrorCode::TransactionBuildError,
            WalletdError::SigningError(_) => ErrorCode::SigningError,
            WalletdError::BroadcastError(_) => ErrorCode::BroadcastError,
            WalletdError::TransactionNotFound(_) => ErrorCode::TransactionNotFound,
            WalletdError::TransactionFailed(_) => ErrorCode::TransactionFailed,
            WalletdError::RpcConnectionError { .. } => ErrorCode::RpcConnectionError,
            WalletdError::RpcRequestError { .. } => ErrorCode::RpcRequestError,
            WalletdError::NetworkTimeout { .. } => ErrorCode::NetworkTimeout,
            WalletdError::RateLimited { .. } => ErrorCode::RateLimited,
            WalletdError::ContractError(_) => ErrorCode::ContractError,
            WalletdError::NotSynced => ErrorCode::NotSynced,
            WalletdError::NotSupported(_) => ErrorCode::NotSupported,
            _ => ErrorCode::Unknown,
        }
    }

    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WalletdError::NetworkTimeout { .. }
                | WalletdError::RateLimited { .. }
                | WalletdError::RpcConnectionError { .. }
                | WalletdError::TransactionTimeout
        )
    }

    /// Returns suggested retry delay in seconds, if applicable
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            WalletdError::RateLimited { retry_after_secs } => Some(*retry_after_secs),
            WalletdError::NetworkTimeout { seconds } => Some(*seconds / 2),
            _ if self.is_retryable() => Some(5),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WalletdError::InvalidAddress {
            address: "0x123".to_string(),
            reason: "Too short".to_string(),
        };
        assert!(err.to_string().contains("0x123"));
        assert!(err.to_string().contains("Too short"));
    }

    #[test]
    fn test_error_code() {
        let err = WalletdError::InsufficientBalance { have: 100, need: 200 };
        assert_eq!(err.code(), ErrorCode::InsufficientBalance);
    }

    #[test]
    fn test_retryable() {
        let timeout = WalletdError::NetworkTimeout { seconds: 30 };
        assert!(timeout.is_retryable());
        assert_eq!(timeout.retry_after(), Some(15));

        let rate_limit = WalletdError::RateLimited { retry_after_secs: 60 };
        assert!(rate_limit.is_retryable());
        assert_eq!(rate_limit.retry_after(), Some(60));

        let invalid = WalletdError::InvalidAddress {
            address: "x".into(),
            reason: "bad".into(),
        };
        assert!(!invalid.is_retryable());
        assert_eq!(invalid.retry_after(), None);
    }

    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), std::io::Error> = 
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file missing"));
        
        let with_ctx = result.context("Failed to load config");
        assert!(with_ctx.is_err());
        assert!(with_ctx.unwrap_err().to_string().contains("Failed to load config"));
    }
}
