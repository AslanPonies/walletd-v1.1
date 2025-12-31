use thiserror::Error;

#[derive(Error, Debug)]
pub enum CardanoError {
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    #[error("Address error: {0}")]
    AddressError(String),

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("UTXO error: {0}")]
    UtxoError(String),

    #[error("Insufficient funds: required {required} lovelace, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
