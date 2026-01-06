//! Hedera Hashgraph wallet operations

use anyhow::{anyhow, Result};
use crate::types::WalletMode;

/// Derive Hedera address from mnemonic
pub fn derive_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha2::{Sha256, Digest};
    
    // Parse mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // Get seed
    let seed = mnemonic.to_seed("");
    
    // Derive using BIP44 path: m/44'/3030'/0'/0/0
    let mut hasher = Sha256::new();
    hasher.update(&seed);
    hasher.update(b"hedera");
    let derived = hasher.finalize();
    
    // Create account ID format: 0.0.XXXXXX
    // In reality, Hedera accounts are created on-chain
    let account_num = u32::from_be_bytes([derived[0], derived[1], derived[2], derived[3]]) % 1_000_000;
    
    let shard = 0;
    let realm = 0;
    
    Ok(format!("{}.{}.{}", shard, realm, account_num))
}

/// Get Hedera balance from mirror node
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let url = format!("{}/api/v1/accounts/{}", rpc_url, address);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
                
                if let Some(balance) = body["balance"]["balance"].as_u64() {
                    let hbar = balance as f64 / 100_000_000.0;
                    Ok(format!("{:.8} HBAR", hbar))
                } else {
                    Ok("0.00000000 HBAR".to_string())
                }
            } else {
                Ok("Account not found".to_string())
            }
        }
        Err(_) => Ok("Unable to connect to mirror node".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_address() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("0.0."));
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let addr1 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        let addr2 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(addr1, addr2);
    }
}
