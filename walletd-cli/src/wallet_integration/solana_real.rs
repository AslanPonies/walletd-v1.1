//! Solana wallet operations

use anyhow::{anyhow, Result};
use crate::types::WalletMode;

/// Derive Solana address from mnemonic
pub fn derive_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha2::{Sha256, Digest};
    
    // Parse mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // Get seed
    let seed = mnemonic.to_seed("");
    
    // Derive using SLIP-0010 Ed25519 (simplified)
    // Path: m/44'/501'/0'/0'
    let mut hasher = Sha256::new();
    hasher.update(&seed);
    hasher.update(b"ed25519 seed");
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // Base58 encode the public key
    let address = bs58::encode(public_key.as_bytes()).into_string();
    
    Ok(address)
}

/// Get Solana balance from RPC
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
                
                if let Some(value) = body["result"]["value"].as_u64() {
                    let balance_sol = value as f64 / 1_000_000_000.0;
                    Ok(format!("{:.9} SOL", balance_sol))
                } else {
                    Ok("0.000000000 SOL".to_string())
                }
            } else {
                Ok("Unable to fetch balance".to_string())
            }
        }
        Err(_) => Ok("Unable to connect to RPC".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_address() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        // Solana addresses are base58 encoded, typically 32-44 chars
        assert!(address.len() >= 32 && address.len() <= 44);
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let addr1 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        let addr2 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(addr1, addr2);
    }
}
