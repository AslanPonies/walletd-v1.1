//! Cardano transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct CardanoBroadcaster {
    client: Client,
    submit_api: String,
    blockfrost_url: String,
    api_key: Option<String>,
}

impl CardanoBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let (submit_api, blockfrost_url) = match config.network {
            NetworkMode::Mainnet => (
                "https://submit-api.mainnet.dandelion.link".to_string(),
                "https://cardano-mainnet.blockfrost.io/api/v0".to_string(),
            ),
            NetworkMode::Testnet => (
                "https://submit-api.preprod.dandelion.link".to_string(),
                "https://cardano-preprod.blockfrost.io/api/v0".to_string(),
            ),
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            submit_api,
            blockfrost_url,
            api_key: None,
        }
    }
}

#[async_trait]
impl TransactionBroadcaster for CardanoBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let response = self.client
            .post(&format!("{}/tx/submit", self.submit_api))
            .header("Content-Type", "application/cbor")
            .body(signed_tx.to_vec())
            .send()
            .await?;

        if response.status().is_success() {
            let tx_hash = response.text().await?.trim_matches('"').to_string();
            return Ok(BroadcastResponse::new(tx_hash, "cardano", "dandelion")
                .with_confirmation_time(20));
        }

        let error = response.text().await.unwrap_or_default();
        Err(BroadcastError::Rejected(error))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let mut request = self.client.get(&format!("{}/txs/{}", self.blockfrost_url, tx_hash));
        if let Some(ref key) = self.api_key {
            request = request.header("project_id", key);
        }

        if let Ok(response) = request.send().await {
            if response.status().is_success() {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    return Ok(TransactionStatus {
                        tx_hash: tx_hash.to_string(),
                        status: TxStatus::Confirmed,
                        confirmations: data.get("block_height").and_then(|h| h.as_u64()).unwrap_or(1),
                        block_number: data.get("block_height").and_then(|h| h.as_u64()),
                        block_hash: data.get("block").and_then(|b| b.as_str()).map(String::from),
                        timestamp: data.get("block_time").and_then(|t| t.as_u64()),
                        fee: data.get("fees").and_then(|f| f.as_str()).and_then(|s| s.parse().ok()),
                        gas_used: None,
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

    async fn estimate_fee(&self, tx_size: usize) -> BroadcastResult<u64> {
        // Cardano: ~0.17 ADA base + 0.000044 per byte
        Ok(170_000 + (tx_size as u64 * 44))
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let mut request = self.client.get(&format!("{}/blocks/latest", self.blockfrost_url));
        if let Some(ref key) = self.api_key {
            request = request.header("project_id", key);
        }

        if let Ok(response) = request.send().await {
            if response.status().is_success() {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    return Ok(NetworkStatus {
                        is_healthy: true,
                        block_height: data.get("height").and_then(|h| h.as_u64()).unwrap_or(0),
                        avg_block_time: std::time::Duration::from_secs(20),
                        mempool_size: None,
                        suggested_fee: 200_000,
                    });
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
