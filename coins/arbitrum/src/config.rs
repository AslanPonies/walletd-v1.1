//! Arbitrum network configuration

use serde::{Deserialize, Serialize};

/// Arbitrum network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Chain ID
    pub chain_id: u64,
    /// Network name
    pub name: String,
    /// Native currency symbol
    pub currency_symbol: String,
    /// Decimal places (18 for ETH)
    pub decimals: u8,
    /// Average block time in milliseconds
    pub block_time_ms: u64,
    /// RPC endpoint URLs
    pub rpc_endpoints: Vec<String>,
    /// Block explorer URL
    pub explorer: String,
    /// Bridge URL
    pub bridge: String,
}

/// Arbitrum One Mainnet chain ID
pub const ARBITRUM_ONE_CHAIN_ID: u64 = 42161;

/// Arbitrum Sepolia testnet chain ID
pub const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421614;

/// Arbitrum Nova chain ID (AnyTrust chain for gaming)
pub const ARBITRUM_NOVA_CHAIN_ID: u64 = 42170;

impl NetworkConfig {
    /// Arbitrum One Mainnet configuration
    pub fn mainnet() -> Self {
        NetworkConfig {
            chain_id: ARBITRUM_ONE_CHAIN_ID,
            name: "Arbitrum One".to_string(),
            currency_symbol: "ETH".to_string(),
            decimals: 18,
            block_time_ms: 250, // ~0.25 seconds
            rpc_endpoints: vec![
                "https://arb1.arbitrum.io/rpc".to_string(),
                "https://arbitrum.publicnode.com".to_string(),
                "https://rpc.ankr.com/arbitrum".to_string(),
                "https://arbitrum.drpc.org".to_string(),
                "https://1rpc.io/arb".to_string(),
            ],
            explorer: "https://arbiscan.io".to_string(),
            bridge: "https://bridge.arbitrum.io".to_string(),
        }
    }

    /// Arbitrum Sepolia testnet configuration
    pub fn sepolia() -> Self {
        NetworkConfig {
            chain_id: ARBITRUM_SEPOLIA_CHAIN_ID,
            name: "Arbitrum Sepolia".to_string(),
            currency_symbol: "ETH".to_string(),
            decimals: 18,
            block_time_ms: 250,
            rpc_endpoints: vec![
                "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
                "https://arbitrum-sepolia.publicnode.com".to_string(),
            ],
            explorer: "https://sepolia.arbiscan.io".to_string(),
            bridge: "https://bridge.arbitrum.io".to_string(),
        }
    }

    /// Arbitrum Nova configuration (AnyTrust chain for gaming/social)
    pub fn nova() -> Self {
        NetworkConfig {
            chain_id: ARBITRUM_NOVA_CHAIN_ID,
            name: "Arbitrum Nova".to_string(),
            currency_symbol: "ETH".to_string(),
            decimals: 18,
            block_time_ms: 250,
            rpc_endpoints: vec![
                "https://nova.arbitrum.io/rpc".to_string(),
                "https://arbitrum-nova.publicnode.com".to_string(),
            ],
            explorer: "https://nova.arbiscan.io".to_string(),
            bridge: "https://bridge.arbitrum.io".to_string(),
        }
    }

    /// Returns the primary RPC endpoint
    pub fn primary_rpc(&self) -> &str {
        self.rpc_endpoints.first().map(|s| s.as_str()).unwrap_or("")
    }

    /// Check if this is a testnet
    pub fn is_testnet(&self) -> bool {
        self.chain_id == ARBITRUM_SEPOLIA_CHAIN_ID
    }

    /// Check if this is Arbitrum Nova
    pub fn is_nova(&self) -> bool {
        self.chain_id == ARBITRUM_NOVA_CHAIN_ID
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_id, 42161);
        assert_eq!(config.currency_symbol, "ETH");
        assert_eq!(config.decimals, 18);
        assert!(!config.rpc_endpoints.is_empty());
        assert!(config.explorer.contains("arbiscan"));
    }

    #[test]
    fn test_sepolia_config() {
        let config = NetworkConfig::sepolia();
        assert_eq!(config.chain_id, 421614);
        assert!(config.is_testnet());
        assert!(!config.is_nova());
    }

    #[test]
    fn test_nova_config() {
        let config = NetworkConfig::nova();
        assert_eq!(config.chain_id, 42170);
        assert!(config.is_nova());
        assert!(!config.is_testnet());
    }

    #[test]
    fn test_primary_rpc() {
        let config = NetworkConfig::mainnet();
        assert!(!config.primary_rpc().is_empty());
        assert!(config.primary_rpc().contains("arbitrum"));
    }

    #[test]
    fn test_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.chain_id, ARBITRUM_ONE_CHAIN_ID);
    }
}
