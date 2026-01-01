//! Real Solana Wallet Integration

use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Real Solana wallet
pub struct RealSolanaWallet {
    pub signing_key: SigningKey,
    pub address: String,
    pub cluster: String,
    rpc_url: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct BalanceResult {
    value: u64,
}

impl RealSolanaWallet {
    /// Create new Solana wallet
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

        Ok(Self {
            signing_key,
            address,
            cluster: cluster.to_string(),
            rpc_url,
        })
    }

    /// Get balance in lamports
    pub async fn get_balance(&self) -> Result<u64> {
        let client = reqwest::Client::new();
        
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [self.address]
        });

        let response = client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?;

        let result: RpcResponse<BalanceResult> = response.json().await?;
        
        if let Some(balance) = result.result {
            Ok(balance.value)
        } else if let Some(error) = result.error {
            Err(anyhow::anyhow!("RPC error: {}", error.message))
        } else {
            Ok(0)
        }
    }

    /// Request airdrop (devnet only)
    pub async fn request_airdrop(&self, lamports: u64) -> Result<String> {
        if self.cluster != "devnet" {
            return Err(anyhow::anyhow!("Airdrop only available on devnet"));
        }

        let client = reqwest::Client::new();
        
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "requestAirdrop",
            "params": [self.address, lamports]
        });

        let response = client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?;

        let result: RpcResponse<String> = response.json().await?;
        
        if let Some(signature) = result.result {
            Ok(signature)
        } else if let Some(error) = result.error {
            Err(anyhow::anyhow!("Airdrop failed: {}", error.message))
        } else {
            Err(anyhow::anyhow!("Unknown error"))
        }
    }

    /// Send transaction (simplified - real impl would use proper tx building)
    pub async fn send_transaction(&self, _to_address: &str, _lamports: u64) -> Result<String> {
        // In production, this would build and sign a proper Solana transaction
        Err(anyhow::anyhow!("Transaction sending requires full SDK integration"))
    }

    /// Get private key as base58
    pub fn get_private_key(&self) -> String {
        bs58::encode(self.signing_key.as_bytes()).into_string()
    }

    /// Get explorer URL
    pub fn explorer_url(&self) -> String {
        match self.cluster.as_str() {
            "devnet" => format!("https://explorer.solana.com/address/{}?cluster=devnet", self.address),
            "testnet" => format!("https://explorer.solana.com/address/{}?cluster=testnet", self.address),
            _ => format!("https://explorer.solana.com/address/{}", self.address),
        }
    }
}
