//! Solana Wallet - Real Implementation

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::Deserialize;

pub struct SolanaWallet {
    signing_key: SigningKey,
    pub address: String,
    pub cluster: String,
    rpc_url: String,
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
}

#[derive(Deserialize)]
struct BalanceResult {
    value: u64,
}

impl SolanaWallet {
    pub fn new(cluster: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();
        let address = bs58::encode(verifying_key.as_bytes()).into_string();
        
        let rpc_url = match cluster {
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "mainnet-beta" => "https://api.mainnet-beta.solana.com",
            _ => "https://api.devnet.solana.com",
        }.to_string();

        Ok(Self { signing_key, address, cluster: cluster.to_string(), rpc_url })
    }

    pub fn from_mnemonic(mnemonic: &str, cluster: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::SOLANA)?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = bs58::encode(verifying_key.as_bytes()).into_string();
        
        let rpc_url = match cluster {
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "mainnet-beta" => "https://api.mainnet-beta.solana.com",
            _ => "https://api.devnet.solana.com",
        }.to_string();

        Ok(Self { signing_key, address, cluster: cluster.to_string(), rpc_url })
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [self.address]
        });

        let resp: RpcResponse<BalanceResult> = reqwest::Client::new()
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.result.map(|r| r.value).unwrap_or(0))
    }

    pub async fn request_airdrop(&self, lamports: u64) -> Result<String> {
        if self.cluster != "devnet" {
            return Err(anyhow::anyhow!("Airdrop only on devnet"));
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "requestAirdrop",
            "params": [self.address, lamports]
        });

        let resp: RpcResponse<String> = reqwest::Client::new()
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        resp.result.ok_or_else(|| anyhow::anyhow!("Airdrop failed"))
    }

    pub async fn send(&self, _to: &str, _lamports: u64) -> Result<String> {
        // Full implementation requires transaction building
        Err(anyhow::anyhow!("Use Solana SDK for full transaction support"))
    }

    pub fn get_private_key(&self) -> String {
        bs58::encode(self.signing_key.as_bytes()).into_string()
    }

    pub fn explorer_url(&self) -> String {
        match self.cluster.as_str() {
            "mainnet-beta" => format!("https://explorer.solana.com/address/{}", self.address),
            _ => format!("https://explorer.solana.com/address/{}?cluster={}", self.address, self.cluster),
        }
    }
}
