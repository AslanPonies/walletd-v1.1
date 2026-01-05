//! Substrate-based chain broadcaster (Polkadot, Kusama, etc.)

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct SubstrateBroadcaster {
    client: Client,
    rpc_urls: Vec<String>,
    chain_name: String,
}

impl SubstrateBroadcaster {
    pub fn new_polkadot(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://polkadot-rpc.dwellir.com".to_string(),
                "https://rpc.polkadot.io".to_string(),
                "https://polkadot.api.onfinality.io/public".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://westend-rpc.polkadot.io".to_string(),
            ],
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            rpc_urls,
            chain_name: "polkadot".to_string(),
        }
    }

    pub fn new_kusama(config: &BroadcastConfig) -> Self {
        let rpc_urls = vec![
            "https://kusama-rpc.polkadot.io".to_string(),
            "https://kusama-rpc.dwellir.com".to_string(),
        ];

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            rpc_urls,
            chain_name: "kusama".to_string(),
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

#[derive(serde::Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(serde::Deserialize)]
struct RpcError {
    message: String,
}

#[async_trait]
impl TransactionBroadcaster for SubstrateBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = format!("0x{}", hex::encode(signed_tx));
        
        let tx_hash: String = self
            .rpc_call("author_submitExtrinsic", serde_json::json!([tx_hex]))
            .await?;

        Ok(BroadcastResponse::new(tx_hash, &self.chain_name, "rpc")
            .with_confirmation_time(6))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        // Query events for this extrinsic
        // This is simplified - real implementation would query storage
        
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
        // Polkadot fee: ~0.015 DOT base
        Ok(15_000_000_000 + (tx_size as u64 * 1_000_000))
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let header: serde_json::Value = self
            .rpc_call("chain_getHeader", serde_json::json!([]))
            .await?;

        let height = header.get("number")
            .and_then(|n| n.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(0);

        Ok(NetworkStatus {
            is_healthy: true,
            block_height: height,
            avg_block_time: std::time::Duration::from_secs(6),
            mempool_size: None,
            suggested_fee: 15_000_000_000,
        })
    }
}
