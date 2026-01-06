#![allow(dead_code)]
//! Ethereum wallet operations using WalletD Ethereum SDK

use anyhow::{anyhow, Result};
use crate::types::WalletMode;

/// Derive Ethereum address from mnemonic
pub fn derive_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha3::{Keccak256, Digest};
    
    // Parse mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // Get seed
    let seed = mnemonic.to_seed("");
    
    // Derive key using BIP44 path: m/44'/60'/0'/0/0
    // Simplified: use first 32 bytes as private key
    let key_bytes = &seed[0..32];
    
    // Create secp256k1 key and derive public key
    use k256::ecdsa::SigningKey;
    
    let signing_key = SigningKey::from_bytes(key_bytes.into())
        .map_err(|e| anyhow!("Invalid private key: {:?}", e))?;
    
    let verifying_key = signing_key.verifying_key();
    let public_key = verifying_key.to_encoded_point(false);
    let public_key_bytes = public_key.as_bytes();
    
    // Skip the 0x04 prefix and hash with Keccak256
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]); // Skip 0x04 prefix
    let hash = hasher.finalize();
    
    // Take last 20 bytes as address
    let address_bytes = &hash[12..32];
    let address = format!("0x{}", hex::encode(address_bytes));
    
    // Convert to checksum address
    Ok(to_checksum_address(&address))
}

/// Convert to EIP-55 checksum address
fn to_checksum_address(address: &str) -> String {
    use sha3::{Keccak256, Digest};
    
    let address = address.trim_start_matches("0x").to_lowercase();
    
    let mut hasher = Keccak256::new();
    hasher.update(address.as_bytes());
    let hash = hasher.finalize();
    
    let checksum: String = address.chars().enumerate().map(|(i, c)| {
        if c.is_ascii_alphabetic() {
            let hash_byte = hash[i / 2];
            let hash_nibble = if i % 2 == 0 { hash_byte >> 4 } else { hash_byte & 0x0f };
            if hash_nibble >= 8 {
                c.to_ascii_uppercase()
            } else {
                c
            }
        } else {
            c
        }
    }).collect();
    
    format!("0x{}", checksum)
}

/// Get Ethereum balance from JSON-RPC
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
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
                
                if let Some(result) = body["result"].as_str() {
                    let balance_wei = u128::from_str_radix(result.trim_start_matches("0x"), 16)
                        .unwrap_or(0);
                    let balance_eth = balance_wei as f64 / 1e18;
                    Ok(format!("{:.6} ETH", balance_eth))
                } else if let Some(error) = body["error"]["message"].as_str() {
                    Err(anyhow!("RPC error: {}", error))
                } else {
                    Ok("0.000000 ETH".to_string())
                }
            } else {
                Ok("Unable to fetch balance".to_string())
            }
        }
        Err(_) => Ok("Unable to connect to RPC".to_string()),
    }
}

/// Send Ethereum transaction
pub async fn send_transaction(
    mnemonic: &str,
    to: &str,
    amount: &str,
    rpc_url: &str,
    mode: WalletMode,
) -> Result<String> {
    use bip39::Mnemonic;
    use k256::ecdsa::SigningKey;
    
    
    // Parse amount in ETH
    let amount_eth: f64 = amount.parse()
        .map_err(|_| anyhow!("Invalid amount"))?;
    
    let amount_wei = (amount_eth * 1e18) as u128;
    
    // Validate address
    if !to.starts_with("0x") || to.len() != 42 {
        return Err(anyhow!("Invalid Ethereum address"));
    }
    
    // In demo mode, return mock tx hash
    if mode.is_demo() {
        return Ok(format!("0x{}", hex::encode(&[0u8; 32])));
    }
    
    // Derive private key from mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    let seed = mnemonic.to_seed("");
    let key_bytes = &seed[0..32];
    
    let signing_key = SigningKey::from_bytes(key_bytes.into())
        .map_err(|e| anyhow!("Invalid private key: {:?}", e))?;
    
    // Get nonce
    let from_address = derive_address(&mnemonic.to_string(), mode)?;
    let nonce = get_nonce(&from_address, rpc_url).await?;
    
    // Get gas price
    let gas_price = get_gas_price(rpc_url).await?;
    
    // Build transaction
    let gas_limit = 21000u64; // Standard ETH transfer
    
    let chain_id = if mode.is_testnet() { 11155111u64 } else { 1u64 }; // Sepolia or mainnet
    
    // Encode transaction (simplified - real impl uses RLP)
    let tx_data = encode_transaction(
        nonce,
        gas_price,
        gas_limit,
        to,
        amount_wei,
        chain_id,
    )?;
    
    // Sign transaction
    let tx_hash = sign_and_broadcast(&tx_data, &signing_key, rpc_url).await?;
    
    Ok(tx_hash)
}

