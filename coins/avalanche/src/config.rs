use serde::{Deserialize, Serialize};

/// Avalanche network configuration (C-Chain - EVM compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: u64,
    pub name: String,
    pub currency_symbol: String,
    pub decimals: u8,
    pub block_time_ms: u64,
    pub rpc_endpoints: Vec<String>,
    pub explorer: String,
    pub chain_type: ChainType,
}

/// Avalanche chain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    CChain,  // EVM-compatible (this is what we support)
    PChain,  // Platform chain (staking) - not supported yet
    XChain,  // Exchange chain (transfers) - not supported yet
}

// Chain IDs
pub const AVALANCHE_MAINNET_CHAIN_ID: u64 = 43114;
pub const AVALANCHE_FUJI_CHAIN_ID: u64 = 43113;

pub const AVALANCHE_MAINNET: NetworkConfig = NetworkConfig {
    chain_id: 43114,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 18,
    block_time_ms: 2000,
    rpc_endpoints: Vec::new(),
    explorer: String::new(),
    chain_type: ChainType::CChain,
};

pub const AVALANCHE_FUJI: NetworkConfig = NetworkConfig {
    chain_id: 43113,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 18,
    block_time_ms: 2000,
    rpc_endpoints: Vec::new(),
    explorer: String::new(),
    chain_type: ChainType::CChain,
};

impl NetworkConfig {
    /// Avalanche C-Chain Mainnet configuration
    pub fn mainnet() -> Self {
        NetworkConfig {
            chain_id: AVALANCHE_MAINNET_CHAIN_ID,
            name: "Avalanche C-Chain".to_string(),
            currency_symbol: "AVAX".to_string(),
            decimals: 18,
            block_time_ms: 2000, // ~2 second blocks
            rpc_endpoints: vec![
                "https://api.avax.network/ext/bc/C/rpc".to_string(),
                "https://rpc.ankr.com/avalanche".to_string(),
                "https://avalanche.publicnode.com".to_string(),
                "https://avalanche-c-chain.publicnode.com".to_string(),
                "https://1rpc.io/avax/c".to_string(),
            ],
            explorer: "https://snowtrace.io".to_string(),
            chain_type: ChainType::CChain,
        }
    }

    /// Avalanche Fuji Testnet configuration
    pub fn fuji() -> Self {
        NetworkConfig {
            chain_id: AVALANCHE_FUJI_CHAIN_ID,
            name: "Avalanche Fuji Testnet".to_string(),
            currency_symbol: "AVAX".to_string(),
            decimals: 18,
            block_time_ms: 2000,
            rpc_endpoints: vec![
                "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
                "https://rpc.ankr.com/avalanche_fuji".to_string(),
                "https://avalanche-fuji.publicnode.com".to_string(),
            ],
            explorer: "https://testnet.snowtrace.io".to_string(),
            chain_type: ChainType::CChain,
        }
    }

    /// Legacy alias for testnet
    pub fn testnet() -> Self {
        Self::fuji()
    }

    /// Check if this is mainnet
    pub fn is_mainnet(&self) -> bool {
        self.chain_id == AVALANCHE_MAINNET_CHAIN_ID
    }

    /// Check if this is C-Chain
    pub fn is_cchain(&self) -> bool {
        self.chain_type == ChainType::CChain
    }

    /// Get gas price multiplier for Avalanche
    pub fn gas_price_multiplier(&self) -> f64 {
        1.1 // 10% buffer
    }

    /// Get recommended gas limit for simple transfers
    pub fn default_gas_limit(&self) -> u64 {
        21000
    }

    /// Get recommended gas limit for contract interactions
    pub fn contract_gas_limit(&self) -> u64 {
        200000 // Avalanche contracts can be more expensive
    }

    /// Get minimum base fee (25 nAVAX on Avalanche)
    pub fn min_base_fee(&self) -> u128 {
        25_000_000_000 // 25 gwei
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_id, 43114);
        assert_eq!(config.currency_symbol, "AVAX");
        assert_eq!(config.decimals, 18);
        assert!(config.is_mainnet());
        assert!(config.is_cchain());
    }

    #[test]
    fn test_fuji_config() {
        let config = NetworkConfig::fuji();
        assert_eq!(config.chain_id, 43113);
        assert_eq!(config.currency_symbol, "AVAX");
        assert!(!config.is_mainnet());
    }

    #[test]
    fn test_testnet_alias() {
        let testnet = NetworkConfig::testnet();
        let fuji = NetworkConfig::fuji();
        assert_eq!(testnet.chain_id, fuji.chain_id);
    }

    #[test]
    fn test_rpc_endpoints_not_empty() {
        let mainnet = NetworkConfig::mainnet();
        let fuji = NetworkConfig::fuji();
        assert!(!mainnet.rpc_endpoints.is_empty());
        assert!(!fuji.rpc_endpoints.is_empty());
    }

    #[test]
    fn test_gas_defaults() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.default_gas_limit(), 21000);
        assert_eq!(config.contract_gas_limit(), 200000);
        assert_eq!(config.min_base_fee(), 25_000_000_000);
    }

    #[test]
    fn test_chain_type() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_type, ChainType::CChain);
    }
}
