//! EVM-compatible chain broadcaster (Base, Polygon, Arbitrum, Avalanche)

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

/// EVM chain type
#[derive(Debug, Clone, Copy)]
pub enum EvmChain {
    Base,
    Polygon,
    Arbitrum,
    Avalanche,
}

impl EvmChain {
    fn name(&self) -> &str {
        match self {
            EvmChain::Base => "base",
            EvmChain::Polygon => "polygon",
            EvmChain::Arbitrum => "arbitrum",
            EvmChain::Avalanche => "avalanche",
        }
    }

    fn block_time_secs(&self) -> u64 {
        match self {
            EvmChain::Base => 2,
            EvmChain::Polygon => 2,
            EvmChain::Arbitrum => 1,
            EvmChain::Avalanche => 2,
        }
    }
}

/// Generic EVM chain broadcaster
pub struct EvmBroadcaster {
    client: Client,
    chain: EvmChain,
    rpc_urls: Vec<String>,
}

impl EvmBroadcaster {
    /// Create Base broadcaster
    pub fn new_base(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://mainnet.base.org".to_string(),
                "https://base.llamarpc.com".to_string(),
                "https://1rpc.io/base".to_string(),
                "https://base.publicnode.com".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://sepolia.base.org".to_string(),
                "https://base-sepolia.publicnode.com".to_string(),
            ],
        };
        Self::new_internal(config, EvmChain::Base, rpc_urls)
    }

    /// Create Polygon broadcaster
    pub fn new_polygon(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://polygon-rpc.com".to_string(),
                "https://polygon.llamarpc.com".to_string(),
                "https://1rpc.io/matic".to_string(),
                "https://polygon-bor.publicnode.com".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://rpc-amoy.polygon.technology".to_string(),
                "https://polygon-amoy.publicnode.com".to_string(),
            ],
        };
        Self::new_internal(config, EvmChain::Polygon, rpc_urls)
    }

    /// Create Arbitrum broadcaster
    pub fn new_arbitrum(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://arb1.arbitrum.io/rpc".to_string(),
                "https://arbitrum.llamarpc.com".to_string(),
                "https://1rpc.io/arb".to_string(),
                "https://arbitrum-one.publicnode.com".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
                "https://arbitrum-sepolia.publicnode.com".to_string(),
            ],
        };
        Self::new_internal(config, EvmChain::Arbitrum, rpc_urls)
    }

    /// Create Avalanche broadcaster
    pub fn new_avalanche(config: &BroadcastConfig) -> Self {
        let rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://api.avax.network/ext/bc/C/rpc".to_string(),
                "https://avalanche.public-rpc.com".to_string(),
                "https://1rpc.io/avax/c".to_string(),
                "https://avalanche-c-chain.publicnode.com".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
                "https://avalanche-fuji-c-chain.publicnode.com".to_string(),
            ],
        };
        Self::new_internal(config, EvmChain::Avalanche, rpc_urls)
    }

    fn new_internal(config: &BroadcastConfig, chain: EvmChain, mut rpc_urls: Vec<String>) -> Self {
        // Add custom endpoints
        if let Some(endpoints) = config.custom_endpoints.get(chain.name()) {
            for endpoint in endpoints {
                rpc_urls.push(endpoint.clone());
            }
        }

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            chain,
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

        let mut last_error = BroadcastError::AllProvidersFailed;

        for rpc_url in &self.rpc_urls {
            match self.client.post(rpc_url).json(&request).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(rpc_response) = response.json::<RpcResponse<T>>().await {
                            if let Some(result) = rpc_response.result {
                                return Ok(result);
                            }
                            if let Some(error) = rpc_response.error {
                                last_error = BroadcastError::Rejected(error.message);
                            }
                        }
                    }
                }
                Err(e) => {
                    last_error = BroadcastError::Network(e);
                }
            }
        }

        Err(last_error)
    }

    async fn get_gas_price(&self) -> BroadcastResult<u64> {
        let result: String = self.rpc_call("eth_gasPrice", serde_json::json!([])).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| BroadcastError::Deserialization(e.to_string()))
    }

    async fn get_block_number(&self) -> BroadcastResult<u64> {
        let result: String = self.rpc_call("eth_blockNumber", serde_json::json!([])).await?;
        u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| BroadcastError::Deserialization(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct TxReceipt {
    #[serde(rename = "transactionHash")]
    transaction_hash: String,
    #[serde(rename = "blockNumber")]
    block_number: Option<String>,
    #[serde(rename = "blockHash")]
    block_hash: Option<String>,
    status: Option<String>,
    #[serde(rename = "gasUsed")]
    gas_used: Option<String>,
}

#[async_trait]
impl TransactionBroadcaster for EvmBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = format!("0x{}", hex::encode(signed_tx));
        
        let tx_hash: String = self
            .rpc_call("eth_sendRawTransaction", serde_json::json!([tx_hex]))
            .await?;

        tracing::info!("{} tx broadcast: {}", self.chain.name(), tx_hash);
        
        Ok(BroadcastResponse::new(tx_hash, self.chain.name(), "rpc")
            .with_confirmation_time(self.chain.block_time_secs()))
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        let receipt: Option<TxReceipt> = self
            .rpc_call("eth_getTransactionReceipt", serde_json::json!([tx_hash]))
            .await
            .ok();

        if let Some(receipt) = receipt {
            let status = match receipt.status.as_deref() {
                Some("0x1") => TxStatus::Confirmed,
                Some("0x0") => TxStatus::Failed,
                _ => TxStatus::Unknown,
            };

            let block_number = receipt.block_number.as_ref().and_then(|b| {
                u64::from_str_radix(b.trim_start_matches("0x"), 16).ok()
            });

            let gas_used = receipt.gas_used.as_ref().and_then(|g| {
                u64::from_str_radix(g.trim_start_matches("0x"), 16).ok()
            });

            return Ok(TransactionStatus {
                tx_hash: tx_hash.to_string(),
                status,
                confirmations: 1,
                block_number,
                block_hash: receipt.block_hash,
                timestamp: None,
                fee: None,
                gas_used,
            });
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
        let gas_price = self.get_gas_price().await?;
        Ok(gas_price * 21_000) // Basic transfer
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let block_height = self.get_block_number().await?;
        let gas_price = self.get_gas_price().await.unwrap_or(1_000_000_000);

        Ok(NetworkStatus {
            is_healthy: true,
            block_height,
            avg_block_time: std::time::Duration::from_secs(self.chain.block_time_secs()),
            mempool_size: None,
            suggested_fee: gas_price,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_names() {
        assert_eq!(EvmChain::Base.name(), "base");
        assert_eq!(EvmChain::Polygon.name(), "polygon");
        assert_eq!(EvmChain::Arbitrum.name(), "arbitrum");
        assert_eq!(EvmChain::Avalanche.name(), "avalanche");
    }
}
