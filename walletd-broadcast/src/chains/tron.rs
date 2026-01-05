//! TRON network broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct TronBroadcaster {
    client: Client,
    api_urls: Vec<String>,
}

impl TronBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let api_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://api.trongrid.io".to_string(),
                "https://api.shasta.trongrid.io".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://api.shasta.trongrid.io".to_string(),
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
impl TransactionBroadcaster for TronBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = hex::encode(signed_tx);
        
        for api_url in &self.api_urls {
            let url = format!("{}/wallet/broadcasthex", api_url);
            let request = serde_json::json!({ "transaction": tx_hex });
            
            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if data.get("result").and_then(|r| r.as_bool()) == Some(true) {
                            if let Some(txid) = data.get("txid").and_then(|t| t.as_str()) {
                                return Ok(BroadcastResponse::new(txid.to_string(), "tron", "trongrid")
                                    .with_confirmation_time(3));
                            }
                        }
                        if let Some(message) = data.get("message").and_then(|m| m.as_str()) {
                            return Err(BroadcastError::Rejected(message.to_string()));
                        }
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        for api_url in &self.api_urls {
            let url = format!("{}/wallet/gettransactionbyid", api_url);
            let request = serde_json::json!({ "value": tx_hash });
            
            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if data.get("txID").is_some() {
                            let result = data.get("ret").and_then(|r| r.get(0))
                                .and_then(|r| r.get("contractRet"))
                                .and_then(|c| c.as_str());
                            
                            let status = match result {
                                Some("SUCCESS") => TxStatus::Confirmed,
                                Some(_) => TxStatus::Failed,
                                None => TxStatus::Pending,
                            };

                            return Ok(TransactionStatus {
                                tx_hash: tx_hash.to_string(),
                                status,
                                confirmations: if status == TxStatus::Confirmed { 1 } else { 0 },
                                block_number: data.get("blockNumber").and_then(|b| b.as_u64()),
                                block_hash: None,
                                timestamp: data.get("blockTimeStamp").and_then(|t| t.as_u64()),
                                fee: None,
                                gas_used: None,
                            });
                        }
                    }
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
        // TRON uses bandwidth/energy model
        Ok(1_000_000) // ~1 TRX
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        for api_url in &self.api_urls {
            let url = format!("{}/wallet/getnowblock", api_url);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let height = data.get("block_header")
                            .and_then(|h| h.get("raw_data"))
                            .and_then(|r| r.get("number"))
                            .and_then(|n| n.as_u64())
                            .unwrap_or(0);

                        return Ok(NetworkStatus {
                            is_healthy: true,
                            block_height: height,
                            avg_block_time: std::time::Duration::from_secs(3),
                            mempool_size: None,
                            suggested_fee: 1_000_000,
                        });
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
