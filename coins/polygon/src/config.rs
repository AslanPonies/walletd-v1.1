use serde::{Deserialize, Serialize};

/// Polygon network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: u64,
    pub name: String,
    pub currency_symbol: String,
    pub decimals: u8,
    pub block_time_ms: u64,
    pub rpc_endpoints: Vec<String>,
    pub explorer: String,
    pub is_pos: bool, // Polygon uses Proof of Stake
}

// Chain IDs
pub const POLYGON_MAINNET_CHAIN_ID: u64 = 137;
pub const POLYGON_AMOY_CHAIN_ID: u64 = 80002; // New testnet (Mumbai deprecated)

pub const POLYGON_MAINNET: NetworkConfig = NetworkConfig {
    chain_id: 137,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 18,
    block_time_ms: 2000,
    rpc_endpoints: Vec::new(),
    explorer: String::new(),
    is_pos: true,
};

pub const POLYGON_AMOY: NetworkConfig = NetworkConfig {
    chain_id: 80002,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 18,
    block_time_ms: 2000,
    rpc_endpoints: Vec::new(),
    explorer: String::new(),
    is_pos: true,
};

impl NetworkConfig {
    /// Polygon Mainnet configuration
    pub fn mainnet() -> Self {
        NetworkConfig {
            chain_id: POLYGON_MAINNET_CHAIN_ID,
            name: "Polygon Mainnet".to_string(),
            currency_symbol: "POL".to_string(), // Rebranded from MATIC
            decimals: 18,
            block_time_ms: 2000, // ~2 second blocks
            rpc_endpoints: vec![
                "https://polygon-rpc.com".to_string(),
                "https://rpc.ankr.com/polygon".to_string(),
                "https://polygon.publicnode.com".to_string(),
                "https://polygon-mainnet.public.blastapi.io".to_string(),
                "https://1rpc.io/matic".to_string(),
            ],
            explorer: "https://polygonscan.com".to_string(),
            is_pos: true,
        }
    }

    /// Polygon Amoy Testnet configuration (replaced Mumbai)
    pub fn amoy() -> Self {
        NetworkConfig {
            chain_id: POLYGON_AMOY_CHAIN_ID,
            name: "Polygon Amoy Testnet".to_string(),
            currency_symbol: "POL".to_string(),
            decimals: 18,
            block_time_ms: 2000,
            rpc_endpoints: vec![
                "https://rpc-amoy.polygon.technology".to_string(),
                "https://polygon-amoy.publicnode.com".to_string(),
                "https://polygon-amoy.drpc.org".to_string(),
            ],
            explorer: "https://amoy.polygonscan.com".to_string(),
            is_pos: true,
        }
    }

    /// Legacy alias for testnet
    pub fn testnet() -> Self {
        Self::amoy()
    }

    /// Get gas price multiplier for Polygon (usually needs higher than estimate)
    pub fn gas_price_multiplier(&self) -> f64 {
        1.1 // 10% buffer for Polygon's variable gas prices
    }

    /// Get recommended gas limit for simple transfers
    pub fn default_gas_limit(&self) -> u64 {
        21000
    }

    /// Get recommended gas limit for contract interactions
    pub fn contract_gas_limit(&self) -> u64 {
        100000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_id, 137);
        assert_eq!(config.currency_symbol, "POL");
        assert_eq!(config.decimals, 18);
        assert!(config.is_pos);
    }

    #[test]
    fn test_amoy_config() {
        let config = NetworkConfig::amoy();
        assert_eq!(config.chain_id, 80002);
        assert_eq!(config.currency_symbol, "POL");
    }

    #[test]
    fn test_testnet_alias() {
        let testnet = NetworkConfig::testnet();
        let amoy = NetworkConfig::amoy();
        assert_eq!(testnet.chain_id, amoy.chain_id);
    }

    #[test]
    fn test_rpc_endpoints_not_empty() {
        let mainnet = NetworkConfig::mainnet();
        let amoy = NetworkConfig::amoy();
        assert!(!mainnet.rpc_endpoints.is_empty());
        assert!(!amoy.rpc_endpoints.is_empty());
    }

    #[test]
    fn test_gas_defaults() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.default_gas_limit(), 21000);
        assert_eq!(config.contract_gas_limit(), 100000);
        assert!(config.gas_price_multiplier() > 1.0);
    }
}
