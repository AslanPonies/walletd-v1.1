use anyhow::Result;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use serde::{Deserialize, Serialize};

use crate::config::NetworkConfig;

/// Avalanche RPC client for C-Chain interactions
pub struct AvalancheRpcClient {
    rpc_url: String,
    config: NetworkConfig,
}

/// Transaction receipt from Avalanche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub gas_used: u64,
    pub effective_gas_price: u64,
    pub status: bool,
    pub from: String,
    pub to: Option<String>,
}

impl AvalancheRpcClient {
    /// Create a new RPC client
    pub fn new(rpc_url: &str, config: NetworkConfig) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            config,
        }
    }

    /// Create client with default mainnet config
    pub fn mainnet(rpc_url: &str) -> Self {
        Self::new(rpc_url, NetworkConfig::mainnet())
    }

    /// Create client with default testnet config
    pub fn testnet(rpc_url: &str) -> Self {
        Self::new(rpc_url, NetworkConfig::fuji())
    }

    /// Get the current chain ID
    pub async fn get_chain_id(&self) -> Result<u64> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let chain_id = provider.get_chain_id().await?;
        Ok(chain_id)
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: Address) -> Result<U256> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let balance = provider.get_balance(address).await?;
        Ok(balance)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let block_number = provider.get_block_number().await?;
        Ok(block_number)
    }

    /// Get gas price
    pub async fn get_gas_price(&self) -> Result<u128> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let gas_price = provider.get_gas_price().await?;
        Ok(gas_price)
    }

    /// Get transaction count (nonce) for an address
    pub async fn get_transaction_count(&self, address: Address) -> Result<u64> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let count = provider.get_transaction_count(address).await?;
        Ok(count)
    }

    /// Get suggested gas price with Avalanche-specific handling
    pub async fn get_suggested_gas_price(&self) -> Result<u128> {
        let base_price = self.get_gas_price().await?;
        let min_fee = self.config.min_base_fee();
        let multiplier = self.config.gas_price_multiplier();
        
        // Ensure we're above minimum base fee
        let price = base_price.max(min_fee);
        Ok((price as f64 * multiplier) as u128)
    }

    /// Verify the connected network matches expected chain ID
    pub async fn verify_network(&self) -> Result<bool> {
        let chain_id = self.get_chain_id().await?;
        Ok(chain_id == self.config.chain_id)
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Get the network config
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mainnet_client() {
        let client = AvalancheRpcClient::mainnet("https://api.avax.network/ext/bc/C/rpc");
        assert_eq!(client.config.chain_id, 43114);
    }

    #[test]
    fn test_create_testnet_client() {
        let client = AvalancheRpcClient::testnet("https://api.avax-test.network/ext/bc/C/rpc");
        assert_eq!(client.config.chain_id, 43113);
    }

    #[test]
    fn test_rpc_url_stored() {
        let url = "https://custom-rpc.example.com";
        let client = AvalancheRpcClient::mainnet(url);
        assert_eq!(client.rpc_url(), url);
    }
}
