//! Aptos blockchain broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct AptosBroadcaster {
    client: Client,
    api_urls: Vec<String>,
}

impl AptosBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let api_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://fullnode.mainnet.aptoslabs.com/v1".to_string(),
                "https://aptos-mainnet.public.blastapi.io/v1".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://fullnode.testnet.aptoslabs.com/v1".to_string(),
            ],
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            api_urls,
        }
    }
}

#[async_trait]
impl TransactionBroadcaster for AptosBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        for api_url in &self.api_urls {
            let url = format!("{}/transactions", api_url);
            
            if let Ok(response) = self.client
                .post(&url)
                .header("Content-Type", "application/x.aptos.signed_transaction+bcs")
                .body(signed_tx.to_vec())
                .send()
                .await
            {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if let Some(hash) = data.get("hash").and_then(|h| h.as_str()) {
                            return Ok(BroadcastResponse::new(hash.to_string(), "aptos", "rest")
                                .with_confirmation_time(1));
                        }
                    }
                } else {
                    let error = response.text().await.unwrap_or_default();
                    return Err(BroadcastError::Rejected(error));
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        for api_url in &self.api_urls {
            let url = format!("{}/transactions/by_hash/{}", api_url, tx_hash);
            
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let success = data.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                        let status = if success { TxStatus::Confirmed } else { TxStatus::Failed };

                        return Ok(TransactionStatus {
                            tx_hash: tx_hash.to_string(),
                            status,
                            confirmations: 1,
                            block_number: data.get("version").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()),
                            block_hash: None,
                            timestamp: data.get("timestamp").and_then(|t| t.as_str()).and_then(|s| s.parse::<u64>().ok()).map(|us| us / 1_000_000),
                            fee: data.get("gas_used").and_then(|g| g.as_str()).and_then(|s| s.parse().ok()),
                            gas_used: data.get("gas_used").and_then(|g| g.as_str()).and_then(|s| s.parse().ok()),
                        });
                    }
                } else if response.status().as_u16() == 404 {
                    return Ok(TransactionStatus {
                        tx_hash: tx_hash.to_string(),
                        status: TxStatus::Pending,
                        confirmations: 0,
                        block_number: None,
                        block_hash: None,
                        timestamp: None,
                        fee: None,
                        gas_used: None,
                    });
                }
            }
        }

        Ok(TransactionStatus {
            tx_hash: tx_hash.to_string(),
            status: TxStatus::Unknown,
            confirmations: 0,
            block_number: None,
            block_hash: None,
            timestamp: None,
            fee: None,
            gas_used: None,
        })
    }

    async fn estimate_fee(&self, _tx_size: usize) -> BroadcastResult<u64> {
        // Aptos: ~100 gas units at 100 octas/gas = 10000 octas
        Ok(100_000) // 0.001 APT
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        for api_url in &self.api_urls {
            let url = format!("{}/", api_url);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let height = data.get("block_height").and_then(|h| h.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);

                        return Ok(NetworkStatus {
                            is_healthy: true,
                            block_height: height,
                            avg_block_time: std::time::Duration::from_millis(500),
                            mempool_size: None,
                            suggested_fee: 100_000,
                        });
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
