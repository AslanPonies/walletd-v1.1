//! Cosmos Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use sha2::{Sha256, Digest};

pub struct CosmosWallet {
    pub address: String,
    pub prefix: String,
    key_bytes: [u8; 32],
    rpc_url: String,
}

impl CosmosWallet {
    pub fn new(prefix: &str) -> Result<Self> {
        let mut key_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key_bytes);
        let address = Self::derive_address(&key_bytes, prefix);
        Ok(Self { 
            address, 
            prefix: prefix.to_string(), 
            key_bytes,
            rpc_url: "https://rpc.cosmos.network".to_string(),
        })
    }

    pub fn from_mnemonic(mnemonic: &str, prefix: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::COSMOS)?;
        let address = Self::derive_address(&key_bytes, prefix);
        Ok(Self { 
            address, 
            prefix: prefix.to_string(), 
            key_bytes,
            rpc_url: "https://rpc.cosmos.network".to_string(),
        })
    }

    fn derive_address(key_bytes: &[u8; 32], prefix: &str) -> String {
        // Simplified - real impl uses bech32 encoding
        let hash = Sha256::digest(key_bytes);
        format!("{}{}", prefix, hex::encode(&hash[..20]))
    }

    pub async fn get_balance(&self) -> Result<u64> {
        // Query LCD API
        let url = format!("{}/cosmos/bank/v1beta1/balances/{}", self.rpc_url, self.address);
        let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        if let Some(balances) = resp["balances"].as_array() {
            for balance in balances {
                if balance["denom"] == "uatom" {
                    return Ok(balance["amount"].as_str().unwrap_or("0").parse().unwrap_or(0));
                }
            }
        }
        Ok(0)
    }

    pub fn explorer_url(&self) -> String {
        format!("https://www.mintscan.io/cosmos/address/{}", self.address)
    }
}
