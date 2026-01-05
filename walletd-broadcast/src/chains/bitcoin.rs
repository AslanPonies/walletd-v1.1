//! Bitcoin transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

/// Bitcoin broadcaster with multiple provider fallback
pub struct BitcoinBroadcaster {
    client: Client,
    providers: Vec<BitcoinProvider>,
    network: NetworkMode,
}

#[derive(Clone)]
struct BitcoinProvider {
    name: String,
    broadcast_url: String,
    status_url: String,
    fee_url: String,
}

impl BitcoinBroadcaster {
    /// Create new Bitcoin broadcaster
    pub fn new(config: &BroadcastConfig) -> Self {
        let providers = match config.network {
            NetworkMode::Mainnet => vec![
                BitcoinProvider {
                    name: "Blockstream".to_string(),
                    broadcast_url: "https://blockstream.info/api/tx".to_string(),
                    status_url: "https://blockstream.info/api/tx/".to_string(),
                    fee_url: "https://blockstream.info/api/fee-estimates".to_string(),
                },
                BitcoinProvider {
                    name: "Mempool.space".to_string(),
                    broadcast_url: "https://mempool.space/api/tx".to_string(),
                    status_url: "https://mempool.space/api/tx/".to_string(),
                    fee_url: "https://mempool.space/api/v1/fees/recommended".to_string(),
                },
                BitcoinProvider {
                    name: "Blockchain.info".to_string(),
                    broadcast_url: "https://blockchain.info/pushtx".to_string(),
                    status_url: "https://blockchain.info/rawtx/".to_string(),
                    fee_url: "https://api.blockchain.info/mempool/fees".to_string(),
                },
            ],
            NetworkMode::Testnet => vec![
                BitcoinProvider {
                    name: "Blockstream Testnet".to_string(),
                    broadcast_url: "https://blockstream.info/testnet/api/tx".to_string(),
                    status_url: "https://blockstream.info/testnet/api/tx/".to_string(),
                    fee_url: "https://blockstream.info/testnet/api/fee-estimates".to_string(),
                },
                BitcoinProvider {
                    name: "Mempool.space Testnet".to_string(),
                    broadcast_url: "https://mempool.space/testnet/api/tx".to_string(),
                    status_url: "https://mempool.space/testnet/api/tx/".to_string(),
                    fee_url: "https://mempool.space/testnet/api/v1/fees/recommended".to_string(),
                },
            ],
        };

        // Add custom endpoints if configured
        let mut all_providers = providers;
        if let Some(endpoints) = config.custom_endpoints.get("bitcoin") {
            for endpoint in endpoints {
                all_providers.push(BitcoinProvider {
                    name: "Custom".to_string(),
                    broadcast_url: format!("{}/tx", endpoint),
                    status_url: format!("{}/tx/", endpoint),
                    fee_url: format!("{}/fee-estimates", endpoint),
                });
            }
        }

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            providers: all_providers,
            network: config.network,
        }
    }

    /// Broadcast via Blockstream API
    async fn broadcast_blockstream(&self, provider: &BitcoinProvider, tx_hex: &str) -> BroadcastResult<String> {
        let response = self
            .client
            .post(&provider.broadcast_url)
            .body(tx_hex.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            let tx_hash = response.text().await?;
            Ok(tx_hash.trim().to_string())
        } else {
            let error = response.text().await.unwrap_or_default();
            Err(BroadcastError::Rejected(error))
        }
    }

    /// Get fee estimates
    pub async fn get_fee_estimates(&self) -> BroadcastResult<FeeEstimates> {
        for provider in &self.providers {
            if let Ok(response) = self.client.get(&provider.fee_url).send().await {
                if response.status().is_success() {
                    if let Ok(fees) = response.json::<serde_json::Value>().await {
                        // Parse different API formats
                        let fast = fees.get("fastestFee")
                            .or_else(|| fees.get("1"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(50);
                        let medium = fees.get("halfHourFee")
                            .or_else(|| fees.get("6"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(25);
                        let slow = fees.get("hourFee")
                            .or_else(|| fees.get("144"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(10);

                        return Ok(FeeEstimates { fast, medium, slow });
                    }
                }
            }
        }
        
        // Default fees if all providers fail
        Ok(FeeEstimates { fast: 50, medium: 25, slow: 10 })
    }
}

#[derive(Debug, Clone)]
pub struct FeeEstimates {
    pub fast: u64,    // sat/vB
    pub medium: u64,
    pub slow: u64,
}

#[derive(Debug, Deserialize)]
struct BlockstreamTxStatus {
    txid: String,
    status: BlockstreamStatus,
    fee: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct BlockstreamStatus {
    confirmed: bool,
    block_height: Option<u64>,
    block_hash: Option<String>,
    block_time: Option<u64>,
}

#[async_trait]
impl TransactionBroadcaster for BitcoinBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = hex::encode(signed_tx);
        
        let mut last_error = BroadcastError::AllProvidersFailed;
        
        for provider in &self.providers {
            match self.broadcast_blockstream(provider, &tx_hex).await {
                Ok(tx_hash) => {
                    tracing::info!("Bitcoin tx broadcast via {}: {}", provider.name, tx_hash);
                    return Ok(BroadcastResponse::new(tx_hash, "bitcoin", &provider.name)
                        .with_confirmation_time(600)); // ~10 minutes
                }
                Err(e) => {
                    tracing::warn!("Provider {} failed: {}", provider.name, e);
                    last_error = e;
                }
            }
        }
        
        Err(last_error)
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        for provider in &self.providers {
            let url = format!("{}{}", provider.status_url, tx_hash);
            
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(tx_info) = response.json::<BlockstreamTxStatus>().await {
                        let status = if tx_info.status.confirmed {
                            TxStatus::Confirmed
                        } else {
                            TxStatus::Pending
                        };
                        
                        return Ok(TransactionStatus {
                            tx_hash: tx_hash.to_string(),
                            status,
                            confirmations: tx_info.status.block_height.map(|h| h).unwrap_or(0),
                            block_number: tx_info.status.block_height,
                            block_hash: tx_info.status.block_hash,
                            timestamp: tx_info.status.block_time,
                            fee: tx_info.fee,
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
        let estimates = self.get_fee_estimates().await?;
        Ok(estimates.medium * tx_size as u64)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        for provider in &self.providers {
            // Try to get latest block info
            let url = if provider.name.contains("Blockstream") {
                "https://blockstream.info/api/blocks/tip/height"
            } else {
                "https://mempool.space/api/blocks/tip/height"
            };
            
            if let Ok(response) = self.client.get(url).send().await {
                if response.status().is_success() {
                    if let Ok(height) = response.text().await {
                        if let Ok(block_height) = height.trim().parse::<u64>() {
                            let fees = self.get_fee_estimates().await.unwrap_or(FeeEstimates {
                                fast: 50, medium: 25, slow: 10
                            });
                            
                            return Ok(NetworkStatus {
                                is_healthy: true,
                                block_height,
                                avg_block_time: std::time::Duration::from_secs(600),
                                mempool_size: None,
                                suggested_fee: fees.medium,
                            });
                        }
                    }
                }
            }
        }
        
        Err(BroadcastError::AllProvidersFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let config = BroadcastConfig::mainnet();
        let broadcaster = BitcoinBroadcaster::new(&config);
        assert!(!broadcaster.providers.is_empty());
    }

    #[test]
    fn test_testnet_providers() {
        let config = BroadcastConfig::testnet();
        let broadcaster = BitcoinBroadcaster::new(&config);
        assert!(broadcaster.providers.iter().any(|p| p.name.contains("Testnet")));
    }
}
