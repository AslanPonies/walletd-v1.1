//! Tron Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use sha2::{Sha256, Digest};

pub struct TronWallet {
    pub address: String,
    key_bytes: [u8; 32],
    api_url: String,
}

impl TronWallet {
    pub fn new() -> Result<Self> {
        let mut key_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key_bytes);
        let address = Self::derive_address(&key_bytes);
        Ok(Self { 
            address, 
            key_bytes,
            api_url: "https://api.trongrid.io".to_string(),
        })
    }

    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::TRON)?;
        let address = Self::derive_address(&key_bytes);
        Ok(Self { 
            address, 
            key_bytes,
            api_url: "https://api.trongrid.io".to_string(),
        })
    }

    fn derive_address(key_bytes: &[u8; 32]) -> String {
        // Simplified Tron address derivation
        let hash = Sha256::digest(key_bytes);
        let mut addr_bytes = vec![0x41u8]; // Mainnet prefix
        addr_bytes.extend_from_slice(&hash[12..32]);
        bs58::encode(&addr_bytes).into_string()
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let url = format!("{}/v1/accounts/{}", self.api_url, self.address);
        let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        if let Some(balance) = resp["data"][0]["balance"].as_u64() {
            Ok(balance)
        } else {
            Ok(0)
        }
    }

    pub fn explorer_url(&self) -> String {
        format!("https://tronscan.org/#/address/{}", self.address)
    }
}
