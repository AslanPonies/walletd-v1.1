//! NEAR Protocol broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct NearBroadcaster {
    client: Client,
    rpc_urls: Vec<String>,
}

impl NearBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://rpc.mainnet.near.org".to_string(),
                "https://near-mainnet.api.pagoda.co/rpc/v1".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://rpc.testnet.near.org".to_string(),
            ],
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            rpc_urls,
        }
    }

    async fn rpc_call<T: for<'de> serde::Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> BroadcastResult<T> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": "walletd"
        });

        for rpc_url in &self.rpc_urls {
            if let Ok(response) = self.client.post(rpc_url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(rpc_response) = response.json::<RpcResponse<T>>().await {
                        if let Some(result) = rpc_response.result {
                            return Ok(result);
                        }
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}

#[derive(serde::Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
}

#[async_trait]
impl TransactionBroadcaster for NearBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signed_tx);
        
        let result: serde_json::Value = self
            .rpc_call("broadcast_tx_commit", serde_json::json!([tx_base64]))
            .await?;

        if let Some(tx_hash) = result.get("transaction").and_then(|t| t.get("hash")).and_then(|h| h.as_str()) {
            return Ok(BroadcastResponse::new(tx_hash.to_string(), "near", "rpc")
                .with_confirmation_time(2));
        }

        Err(BroadcastError::Rejected("No transaction hash returned".into()))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let result: serde_json::Value = self
            .rpc_call("tx", serde_json::json!([tx_hash, "system"]))
            .await
            .unwrap_or_default();

        let status = if result.get("status").is_some() {
            TxStatus::Confirmed
        } else {
            TxStatus::Unknown
        };

        Ok(TransactionStatus {
            tx_hash: tx_hash.to_string(),
            status,
            confirmations: if status == TxStatus::Confirmed { 1 } else { 0 },
            block_number: None,
            block_hash: result.get("transaction_outcome")
                .and_then(|o| o.get("block_hash"))
                .and_then(|h| h.as_str())
                .map(String::from),
            timestamp: None,
            fee: None,
            gas_used: None,
        })
    }

    async fn estimate_fee(&self, _tx_size: usize) -> BroadcastResult<u64> {
        // NEAR: ~0.0001 NEAR per action
        Ok(100_000_000_000_000_u64) // 0.0001 NEAR in yoctoNEAR
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let status: serde_json::Value = self
            .rpc_call("status", serde_json::json!([]))
            .await?;

        let height = status.get("sync_info")
            .and_then(|s| s.get("latest_block_height"))
            .and_then(|h| h.as_u64())
            .unwrap_or(0);

        Ok(NetworkStatus {
            is_healthy: true,
            block_height: height,
            avg_block_time: std::time::Duration::from_secs(1),
            mempool_size: None,
            suggested_fee: 100_000_000_000_000_u64,
        })
    }
}
