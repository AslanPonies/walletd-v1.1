//! Solana transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

/// Solana transaction broadcaster
pub struct SolanaBroadcaster {
    client: Client,
    rpc_urls: Vec<String>,
}

impl SolanaBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let mut rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://api.mainnet-beta.solana.com".to_string(),
                "https://solana-mainnet.rpc.extrnode.com".to_string(),
                "https://rpc.ankr.com/solana".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://api.devnet.solana.com".to_string(),
                "https://rpc.ankr.com/solana_devnet".to_string(),
            ],
        };

        if let Some(endpoints) = config.custom_endpoints.get("solana") {
            rpc_urls.extend(endpoints.clone());
        }

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            rpc_urls,
        }
    }

    async fn rpc_call<T: for<'de> Deserialize<'de>>(
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
                        if let Some(error) = rpc_response.error {
                            return Err(BroadcastError::Rejected(error.message));
                        }
                    }
                }
            }
        }
        Err(BroadcastError::AllProvidersFailed)
    }
}

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Deserialize)]
struct SignatureStatus {
    slot: Option<u64>,
    confirmations: Option<u64>,
    err: Option<serde_json::Value>,
    #[serde(rename = "confirmationStatus")]
    confirmation_status: Option<String>,
}

#[async_trait]
impl TransactionBroadcaster for SolanaBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signed_tx);
        
        let signature: String = self
            .rpc_call(
                "sendTransaction",
                serde_json::json!([tx_base64, {"encoding": "base64"}]),
            )
            .await?;

        Ok(BroadcastResponse::new(signature, "solana", "rpc")
            .with_confirmation_time(1))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let statuses: serde_json::Value = self
            .rpc_call(
                "getSignatureStatuses",
                serde_json::json!([[tx_hash]]),
            )
            .await?;

        if let Some(value) = statuses.get("value").and_then(|v| v.get(0)) {
            if value.is_null() {
                return Ok(TransactionStatus {
                    tx_hash: tx_hash.to_string(),
                    status: TxStatus::Unknown,
                    confirmations: 0,
                    block_number: None,
                    block_hash: None,
                    timestamp: None,
                    fee: None,
                    gas_used: None,
                });
            }

            let slot = value.get("slot").and_then(|s| s.as_u64());
            let confirmations = value.get("confirmations").and_then(|c| c.as_u64()).unwrap_or(0);
            let has_error = value.get("err").map(|e| !e.is_null()).unwrap_or(false);
            let conf_status = value.get("confirmationStatus").and_then(|s| s.as_str());

            let status = if has_error {
                TxStatus::Failed
            } else if conf_status == Some("finalized") {
                TxStatus::Confirmed
            } else {
                TxStatus::Pending
            };

            return Ok(TransactionStatus {
                tx_hash: tx_hash.to_string(),
                status,
                confirmations,
                block_number: slot,
                block_hash: None,
                timestamp: None,
                fee: None,
                gas_used: None,
            });
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
        // Solana base fee is 5000 lamports per signature
        Ok(5000)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let slot: u64 = self
            .rpc_call("getSlot", serde_json::json!([]))
            .await?;

        Ok(NetworkStatus {
            is_healthy: true,
            block_height: slot,
            avg_block_time: std::time::Duration::from_millis(400),
            mempool_size: None,
            suggested_fee: 5000,
        })
    }
}
