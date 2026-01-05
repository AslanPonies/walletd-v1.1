use thiserror::Error;

pub type MultisigResult<T> = Result<T, MultisigError>;

#[derive(Error, Debug)]
pub enum MultisigError {
    #[error("Invalid threshold: M must be <= N")]
    InvalidThreshold,
    #[error("Invalid signer count")]
    InvalidSignerCount,
    #[error("Transaction not found")]
    TransactionNotFound,
    #[error("Unauthorized signer")]
    UnauthorizedSigner,
    #[error("Duplicate signature")]
    DuplicateSignature,
    #[error("Insufficient signatures")]
    InsufficientSignatures,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unsupported chain")]
    UnsupportedChain,
    #[error("Script error: {0}")]
    ScriptError(String),
}
