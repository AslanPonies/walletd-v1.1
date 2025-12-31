//! Generic ERC-20 adapter trait and types

use alloy::primitives::{Address, U256};
use async_trait::async_trait;
use std::fmt;

/// Error type for ERC-20 operations
#[derive(Debug)]
pub enum Erc20Error {
    /// Contract call failed
    ContractError(String),
    /// Invalid address
    InvalidAddress(String),
    /// Provider error
    ProviderError(String),
}

impl fmt::Display for Erc20Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Erc20Error::ContractError(e) => write!(f, "Contract error: {}", e),
            Erc20Error::InvalidAddress(e) => write!(f, "Invalid address: {}", e),
            Erc20Error::ProviderError(e) => write!(f, "Provider error: {}", e),
        }
    }
}

impl std::error::Error for Erc20Error {}

/// Trait for ERC-20 token adapters
#[async_trait]
pub trait Erc20Adapter: Send + Sync {
    /// Returns the contract address for this token
    fn contract_address(&self) -> Address;

    /// Returns the token name
    async fn name(&self, rpc_url: &str) -> Result<String, Erc20Error>;

    /// Returns the token symbol
    async fn symbol(&self, rpc_url: &str) -> Result<String, Erc20Error>;

    /// Returns the number of decimals
    async fn decimals(&self, rpc_url: &str) -> Result<u8, Erc20Error>;

    /// Returns the total supply
    async fn total_supply(&self, rpc_url: &str) -> Result<U256, Erc20Error>;

    /// Returns the balance of the given address
    async fn balance_of(&self, rpc_url: &str, owner: Address) -> Result<U256, Erc20Error>;

    /// Returns the allowance for a spender
    async fn allowance(&self, rpc_url: &str, owner: Address, spender: Address) -> Result<U256, Erc20Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erc20_error_display_contract_error() {
        let error = Erc20Error::ContractError("call failed".to_string());
        assert_eq!(format!("{}", error), "Contract error: call failed");
    }

    #[test]
    fn test_erc20_error_display_invalid_address() {
        let error = Erc20Error::InvalidAddress("bad address".to_string());
        assert_eq!(format!("{}", error), "Invalid address: bad address");
    }

    #[test]
    fn test_erc20_error_display_provider_error() {
        let error = Erc20Error::ProviderError("connection refused".to_string());
        assert_eq!(format!("{}", error), "Provider error: connection refused");
    }

    #[test]
    fn test_erc20_error_is_error_trait() {
        let error: Box<dyn std::error::Error> = Box::new(Erc20Error::ContractError("test".to_string()));
        assert!(error.to_string().contains("test"));
    }

    #[test]
    fn test_erc20_error_debug() {
        let error = Erc20Error::ContractError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ContractError"));
        assert!(debug_str.contains("test"));
    }
}
