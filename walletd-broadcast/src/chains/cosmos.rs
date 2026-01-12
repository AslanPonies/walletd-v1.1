//! Cosmos SDK chain broadcaster

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

pub struct CosmosBroadcaster {
    client: Client,
    lcd_urls: Vec<String>,
    chain_id: String,
}

impl CosmosBroadcaster {
    pub fn new(config: &BroadcastConfig) -> Self {
        let (lcd_urls, chain_id) = match config.network {
            NetworkMode::Mainnet => (
                vec![
                    "https://cosmos-rest.publicnode.com".to_string(),
                    "https://rest.cosmos.directory/cosmoshub".to_string(),
                    "https://lcd-cosmoshub.keplr.app".to_string(),
                ],
                "cosmoshub-4".to_string(),
            ),
            NetworkMode::Testnet => (
                vec!["https://rest.testnet.cosmos.network".to_string()],
                "theta-testnet-001".to_string(),
            ),
        };

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            lcd_urls,
            chain_id,
        }
    }
}

#[async_trait]
impl TransactionBroadcaster for CosmosBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_bytes = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signed_tx);
        let request = serde_json::json!({
            "tx_bytes": tx_bytes,
            "mode": "BROADCAST_MODE_SYNC"
        });

        for lcd_url in &self.lcd_urls {
            let url = format!("{}/cosmos/tx/v1beta1/txs", lcd_url);
            if let Ok(response) = self.client.post(&url).json(&request).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if let Some(tx_hash) = data.get("tx_response")
                            .and_then(|r| r.get("txhash"))
                            .and_then(|h| h.as_str())
                        {
                            return Ok(BroadcastResponse::new(tx_hash.to_string(), "cosmos", "lcd")
                                .with_confirmation_time(7));
                        }
                        if let Some(error) = data.get("tx_response")
                            .and_then(|r| r.get("raw_log"))
                            .and_then(|l| l.as_str())
                        {
                            return Err(BroadcastError::Rejected(error.to_string()));
                        }
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        for lcd_url in &self.lcd_urls {
            let url = format!("{}/cosmos/tx/v1beta1/txs/{}", lcd_url, tx_hash);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let code = data.get("tx_response")
                            .and_then(|r| r.get("code"))
                            .and_then(|c| c.as_u64())
                            .unwrap_or(0);
                        
                        let status = if code == 0 { TxStatus::Confirmed } else { TxStatus::Failed };
                        
                        return Ok(TransactionStatus {
                            tx_hash: tx_hash.to_string(),
                            status,
                            confirmations: 1,
                            block_number: data.get("tx_response")
                                .and_then(|r| r.get("height"))
                                .and_then(|h| h.as_str())
                                .and_then(|s| s.parse().ok()),
                            block_hash: None,
                            timestamp: data.get("tx_response")
                                .and_then(|r| r.get("timestamp"))
                                .and_then(|t| t.as_str())
                                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.timestamp() as u64),
                            fee: None,
                            gas_used: data.get("tx_response")
                                .and_then(|r| r.get("gas_used"))
                                .and_then(|g| g.as_str())
                                .and_then(|s| s.parse().ok()),
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
        // Default Cosmos Hub fee: ~0.025 ATOM
        Ok(25_000)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        for lcd_url in &self.lcd_urls {
            let url = format!("{}/cosmos/base/tendermint/v1beta1/blocks/latest", lcd_url);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let height = data.get("block")
                            .and_then(|b| b.get("header"))
                            .and_then(|h| h.get("height"))
                            .and_then(|h| h.as_str())
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);

                        return Ok(NetworkStatus {
                            is_healthy: true,
                            block_height: height,
                            avg_block_time: std::time::Duration::from_secs(7),
                            mempool_size: None,
                            suggested_fee: 25_000,
                        });
                    }
                }
            }
        }

        Err(BroadcastError::AllProvidersFailed)
    }
}
