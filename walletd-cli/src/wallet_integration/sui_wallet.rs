//! SUI Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ed25519_dalek::SigningKey;

pub struct SuiWallet {
    signing_key: SigningKey,
    pub address: String,
    pub network: String,
    rpc_url: String,
}

impl SuiWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let address = format!("0x{}", hex::encode(signing_key.verifying_key().as_bytes()));
        let rpc_url = match network {
            "mainnet" => "https://fullnode.mainnet.sui.io",
            "testnet" => "https://fullnode.testnet.sui.io",
            _ => "https://fullnode.devnet.sui.io",
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), rpc_url })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::SUI)?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let address = format!("0x{}", hex::encode(signing_key.verifying_key().as_bytes()));
        let rpc_url = match network {
            "mainnet" => "https://fullnode.mainnet.sui.io",
            "testnet" => "https://fullnode.testnet.sui.io",
            _ => "https://fullnode.devnet.sui.io",
        }.to_string();
        
        Ok(Self { signing_key, address, network: network.to_string(), rpc_url })
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "suix_getBalance",
            "params": [self.address, "0x2::sui::SUI"]
        });

        let resp: serde_json::Value = reqwest::Client::new()
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(balance) = resp["result"]["totalBalance"].as_str() {
            Ok(balance.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => format!("https://suiexplorer.com/address/{}", self.address),
            _ => format!("https://suiexplorer.com/address/{}?network={}", self.address, self.network),
        }
    }
}
