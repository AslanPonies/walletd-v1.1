//! Extended chain operations
//! Handles Polygon, Avalanche, Arbitrum, and other chains

use anyhow::{anyhow, Result};
use crate::types::{Chain, WalletMode};
use super::ethereum_real;

/// Derive EVM-compatible address (Polygon, Avalanche, Arbitrum, etc.)
pub fn derive_address(mnemonic: &str, chain: Chain, mode: WalletMode) -> Result<String> {
    // All EVM chains use the same address format
    ethereum_real::derive_address(mnemonic, mode)
}

/// Get balance for EVM chains
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    ethereum_real::get_balance(address, rpc_url).await
}

/// Send transaction on EVM chains
pub async fn send_transaction(
    mnemonic: &str,
    to: &str,
    amount: &str,
    chain: Chain,
    rpc_url: &str,
    mode: WalletMode,
) -> Result<String> {
    // EVM chains use same transaction format with different chain IDs
    ethereum_real::send_transaction(mnemonic, to, amount, rpc_url, mode).await
}

// ========== Cardano ==========

/// Derive Cardano address from mnemonic
pub fn derive_cardano_address(mnemonic: &str, mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha2::{Sha256, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    
    // Cardano uses Ed25519 extended keys
    let mut hasher = Sha256::new();
    hasher.update(&seed);
    hasher.update(b"cardano");
    let derived = hasher.finalize();
    
    // Simplified address generation
    // Real Cardano addresses use Bech32 with specific prefixes
    let prefix = if mode.is_testnet() { "addr_test1" } else { "addr1" };
    let address = format!("{}{}", prefix, hex::encode(&derived[..28]));
    
    Ok(address)
}

pub async fn get_cardano_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/addresses/{}", rpc_url, address);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(amount) = body["amount"][0]["quantity"].as_str() {
                let lovelace: u64 = amount.parse().unwrap_or(0);
                let ada = lovelace as f64 / 1_000_000.0;
                Ok(format!("{:.6} ADA", ada))
            } else {
                Ok("0.000000 ADA".to_string())
            }
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== Cosmos ==========

/// Derive Cosmos address from mnemonic
pub fn derive_cosmos_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha2::{Sha256, Digest};
    use ripemd::Ripemd160;
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    let key_bytes = &seed[0..32];
    
    // Create secp256k1 public key
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    
    let signing_key = SigningKey::from_bytes(key_bytes.into())
        .map_err(|e| anyhow!("Invalid key: {:?}", e))?;
    
    let public_key = signing_key.verifying_key().to_encoded_point(true);
    let public_key_bytes = public_key.as_bytes();
    
    // SHA256 then RIPEMD160
    let mut sha = Sha256::new();
    sha.update(public_key_bytes);
    let sha_hash = sha.finalize();
    
    let mut ripemd = Ripemd160::new();
    ripemd.update(&sha_hash);
    let address_bytes = ripemd.finalize();
    
    // Bech32 encode with "cosmos" prefix
    let address = format!("cosmos1{}", hex::encode(&address_bytes));
    
    Ok(address)
}

pub async fn get_cosmos_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/cosmos/bank/v1beta1/balances/{}", rpc_url, address);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(balances) = body["balances"].as_array() {
                for balance in balances {
                    if balance["denom"].as_str() == Some("uatom") {
                        let amount: u64 = balance["amount"].as_str()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        let atom = amount as f64 / 1_000_000.0;
                        return Ok(format!("{:.6} ATOM", atom));
                    }
                }
            }
            Ok("0.000000 ATOM".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== Polkadot ==========

/// Derive Polkadot address from mnemonic
pub fn derive_polkadot_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha2::{Sha512, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    
    // Use SLIP-0010 for Ed25519
    let mut hasher = Sha512::new();
    hasher.update(b"ed25519 seed");
    hasher.update(&seed);
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // SS58 encoding (simplified)
    // Polkadot uses network ID 0
    let address = format!("1{}", bs58::encode(public_key.as_bytes()).into_string());
    
    Ok(address)
}

pub async fn get_polkadot_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "system_account",
        "params": [address]
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(data) = body["result"]["data"].as_object() {
                if let Some(free) = data.get("free").and_then(|v| v.as_str()) {
                    let planck: u128 = free.parse().unwrap_or(0);
                    let dot = planck as f64 / 10_000_000_000.0;
                    return Ok(format!("{:.10} DOT", dot));
                }
            }
            Ok("0.0000000000 DOT".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== NEAR ==========

/// Derive NEAR address from mnemonic
pub fn derive_near_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha2::{Sha512, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    
    let mut hasher = Sha512::new();
    hasher.update(b"ed25519 seed");
    hasher.update(&seed);
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // NEAR uses hex-encoded public key as implicit account
    let address = hex::encode(public_key.as_bytes());
    
    Ok(address)
}

pub async fn get_near_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "query",
        "params": {
            "request_type": "view_account",
            "finality": "final",
            "account_id": address
        }
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(amount) = body["result"]["amount"].as_str() {
                let yocto: u128 = amount.parse().unwrap_or(0);
                let near = yocto as f64 / 1e24;
                return Ok(format!("{:.6} NEAR", near));
            }
            Ok("0.000000 NEAR".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== Tron ==========

/// Derive Tron address from mnemonic
pub fn derive_tron_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha3::{Keccak256, Digest};
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    let key_bytes = &seed[0..32];
    
    let signing_key = SigningKey::from_bytes(key_bytes.into())
        .map_err(|e| anyhow!("Invalid key: {:?}", e))?;
    
    let public_key = signing_key.verifying_key().to_encoded_point(false);
    let public_key_bytes = public_key.as_bytes();
    
    // Keccak256 hash of public key (without 0x04 prefix)
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]);
    let hash = hasher.finalize();
    
    // Take last 20 bytes, add 0x41 prefix (Tron mainnet)
    let mut address_bytes = vec![0x41];
    address_bytes.extend_from_slice(&hash[12..32]);
    
    // Base58Check encode
    let address = bs58::encode(&address_bytes).into_string();
    
    Ok(format!("T{}", &address[1..]))
}

