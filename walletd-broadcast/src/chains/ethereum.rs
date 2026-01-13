//! Ethereum transaction broadcaster

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    error::{BroadcastError, BroadcastResult},
    types::{BroadcastConfig, BroadcastResponse, NetworkMode, TransactionStatus, TxStatus},
    NetworkStatus, TransactionBroadcaster,
};

/// Ethereum broadcaster with multiple RPC provider support
pub struct EthereumBroadcaster {
    client: Client,
    rpc_urls: Vec<String>,
    network: NetworkMode,
}

impl EthereumBroadcaster {
    /// Create new Ethereum broadcaster
    pub fn new(config: &BroadcastConfig) -> Self {
        let mut rpc_urls = match config.network {
            NetworkMode::Mainnet => vec![
                "https://eth.llamarpc.com".to_string(),
                "https://rpc.ankr.com/eth".to_string(),
                "https://ethereum.publicnode.com".to_string(),
                "https://1rpc.io/eth".to_string(),
            ],
            NetworkMode::Testnet => vec![
                "https://rpc.sepolia.org".to_string(),
                "https://ethereum-sepolia.publicnode.com".to_string(),
                "https://rpc.ankr.com/eth_sepolia".to_string(),
            ],
        };

        // Add Infura if API key provided
        if let Some(ref infura_key) = config.api_keys.infura {
            let network = if config.network == NetworkMode::Mainnet { "mainnet" } else { "sepolia" };
            rpc_urls.insert(0, format!("https://{}.infura.io/v3/{}", network, infura_key));
        }

        // Add Alchemy if API key provided
        if let Some(ref alchemy_key) = config.api_keys.alchemy {
            let network = if config.network == NetworkMode::Mainnet { "eth-mainnet" } else { "eth-sepolia" };
            rpc_urls.insert(0, format!("https://{}.g.alchemy.com/v2/{}", network, alchemy_key));
        }

        // Add custom endpoints
        if let Some(endpoints) = config.custom_endpoints.get("ethereum") {
            for endpoint in endpoints {
                rpc_urls.push(endpoint.clone());
            }
        }

        Self {
            client: Client::builder()
                .timeout(config.timeout)
                .build()
                .expect("Failed to create HTTP client"),
            rpc_urls,
            network: config.network,
        }
    }

    /// Make JSON-RPC call
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
                    tracing::warn!("RPC {} failed: {}", rpc_url, e);
                    last_error = BroadcastError::Network(e);
                }
            }
        }

        Err(last_error)
    }

    /// Get gas price
    pub async fn get_gas_price(&self) -> BroadcastResult<u64> {
        let result: String = self.rpc_call("eth_gasPrice", serde_json::json!([])).await?;
        let gas_price = u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| BroadcastError::Deserialization(e.to_string()))?;
        Ok(gas_price)
    }

    /// Get transaction count (nonce)
    pub async fn get_transaction_count(&self, address: &str) -> BroadcastResult<u64> {
        let result: String = self
            .rpc_call("eth_getTransactionCount", serde_json::json!([address, "pending"]))
            .await?;
        let nonce = u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| BroadcastError::Deserialization(e.to_string()))?;
        Ok(nonce)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> BroadcastResult<u64> {
        let result: String = self.rpc_call("eth_blockNumber", serde_json::json!([])).await?;
        let block = u64::from_str_radix(result.trim_start_matches("0x"), 16)
            .map_err(|e| BroadcastError::Deserialization(e.to_string()))?;
        Ok(block)
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct EthTransactionReceipt {
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
impl TransactionBroadcaster for EthereumBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        let tx_hex = format!("0x{}", hex::encode(signed_tx));
        
        let tx_hash: String = self
            .rpc_call("eth_sendRawTransaction", serde_json::json!([tx_hex]))
            .await?;

        tracing::info!("Ethereum tx broadcast: {}", tx_hash);
        
        Ok(BroadcastResponse::new(tx_hash, "ethereum", "rpc")
            .with_confirmation_time(15)) // ~15 seconds
    }

    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        // Try to get receipt first (confirmed tx)
        let receipt_result: Option<EthTransactionReceipt> = self
            .rpc_call("eth_getTransactionReceipt", serde_json::json!([tx_hash]))
            .await
            .ok();

        if let Some(receipt) = receipt_result {
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

        // Check if tx exists but not confirmed
        let tx_result: Option<serde_json::Value> = self
            .rpc_call("eth_getTransactionByHash", serde_json::json!([tx_hash]))
            .await
            .ok();

        if tx_result.is_some() {
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
        let gas_price = self.get_gas_price().await?;
        // Typical ERC-20 transfer gas: 65000
        let gas_limit = if tx_size > 100 { 100_000 } else { 21_000 };
        Ok(gas_price * gas_limit)
    }

    async fn network_status(&self) -> BroadcastResult<NetworkStatus> {
        let block_height = self.get_block_number().await?;
        let gas_price = self.get_gas_price().await.unwrap_or(20_000_000_000); // 20 gwei default

        Ok(NetworkStatus {
            is_healthy: true,
            block_height,
            avg_block_time: std::time::Duration::from_secs(12),
            mempool_size: None,
            suggested_fee: gas_price,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let config = BroadcastConfig::mainnet();
        let broadcaster = EthereumBroadcaster::new(&config);
        assert!(!broadcaster.rpc_urls.is_empty());
    }

    #[test]
    fn test_infura_priority() {
        let mut config = BroadcastConfig::mainnet();
        config.api_keys.infura = Some("test_key".to_string());
        let broadcaster = EthereumBroadcaster::new(&config);
        assert!(broadcaster.rpc_urls[0].contains("infura"));
    }
}
