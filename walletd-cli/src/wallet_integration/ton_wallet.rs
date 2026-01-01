//! TON Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ed25519_dalek::SigningKey;
use sha2::{Sha256, Digest};

pub struct TonWallet {
    signing_key: SigningKey,
    pub address: String,
    pub network: String,
    api_url: String,
}

impl TonWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let address = Self::derive_address(&signing_key);
        let api_url = if network == "mainnet" {
            "https://toncenter.com/api/v2"
        } else {
            "https://testnet.toncenter.com/api/v2"
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), api_url })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::TON)?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let address = Self::derive_address(&signing_key);
        let api_url = if network == "mainnet" {
            "https://toncenter.com/api/v2"
        } else {
            "https://testnet.toncenter.com/api/v2"
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), api_url })
    }

    fn derive_address(signing_key: &SigningKey) -> String {
        // Simplified TON address derivation
        let pk_bytes = signing_key.verifying_key().as_bytes();
        let hash = Sha256::digest(pk_bytes);
        format!("EQ{}", base64::encode(&hash[..32]))
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let url = format!("{}/getAddressBalance?address={}", self.api_url, self.address);
        let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        if let Some(balance) = resp["result"].as_str() {
            Ok(balance.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => format!("https://tonscan.org/address/{}", self.address),
            _ => format!("https://testnet.tonscan.org/address/{}", self.address),
        }
    }
}