async fn get_nonce(address: &str, rpc_url: &str) -> Result<u64> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionCount",
        "params": [address, "pending"],
        "id": 1
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to get nonce: {}", e))?;
    
    let body: serde_json::Value = response.json().await?;
    
    if let Some(result) = body["result"].as_str() {
        let nonce = u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|_| anyhow!("Invalid nonce"))?;
        Ok(nonce)
    } else {
        Ok(0)
    }
}

async fn get_gas_price(rpc_url: &str) -> Result<u128> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to get gas price: {}", e))?;
    
    let body: serde_json::Value = response.json().await?;
    
    if let Some(result) = body["result"].as_str() {
        let gas_price = u128::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|_| anyhow!("Invalid gas price"))?;
        Ok(gas_price)
    } else {
        Ok(20_000_000_000u128) // 20 Gwei default
    }
}

fn encode_transaction(
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to: &str,
    value: u128,
    chain_id: u64,
) -> Result<Vec<u8>> {
    // Simplified RLP encoding for EIP-1559 transaction
    // Real implementation would use proper RLP encoding
    
    let to_bytes = hex::decode(to.trim_start_matches("0x"))
        .map_err(|_| anyhow!("Invalid to address"))?;
    
    let mut data = Vec::new();
    data.extend_from_slice(&nonce.to_be_bytes());
    data.extend_from_slice(&gas_price.to_be_bytes());
    data.extend_from_slice(&gas_limit.to_be_bytes());
    data.extend_from_slice(&to_bytes);
    data.extend_from_slice(&value.to_be_bytes());
    data.extend_from_slice(&chain_id.to_be_bytes());
    
    Ok(data)
}

async fn sign_and_broadcast(
    tx_data: &[u8],
    _signing_key: &k256::ecdsa::SigningKey,
    _rpc_url: &str,
) -> Result<String> {
    use sha3::{Keccak256, Digest};
    
    // Hash transaction
    let mut hasher = Keccak256::new();
    hasher.update(tx_data);
    let _tx_hash = hasher.finalize();
    
    // Sign and broadcast would happen here
    // For now, return placeholder
    Err(anyhow!("Transaction broadcasting not fully implemented. Use a web3 provider."))
}

/// Get ERC-20 token balance
pub async fn get_token_balance(
    address: &str,
    token_contract: &str,
    rpc_url: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    
    // balanceOf(address) function selector
    let selector = "70a08231";
    let padded_address = format!("{:0>64}", address.trim_start_matches("0x"));
    let data = format!("0x{}{}", selector, padded_address);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{
            "to": token_contract,
            "data": data
        }, "latest"],
        "id": 1
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
                
                if let Some(result) = body["result"].as_str() {
                    let balance = u128::from_str_radix(result.trim_start_matches("0x"), 16)
                        .unwrap_or(0);
                    // Assuming 18 decimals (like USDC uses 6, needs token-specific handling)
                    let balance_formatted = balance as f64 / 1e18;
                    Ok(format!("{:.4}", balance_formatted))
                } else {
                    Ok("0".to_string())
                }
            } else {
                Ok("Unable to fetch token balance".to_string())
            }
        }
        Err(_) => Ok("Unable to connect".to_string()),
    }
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
    
    #[test]
    fn test_checksum_address() {
        let address = "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359";
        let checksum = to_checksum_address(address);
        assert!(checksum.starts_with("0x"));
        // Check that some letters are uppercase
        assert!(checksum.chars().any(|c| c.is_uppercase()));
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let addr1 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        let addr2 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(addr1, addr2);
    }
}
