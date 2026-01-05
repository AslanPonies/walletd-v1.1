//! Internet Computer (ICP) transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct IcpBroadcaster {
    client: Client,
    boundary_nodes: Vec<String>,
    rosetta_url: String,
}

impl IcpBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let (boundary_nodes, rosetta_url) = match config.network {
            NetworkMode::Mainnet => (
                vec![
                    "https://ic0.app".to_string(),
                    "https://icp-api.io".to_string(),
                    "https://icp0.io".to_string(),
                ],
                "https://rosetta-api.internetcomputer.org".to_string(),
            ),
            NetworkMode::Testnet => (
                vec!["https://ic0.app".to_string()],
                "https://rosetta-api.internetcomputer.org".to_string(),
            ),
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            boundary_nodes,
            rosetta_url,
        }
    }

    pub async fn get_account_balance(&self, account_id: &str) -> BroadcastResult<u64> {
        let request = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            },
            "account_identifier": {
                "address": account_id
            }
        });

        let url = format!("{}/account/balance", self.rosetta_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(balance) = data.get("balances")
                .and_then(|b| b.get(0))
                .and_then(|b| b.get("value"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
            {
                return Ok(balance);
            }
        }
        Err(BroadcastError::Unknown("Failed to get balance".into()))
    }
}

#[async_trait]
impl TransactionBroadcaster for IcpBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let request = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            },
            "signed_transaction": hex::encode(signed_tx)
        });

        let url = format!("{}/construction/submit", self.rosetta_url);
        let response = self.client.post(&url).json(&request).send().await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(hash) = data.get("transaction_identifier")
                .and_then(|t| t.get("hash"))
                .and_then(|h| h.as_str())
            {
                return Ok(BroadcastResponse::new(hash.to_string(), "icp", "rosetta")
                    .with_confirmation_time(2));
            }
        }

        let error = response.text().await.unwrap_or_default();
        Err(BroadcastError::Rejected(error))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let request = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            },
            "transaction_identifier": {
                "hash": tx_hash
            }
        });

        let url = format!("{}/search/transactions", self.rosetta_url);
        if let Ok(response) = self.client.post(&url).json(&request).send().await {
            if response.status().is_success() {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    let found = data.get("transactions")
                        .and_then(|t| t.as_array())
                        .map(|a| !a.is_empty())
                        .unwrap_or(false);

                    if found {
                        return Ok(TransactionStatus {
                            tx_hash: tx_hash.to_string(),
                            status: TxStatus::Confirmed,
                            confirmations: 1,
                            block_number: None,
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
            status: TxStatus::Pending,
            confirmations: 0,
            block_number: None,
            block_hash: None,
            timestamp: None,
            fee: None,
            gas_used: None,
        })
    }

    async fn estimate_fee(&self, _tx_size: usize) -> BroadcastResult<u64> {
        // ICP transfer fee: 10,000 e8s (0.0001 ICP)
        Ok(10_000)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let request = serde_json::json!({
            "network_identifier": {
                "blockchain": "Internet Computer",
                "network": "00000000000000020101"
            }
        });

        let url = format!("{}/network/status", self.rosetta_url);
        if let Ok(response) = self.client.post(&url).json(&request).send().await {
            if response.status().is_success() {
                if let Ok(data) = response.json::<serde_json::Value>().await {
                    let height = data.get("current_block_identifier")
                        .and_then(|b| b.get("index"))
                        .and_then(|i| i.as_u64())
                        .unwrap_or(0);

                    return Ok(NetworkStatus {
                        is_healthy: true,
                        block_height: height,
                        avg_block_time: std::time::Duration::from_secs(1),
                        mempool_size: None,
                        suggested_fee: 10_000,
                    });
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
