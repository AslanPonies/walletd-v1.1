//! SUI blockchain broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct SuiBroadcaster {
    client: Client,
    rpc_urls: Vec<String>,
}

impl SuiBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://fullnode.mainnet.sui.io:443".to_string(),
                "https://sui-mainnet.public.blastapi.io".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://fullnode.testnet.sui.io:443".to_string(),
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
            "id": 1
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
impl TransactionBroadcaster for SuiBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signed_tx);
        
        let result: serde_json::Value = self
            .rpc_call("sui_executeTransactionBlock", serde_json::json!([
                tx_base64,
                [],
                {"showEffects": true}
            ]))
            .await?;

        if let Some(digest) = result.get("digest").and_then(|d| d.as_str()) {
            return Ok(BroadcastResponse::new(digest.to_string(), "sui", "rpc")
                .with_confirmation_time(1));
        }

        Err(BroadcastError::Rejected("No digest returned".into()))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let result: serde_json::Value = self
            .rpc_call("sui_getTransactionBlock", serde_json::json!([tx_hash]))
            .await
            .unwrap_or_default();

        let status = if result.get("digest").is_some() {
            let success = result.get("effects")
                .and_then(|e| e.get("status"))
                .and_then(|s| s.get("status"))
                .and_then(|s| s.as_str()) == Some("success");
            
            if success { TxStatus::Confirmed } else { TxStatus::Failed }
        } else {
            TxStatus::Unknown
        };

        Ok(TransactionStatus {
            tx_hash: tx_hash.to_string(),
            status,
            confirmations: if status == TxStatus::Confirmed { 1 } else { 0 },
            block_number: result.get("checkpoint").and_then(|c| c.as_str()).and_then(|s| s.parse().ok()),
            block_hash: None,
            timestamp: result.get("timestampMs").and_then(|t| t.as_str()).and_then(|s| s.parse::<u64>().ok()).map(|ms| ms / 1000),
            fee: None,
            gas_used: result.get("effects").and_then(|e| e.get("gasUsed")).and_then(|g| g.get("computationCost")).and_then(|c| c.as_str()).and_then(|s| s.parse().ok()),
        })
    }

    async fn estimate_fee(&self, _tx_size: usize) -> BroadcastResult<u64> {
        Ok(1_000_000) // ~0.001 SUI
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let checkpoint: serde_json::Value = self
            .rpc_call("sui_getLatestCheckpointSequenceNumber", serde_json::json!([]))
            .await?;

        let height = checkpoint.as_str().and_then(|s| s.parse().ok()).unwrap_or(0);

        Ok(NetworkStatus {
            is_healthy: true,
            block_height: height,
            avg_block_time: std::time::Duration::from_millis(500),
            mempool_size: None,
            suggested_fee: 1_000_000,
        })
    }
}
