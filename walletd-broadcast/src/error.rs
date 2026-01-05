//! Broadcast error types

use thiserror::Error;

/// Result type for broadcast operations
pub type BroadcastResult<T> = Result<T, BroadcastError>;

/// Broadcast errors
#[derive(Error, Debug)]
pub enum BroadcastError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Transaction rejected: {0}")]
    Rejected(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("Nonce too low: expected {expected}, got {got}")]
    NonceTooLow { expected: u64, got: u64 },

    #[error("Gas price too low: minimum {minimum}")]
    GasPriceTooLow { minimum: u64 },

    #[error("Transaction already exists: {tx_hash}")]
    AlreadyExists { tx_hash: String },

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("All providers failed")]
    AllProvidersFailed,

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Chain not supported: {0}")]
    ChainNotSupported(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for BroadcastError {
    fn from(err: serde_json::Error) -> Self {
        BroadcastError::Serialization(err.to_string())
    }
}

impl BroadcastError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BroadcastError::Network(_)
                | BroadcastError::ProviderUnavailable(_)
                | BroadcastError::Timeout { .. }
                | BroadcastError::RateLimited { .. }
        )
    }

    /// Get suggested retry delay in seconds
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            BroadcastError::RateLimited { retry_after } => Some(*retry_after),
            BroadcastError::Timeout { .. } => Some(5),
            BroadcastError::Network(_) => Some(2),
            BroadcastError::ProviderUnavailable(_) => Some(10),
            _ => None,
        }
    }
}
