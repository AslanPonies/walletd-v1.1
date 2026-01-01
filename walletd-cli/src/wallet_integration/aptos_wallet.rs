//! Aptos Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ed25519_dalek::SigningKey;

pub struct AptosWallet {
    signing_key: SigningKey,
    pub address: String,
    pub network: String,
    api_url: String,
}

impl AptosWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let address = format!("0x{}", hex::encode(signing_key.verifying_key().as_bytes()));
        let api_url = match network {
            "mainnet" => "https://fullnode.mainnet.aptoslabs.com/v1",
            "testnet" => "https://fullnode.testnet.aptoslabs.com/v1",
            _ => "https://fullnode.devnet.aptoslabs.com/v1",
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), api_url })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::APTOS)?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let address = format!("0x{}", hex::encode(signing_key.verifying_key().as_bytes()));
        let api_url = match network {
            "mainnet" => "https://fullnode.mainnet.aptoslabs.com/v1",
            "testnet" => "https://fullnode.testnet.aptoslabs.com/v1",
            _ => "https://fullnode.devnet.aptoslabs.com/v1",
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), api_url })
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let url = format!("{}/accounts/{}/resource/0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>", 
            self.api_url, self.address);
        
        let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        if let Some(coin) = resp["data"]["coin"]["value"].as_str() {
            Ok(coin.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => format!("https://explorer.aptoslabs.com/account/{}", self.address),
            _ => format!("https://explorer.aptoslabs.com/account/{}?network={}", self.address, self.network),
        }
    }
}