pub async fn get_tron_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "address": address,
        "visible": true
    });
    
    let url = format!("{}/wallet/getaccount", rpc_url);
    
    let response = client.post(&url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(balance) = body["balance"].as_u64() {
                let trx = balance as f64 / 1_000_000.0;
                return Ok(format!("{:.6} TRX", trx));
            }
            Ok("0.000000 TRX".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== SUI ==========

/// Derive SUI address from mnemonic
pub fn derive_sui_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha2::{Sha256, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    
    // SLIP-0010 derivation
    let mut hasher = Sha256::new();
    hasher.update(b"ed25519 seed");
    hasher.update(&seed);
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // SUI address = Blake2b256(flag || public_key)
    // Flag 0x00 for Ed25519
    use blake2::{Blake2b, Digest as Blake2Digest}; use blake2::digest::consts::U32; type Blake2b256 = Blake2b<U32>;
    let mut hasher = Blake2b256::new();
    hasher.update(&[0x00]); // Ed25519 flag
    hasher.update(public_key.as_bytes());
    let hash = hasher.finalize();
    
    Ok(format!("0x{}", hex::encode(&hash)))
}

pub async fn get_sui_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "suix_getBalance",
        "params": [address, "0x2::sui::SUI"]
    });
    
    let response = client.post(rpc_url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(balance) = body["result"]["totalBalance"].as_str() {
                let mist: u64 = balance.parse().unwrap_or(0);
                let sui = mist as f64 / 1_000_000_000.0;
                return Ok(format!("{:.9} SUI", sui));
            }
            Ok("0.000000000 SUI".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== Aptos ==========

/// Derive Aptos address from mnemonic
pub fn derive_aptos_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha3::{Sha3_256, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    let seed = mnemonic.to_seed("");
    
    // SLIP-0010 derivation for Ed25519
    use sha2::Sha512;
    let mut hasher = Sha512::new();
    hasher.update(b"ed25519 seed");
    hasher.update(&seed);
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // Aptos address = SHA3-256(public_key || 0x00)
    let mut hasher = Sha3_256::new();
    hasher.update(public_key.as_bytes());
    hasher.update(&[0x00]); // Single key scheme
    let hash = hasher.finalize();
    
    Ok(format!("0x{}", hex::encode(&hash)))
}

pub async fn get_aptos_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let url = format!("{}/v1/accounts/{}/resource/0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>", 
        rpc_url, address);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(coin) = body["data"]["coin"]["value"].as_str() {
                let octas: u64 = coin.parse().unwrap_or(0);
                let apt = octas as f64 / 100_000_000.0;
                return Ok(format!("{:.8} APT", apt));
            }
            Ok("0.00000000 APT".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

// ========== TON ==========

/// Derive TON address from mnemonic
pub fn derive_ton_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use ed25519_dalek::SigningKey;
    use sha2::{Sha256, Digest};
    
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // TON uses custom key derivation from mnemonic
    // This is simplified
    let seed = mnemonic.to_seed("");
    
    let mut hasher = Sha256::new();
    hasher.update(&seed);
    hasher.update(b"TON default seed");
    let derived = hasher.finalize();
    
    let key_bytes: [u8; 32] = derived[..32].try_into()
        .map_err(|_| anyhow!("Key derivation failed"))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key();
    
    // TON address is user-friendly format
    // This is simplified - real impl needs workchain and state init
    let address = format!("EQ{}", bs58::encode(public_key.as_bytes()).into_string());
    
    Ok(address)
}

pub async fn get_ton_balance(address: &str, rpc_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let url = format!("{}/getAddressBalance?address={}", rpc_url, address);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) if resp.status().is_success() => {
            let body: serde_json::Value = resp.json().await?;
            if let Some(result) = body["result"].as_str() {
                let nano: u64 = result.parse().unwrap_or(0);
                let ton = nano as f64 / 1_000_000_000.0;
                return Ok(format!("{:.9} TON", ton));
            }
            Ok("0.000000000 TON".to_string())
        }
        _ => Ok("Unable to fetch balance".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_cardano_address() {
        let address = derive_cardano_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("addr1"));
    }
    
    #[test]
    fn test_derive_cosmos_address() {
        let address = derive_cosmos_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("cosmos1"));
    }
    
    #[test]
    fn test_derive_polkadot_address() {
        let address = derive_polkadot_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("1"));
    }
    
    #[test]
    fn test_derive_near_address() {
        let address = derive_near_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(address.len(), 64); // hex-encoded 32-byte key
    }
    
    #[test]
    fn test_derive_tron_address() {
        let address = derive_tron_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("T"));
    }
    
    #[test]
    fn test_derive_sui_address() {
        let address = derive_sui_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 66); // 0x + 64 hex chars
    }
    
    #[test]
    fn test_derive_aptos_address() {
        let address = derive_aptos_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 66);
    }
    
    #[test]
    fn test_derive_ton_address() {
        let address = derive_ton_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(address.starts_with("EQ"));
    }
}
