use thiserror::Error;

pub type HardwareResult<T> = Result<T, HardwareError>;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Device disconnected")]
    Disconnected,
    #[error("User rejected")]
    UserRejected,
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Communication error: {0}")]
    Communication(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("Device locked")]
    Locked,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
