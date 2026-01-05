//! Monero transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct MoneroBroadcaster {
    client: Client,
    daemon_urls: Vec<String>,
}

impl MoneroBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let daemon_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://node.moneroworld.com:18089".to_string(),
                "https://xmr-node.cakewallet.com:18081".to_string(),
                "https://nodes.hashvault.pro:18081".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://stagenet.xmr-tw.org:38081".to_string(),
            ],
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            daemon_urls,
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
            "id": "0"
        });

        for daemon_url in &self.daemon_urls {
            let url = format!("{}/json_rpc", daemon_url);
            if let Ok(response) = self.client.post(&url).json(&request).send().await {
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

#[derive(Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
}

#[derive(Deserialize)]
struct SubmitResult {
    status: String,
}

#[async_trait]
impl TransactionBroadcaster for MoneroBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = hex::encode(signed_tx);
        
        for daemon_url in &self.daemon_urls {
            let url = format!("{}/sendrawtransaction", daemon_url);
            let request = serde_json::json!({
                "tx_as_hex": tx_hex,
                "do_not_relay": false
            });

            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(result) = response.json::<serde_json::Value>().await {
                        if result.get("status").and_then(|s| s.as_str()) == Some("OK") {
                            // Calculate tx hash from the raw transaction
                            let tx_hash = format!("{:064x}", md5::compute(signed_tx).0.iter().fold(0u128, |acc, &b| acc << 8 | b as u128));
                            return Ok(BroadcastResponse::new(tx_hash, "monero", "daemon")
                                .with_confirmation_time(120));
                        }
                        if let Some(reason) = result.get("reason").and_then(|r| r.as_str()) {
                            return Err(BroadcastError::Rejected(reason.to_string()));
                        }
                    }
                }
            }
        }
        
        Err(BroadcastError::AllProvidersFailed)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        for daemon_url in &self.daemon_urls {
            let url = format!("{}/get_transactions", daemon_url);
            let request = serde_json::json!({
                "txs_hashes": [tx_hash],
                "decode_as_json": true
            });

            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let in_pool = data.get("txs").and_then(|t| t.get(0))
                            .and_then(|t| t.get("in_pool")).and_then(|p| p.as_bool()).unwrap_or(false);
                        let block_height = data.get("txs").and_then(|t| t.get(0))
                            .and_then(|t| t.get("block_height")).and_then(|h| h.as_u64());

                        let status = if in_pool {
                            TxStatus::Pending
                        } else if block_height.is_some() {
                            TxStatus::Confirmed
                        } else {
                            TxStatus::Unknown
                        };

                        return Ok(TransactionStatus {
                            tx_hash: tx_hash.to_string(),
                            status,
                            confirmations: if status == TxStatus::Confirmed { 1 } else { 0 },
                            block_number: block_height,
                            block_hash: None,
                            timestamp: None,
                            fee: None,
                            gas_used: None,
                        });
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

    async fn estimate_fee(&self, tx_size: usize) -> BroadcastResult<u64> {
        // Monero fee is per byte, ~0.00002 XMR per kB
        Ok((tx_size as u64 / 1024 + 1) * 20_000_000) // In atomic units
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        #[derive(Deserialize)]
        struct InfoResult {
            height: u64,
            tx_pool_size: u64,
        }

        let info: InfoResult = self.rpc_call("get_info", serde_json::json!({})).await?;

        Ok(NetworkStatus {
            is_healthy: true,
            block_height: info.height,
            avg_block_time: std::time::Duration::from_secs(120),
            mempool_size: Some(info.tx_pool_size),
            suggested_fee: 20_000_000,
        })
    }
}
