use serde::{Deserialize, Serialize};

/// Cardano network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_id: u8,
    pub name: String,
    pub currency_symbol: String,
    pub decimals: u8,
    pub slot_length_seconds: u64,
    pub api_endpoints: Vec<String>,
    pub explorer: String,
    pub address_prefix: String,
}

/// Cardano network IDs
pub const MAINNET_NETWORK_ID: u8 = 1;
pub const TESTNET_NETWORK_ID: u8 = 0; // Preview/Preprod

/// Address types in Cardano
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    Base,       // Payment + staking key
    Enterprise, // Payment key only (no staking)
    Pointer,    // Payment + stake pool pointer
    Reward,     // Staking rewards address
    Byron,      // Legacy Byron addresses
}

/// Lovelace is the smallest unit (1 ADA = 1,000,000 Lovelace)
pub const LOVELACE_PER_ADA: u64 = 1_000_000;

/// Minimum UTXO value (depends on era, ~1 ADA typically)
pub const MIN_UTXO_LOVELACE: u64 = 1_000_000;

pub const CARDANO_MAINNET: NetworkConfig = NetworkConfig {
    network_id: 1,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 6,
    slot_length_seconds: 1,
    api_endpoints: Vec::new(),
    explorer: String::new(),
    address_prefix: String::new(),
};

pub const CARDANO_TESTNET: NetworkConfig = NetworkConfig {
    network_id: 0,
    name: String::new(),
    currency_symbol: String::new(),
    decimals: 6,
    slot_length_seconds: 1,
    api_endpoints: Vec::new(),
    explorer: String::new(),
    address_prefix: String::new(),
};

impl NetworkConfig {
    /// Cardano Mainnet configuration
    pub fn mainnet() -> Self {
        NetworkConfig {
            network_id: MAINNET_NETWORK_ID,
            name: "Cardano Mainnet".to_string(),
            currency_symbol: "ADA".to_string(),
            decimals: 6, // 1 ADA = 1,000,000 Lovelace
            slot_length_seconds: 1,
            api_endpoints: vec![
                "https://cardano-mainnet.blockfrost.io/api/v0".to_string(),
                "https://api.koios.rest/api/v1".to_string(),
            ],
            explorer: "https://cardanoscan.io".to_string(),
            address_prefix: "addr".to_string(),
        }
    }

    /// Cardano Preview Testnet configuration
    pub fn preview() -> Self {
        NetworkConfig {
            network_id: TESTNET_NETWORK_ID,
            name: "Cardano Preview".to_string(),
            currency_symbol: "tADA".to_string(),
            decimals: 6,
            slot_length_seconds: 1,
            api_endpoints: vec![
                "https://cardano-preview.blockfrost.io/api/v0".to_string(),
            ],
            explorer: "https://preview.cardanoscan.io".to_string(),
            address_prefix: "addr_test".to_string(),
        }
    }

    /// Cardano Preprod Testnet configuration
    pub fn preprod() -> Self {
        NetworkConfig {
            network_id: TESTNET_NETWORK_ID,
            name: "Cardano Preprod".to_string(),
            currency_symbol: "tADA".to_string(),
            decimals: 6,
            slot_length_seconds: 1,
            api_endpoints: vec![
                "https://cardano-preprod.blockfrost.io/api/v0".to_string(),
            ],
            explorer: "https://preprod.cardanoscan.io".to_string(),
            address_prefix: "addr_test".to_string(),
        }
    }

    /// Legacy testnet alias
    pub fn testnet() -> Self {
        Self::preview()
    }

    /// Check if mainnet
    pub fn is_mainnet(&self) -> bool {
        self.network_id == MAINNET_NETWORK_ID
    }

    /// Convert ADA to Lovelace
    pub fn ada_to_lovelace(ada: f64) -> u64 {
        (ada * LOVELACE_PER_ADA as f64) as u64
    }

    /// Convert Lovelace to ADA
    pub fn lovelace_to_ada(lovelace: u64) -> f64 {
        lovelace as f64 / LOVELACE_PER_ADA as f64
    }

    /// Get minimum UTXO value
    pub fn min_utxo(&self) -> u64 {
        MIN_UTXO_LOVELACE
    }

    /// Get transaction fee estimate (simplified)
    pub fn estimate_fee(&self, tx_size_bytes: usize) -> u64 {
        // Cardano fee formula: a + b * size
        // a = 155381 lovelace (constant)
        // b = 44 lovelace per byte
        let a: u64 = 155381;
        let b: u64 = 44;
        a + b * tx_size_bytes as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_config() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.network_id, 1);
        assert_eq!(config.currency_symbol, "ADA");
        assert_eq!(config.decimals, 6);
        assert!(config.is_mainnet());
    }

    #[test]
    fn test_preview_config() {
        let config = NetworkConfig::preview();
        assert_eq!(config.network_id, 0);
        assert_eq!(config.currency_symbol, "tADA");
        assert!(!config.is_mainnet());
    }

    #[test]
    fn test_preprod_config() {
        let config = NetworkConfig::preprod();
        assert_eq!(config.network_id, 0);
        assert_eq!(config.address_prefix, "addr_test");
    }

    #[test]
    fn test_ada_lovelace_conversion() {
        assert_eq!(NetworkConfig::ada_to_lovelace(1.0), 1_000_000);
        assert_eq!(NetworkConfig::ada_to_lovelace(0.5), 500_000);
        assert_eq!(NetworkConfig::lovelace_to_ada(1_000_000), 1.0);
        assert_eq!(NetworkConfig::lovelace_to_ada(500_000), 0.5);
    }

    #[test]
    fn test_min_utxo() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.min_utxo(), 1_000_000);
    }

    #[test]
    fn test_fee_estimate() {
        let config = NetworkConfig::mainnet();
        // Typical simple transaction ~300 bytes
        let fee = config.estimate_fee(300);
        assert!(fee > 155381); // More than base fee
        assert!(fee < 500_000); // Less than 0.5 ADA
    }

    #[test]
    fn test_address_prefix() {
        let mainnet = NetworkConfig::mainnet();
        let testnet = NetworkConfig::preview();
        assert_eq!(mainnet.address_prefix, "addr");
        assert_eq!(testnet.address_prefix, "addr_test");
    }
}
