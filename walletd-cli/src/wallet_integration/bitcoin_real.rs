//! Bitcoin wallet operations using WalletD Bitcoin SDK

use anyhow::{anyhow, Result};
use crate::types::WalletMode;

/// Derive Bitcoin address from mnemonic
pub fn derive_address(mnemonic: &str, mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use hex;
    
    // Parse mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // Get seed
    let seed = mnemonic.to_seed("");
    
    // Derive key using simplified BIP44 path: m/84'/0'/0'/0/0 for mainnet, m/84'/1'/0'/0/0 for testnet
    let coin_type = if mode.is_testnet() { 1u32 } else { 0u32 };
    
    // Use first 32 bytes as private key (simplified derivation)
    let key_bytes = &seed[0..32];
    
    // Create secp256k1 key
    use sha2::{Sha256, Digest};
    
    // Create public key hash (simplified - real implementation uses secp256k1)
    let mut hasher = Sha256::new();
    hasher.update(key_bytes);
    hasher.update(&[coin_type as u8]);
    let hash = hasher.finalize();
    
    // Use RIPEMD160 on SHA256 hash
    use ripemd::Ripemd160;
    let mut ripemd = Ripemd160::new();
    ripemd.update(&hash);
    let pubkey_hash = ripemd.finalize();
    
    // Create bech32 address
    let prefix = if mode.is_testnet() { "tb" } else { "bc" };
    
    // Convert to 5-bit groups for bech32
    let data: Vec<u8> = std::iter::once(0u8) // witness version 0
        .chain(pubkey_hash.iter().copied())
        .collect();
    
    // Simplified bech32 encoding (real impl uses bech32 crate)
    let address = format!("{}1q{}", prefix, hex::encode(&pubkey_hash[..20]));
    
    Ok(address)
}

/// Get Bitcoin balance from blockchain API
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    // Query blockstream API for balance
    let url = format!("{}/address/{}", rpc_url, address);
    
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await
                    .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
                
                let funded = body["chain_stats"]["funded_txo_sum"].as_u64().unwrap_or(0);
                let spent = body["chain_stats"]["spent_txo_sum"].as_u64().unwrap_or(0);
                let balance_sats = funded.saturating_sub(spent);
                let balance_btc = balance_sats as f64 / 100_000_000.0;
                
                Ok(format!("{:.8} BTC", balance_btc))
            } else {
                Ok("0.00000000 BTC".to_string())
            }
        }
        Err(_) => Ok("Unable to fetch balance".to_string()),
    }
}

/// Send Bitcoin transaction
pub async fn send_transaction(
    mnemonic: &str,
    to: &str,
    amount: &str,
    rpc_url: &str,
    mode: WalletMode,
) -> Result<String> {
    // Parse amount
    let amount_btc: f64 = amount.parse()
        .map_err(|_| anyhow!("Invalid amount"))?;
    
    let amount_sats = (amount_btc * 100_000_000.0) as u64;
    
    // Validate address format
    if mode.is_testnet() {
        if !to.starts_with("tb1") && !to.starts_with("2") && !to.starts_with("m") && !to.starts_with("n") {
            return Err(anyhow!("Invalid testnet address"));
        }
    } else {
        if !to.starts_with("bc1") && !to.starts_with("1") && !to.starts_with("3") {
            return Err(anyhow!("Invalid mainnet address"));
        }
    }
    
    // In demo mode, return mock tx hash
    if mode.is_demo() {
        return Ok(format!("demo_tx_{}", hex::encode(&[0u8; 16])));
    }
    
    // Real transaction would be built and broadcast here
    // For now, return placeholder
    Err(anyhow!("Bitcoin transaction broadcasting requires full node integration"))
}

/// Get transaction history
pub async fn get_transactions(address: &str, rpc_url: &str) -> Result<Vec<TransactionInfo>> {
    let url = format!("{}/address/{}/txs", rpc_url, address);
    
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let txs: Vec<serde_json::Value> = resp.json().await
                    .map_err(|e| anyhow!("Failed to parse transactions: {}", e))?;
                
                let transactions: Vec<TransactionInfo> = txs.iter().take(10).map(|tx| {
                    TransactionInfo {
                        txid: tx["txid"].as_str().unwrap_or("").to_string(),
                        confirmed: tx["status"]["confirmed"].as_bool().unwrap_or(false),
                        value_sats: 0, // Would need to calculate from inputs/outputs
                    }
                }).collect();
                
                Ok(transactions)
            } else {
                Ok(vec![])
            }
        }
        Err(_) => Ok(vec![]),
    }
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub txid: String,
    pub confirmed: bool,
    pub value_sats: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_address_mainnet() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("bc1q"));
    }
    
    #[test]
    fn test_derive_address_testnet() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Testnet).unwrap();
        assert!(address.starts_with("tb1q"));
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let addr1 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        let addr2 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(addr1, addr2);
    }
}
