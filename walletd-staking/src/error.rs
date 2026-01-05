use thiserror::Error;

pub type StakingResult<T> = Result<T, StakingError>;

#[derive(Error, Debug)]
pub enum StakingError {
    #[error("Insufficient balance: need {required}, have {available}")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("Validator not found: {0}")]
    ValidatorNotFound(String),
    #[error("Minimum stake not met: {0}")]
    MinimumStake(u64),
    #[error("Unbonding period active")]
    UnbondingPeriod,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid amount")]
    InvalidAmount,
}
