//! TON (The Open Network) broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct TonBroadcaster {
    client: Client,
    api_urls: Vec<String>,
}

impl TonBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let api_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://toncenter.com/api/v2".to_string(),
                "https://ton.blockchair.com/api/v1".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://testnet.toncenter.com/api/v2".to_string(),
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
impl TransactionBroadcaster for TonBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let boc = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signed_tx);
        
        for api_url in &self.api_urls {
            let url = format!("{}/sendBoc", api_url);
            let request = serde_json::json!({ "boc": boc });
            
            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if data.get("ok").and_then(|o| o.as_bool()) == Some(true) {
                            // TON doesn't return tx hash immediately, compute from BOC
                            let hash = format!("{:x}", md5::compute(signed_tx));
                            return Ok(BroadcastResponse::new(hash, "ton", "toncenter")
                                .with_confirmation_time(5));
                        }
                        if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
                            return Err(BroadcastError::Rejected(error.to_string()));
                        }
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        // TON transaction lookup by hash is complex, simplified here
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
        // TON: ~0.01 TON for simple transfer
        Ok(10_000_000) // 0.01 TON in nanotons
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        for api_url in &self.api_urls {
            let url = format!("{}/getMasterchainInfo", api_url);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if data.get("ok").and_then(|o| o.as_bool()) == Some(true) {
                            let height = data.get("result")
                                .and_then(|r| r.get("last"))
                                .and_then(|l| l.get("seqno"))
                                .and_then(|s| s.as_u64())
                                .unwrap_or(0);

                            return Ok(NetworkStatus {
                                is_healthy: true,
                                block_height: height,
                                avg_block_time: std::time::Duration::from_secs(5),
                                mempool_size: None,
                                suggested_fee: 10_000_000,
                            });
                        }
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
