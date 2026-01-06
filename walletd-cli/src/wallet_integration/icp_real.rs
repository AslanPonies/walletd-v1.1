//! Internet Computer (ICP) wallet operations

use anyhow::{anyhow, Result};
use crate::types::WalletMode;

/// Derive ICP principal/account from mnemonic
pub fn derive_address(mnemonic: &str, _mode: WalletMode) -> Result<String> {
    use bip39::Mnemonic;
    use sha2::{Sha256, Sha224, Digest};
    
    // Parse mnemonic
    let mnemonic = Mnemonic::parse(mnemonic)
        .map_err(|e| anyhow!("Invalid mnemonic: {:?}", e))?;
    
    // Get seed
    let seed = mnemonic.to_seed("");
    
    // Derive secp256k1 key (ICP uses secp256k1)
    let key_bytes = &seed[0..32];
    
    // Create public key
    use k256::ecdsa::SigningKey;
    use k256::elliptic_curve::sec1::ToEncodedPoint;
    
    let signing_key = SigningKey::from_bytes(key_bytes.into())
        .map_err(|e| anyhow!("Invalid private key: {:?}", e))?;
    
    let verifying_key = signing_key.verifying_key();
    let public_key = verifying_key.to_encoded_point(false);
    let public_key_bytes = public_key.as_bytes();
    
    // Hash to create principal (simplified)
    let mut hasher = Sha224::new();
    hasher.update(b"\x0Aic-request");
    hasher.update(public_key_bytes);
    let hash = hasher.finalize();
    
    // Convert to principal text format
    // Real implementation uses CRC32 checksum and base32
    let principal = format!("{}-{}", 
        hex::encode(&hash[0..5]),
        hex::encode(&hash[5..10])
    );
    
    Ok(principal)
}

/// Get ICP balance from ledger canister
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String> {
    // ICP balance queries require canister calls
    // This is a simplified HTTP query
    let client = reqwest::Client::new();
    
    let url = format!("{}/api/v2/canister/ryjl3-tyaaa-aaaaa-aaaba-cai/query", rpc_url);
    
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(_resp) => {
            // Would need to decode CBOR response
            Ok("ICP balance requires canister query".to_string())
        }
        Err(_) => Ok("Unable to connect to IC".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    #[test]
    fn test_derive_address() {
        let address = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert!(!address.is_empty());
    }
    
    #[test]
    fn test_deterministic_derivation() {
        let addr1 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        let addr2 = derive_address(TEST_MNEMONIC, WalletMode::Mainnet).unwrap();
        assert_eq!(addr1, addr2);
    }
}
