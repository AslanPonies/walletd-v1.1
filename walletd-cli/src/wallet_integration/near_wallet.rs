//! Near Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ed25519_dalek::SigningKey;

pub struct NearWallet {
    signing_key: SigningKey,
    pub account_id: String,
    pub network: String,
    rpc_url: String,
}

impl NearWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let account_id = format!("{}.{}", &pk_hex[..16], if network == "mainnet" { "near" } else { "testnet" });
        let rpc_url = if network == "mainnet" {
            "https://rpc.mainnet.near.org"
        } else {
            "https://rpc.testnet.near.org"
        }.to_string();
        
        Ok(Self { signing_key, account_id, network: network.to_string(), rpc_url })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::NEAR)?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let pk_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let account_id = format!("{}.{}", &pk_hex[..16], if network == "mainnet" { "near" } else { "testnet" });
        let rpc_url = if network == "mainnet" {
            "https://rpc.mainnet.near.org"
        } else {
            "https://rpc.testnet.near.org"
        }.to_string();
        
        Ok(Self { signing_key, account_id, network: network.to_string(), rpc_url })
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "query",
            "params": {
                "request_type": "view_account",
                "finality": "final",
                "account_id": self.account_id
            }
        });

        let resp: serde_json::Value = reqwest::Client::new()
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(amount) = resp["result"]["amount"].as_str() {
            Ok(amount.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => format!("https://explorer.near.org/accounts/{}", self.account_id),
            _ => format!("https://explorer.testnet.near.org/accounts/{}", self.account_id),
        }
    }
}
