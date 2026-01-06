#![allow(dead_code)]
//! Base L2 wallet operations
//! Base is EVM-compatible, so we reuse Ethereum logic

use anyhow::Result;
use crate::types::WalletMode;
use super::ethereum_real;

/// Derive Base address (same as Ethereum)
pub fn derive_address(mnemonic: &str, mode: WalletMode) -> Result<String> {
    ethereum_real::derive_address(mnemonic, mode)
}

/// Get Base balance
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    let result = ethereum_real::get_balance(address, rpc_url).await?;
    // Replace ETH with BASE-ETH for clarity
    Ok(result.replace("ETH", "ETH (Base)"))
}

/// Send Base transaction
pub async fn send_transaction(
    mnemonic: &str,
    to: &str,
    amount: &str,
    rpc_url: &str,
    mode: WalletMode,
) -> Result<String> {
    ethereum_real::send_transaction(mnemonic, to, amount, rpc_url, mode).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_address() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }
}
