use thiserror::Error;

#[derive(Error, Debug)]
pub enum AvalancheError {
    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
