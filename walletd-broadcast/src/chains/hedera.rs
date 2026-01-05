//! Hedera transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct HederaBroadcaster {
    client: Client,
    mirror_url: String,
    grpc_endpoints: Vec<String>,
}

impl HederaBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let (mirror_url, grpc_endpoints) = match config.network {
            NetworkMode::Mainnet => (
                "https://mainnet-public.mirrornode.hedera.com".to_string(),
                vec![
                    "mainnet.hedera.com:50211".to_string(),
                    "35.237.200.180:50211".to_string(),
                ],
            ),
            NetworkMode::Testnet => (
                "https://testnet.mirrornode.hedera.com".to_string(),
                vec![
                    "testnet.hedera.com:50211".to_string(),
                    "34.94.106.61:50211".to_string(),
                ],
            ),
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            mirror_url,
            grpc_endpoints,
        }
    }

    pub async fn get_account_balance(&self, account_id: &str) -> BroadcastResult<u64> {
        let url = format!("{}/api/v1/accounts/{}", self.mirror_url, account_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(balance) = data.get("balance").and_then(|b| b.get("balance")).and_then(|b| b.as_u64()) {
                return Ok(balance);
            }
        }
        Err(BroadcastError::Unknown("Failed to get balance".into()))
    }
}

#[async_trait]
impl TransactionBroadcaster for HederaBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        // Hedera requires gRPC for transaction submission
        // This is a placeholder - real implementation would use hedera-sdk
        let tx_hash = format!("0.0.{}", hex::encode(&signed_tx[..8]));
        
        Ok(BroadcastResponse::new(tx_hash, "hedera", "grpc")
            .with_confirmation_time(5))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let url = format!("{}/api/v1/transactions/{}", self.mirror_url, tx_hash);
        
        if let Ok(response) = self.client.get(&url).send().await {
            if response.status().is_success() {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    let result = data.get("result").and_then(|r| r.as_str()).unwrap_or("");
                    let status = if result == "SUCCESS" {
                        TxStatus::Confirmed
                    } else if result.contains("FAIL") {
                        TxStatus::Failed
                    } else {
                        TxStatus::Pending
                    };

                    return Ok(TransactionStatus {
                        tx_hash: tx_hash.to_string(),
                        status,
                        confirmations: 1,
                        block_number: None,
                        block_hash: None,
                        timestamp: data.get("consensus_timestamp").and_then(|t| {
                            t.as_str().and_then(|s| s.parse::<f64>().ok()).map(|f| f as u64)
                        }),
                        fee: data.get("charged_tx_fee").and_then(|f| f.as_u64()),
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
        // Hedera base fee: ~$0.0001 USD = ~1000 tinybars
        Ok(100_000)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let url = format!("{}/api/v1/network/nodes", self.mirror_url);
        
        if let Ok(response) = self.client.get(&url).send().await {
            if response.status().is_success() {
                return Ok(NetworkStatus {
                    is_healthy: true,
                    block_height: 0,
                    avg_block_time: std::time::Duration::from_secs(3),
                    mempool_size: None,
                    suggested_fee: 100_000,
                });
            }
        }
        
        Err(BroadcastError::AllProvidersFailed)
    }
}
