#![allow(dead_code)]
//! Configuration management for WalletD CLI
//! Compatible with walletd_config.json format from original CLI

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure compatible with walletd_config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    #[serde(default)]
    pub version: String,
    
    #[serde(default)]
    pub default_mode: String,
    
    #[serde(default)]
    pub rpc_endpoints: RpcEndpoints,
    
    #[serde(default)]
    pub testnet_faucets: HashMap<String, String>,
    
    #[serde(default)]
    pub wallets: HashMap<String, WalletEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcEndpoints {
    // Bitcoin
    #[serde(default)]
    pub bitcoin_mainnet: String,
    #[serde(default)]
    pub bitcoin_testnet: String,
    
    // Ethereum
    #[serde(default)]
    pub ethereum_mainnet: String,
    #[serde(default)]
    pub ethereum_sepolia: String,
    
    // Solana
    #[serde(default)]
    pub solana_mainnet: String,
    #[serde(default)]
    pub solana_devnet: String,
    
    // Hedera
    #[serde(default)]
    pub hedera_mainnet: String,
    #[serde(default)]
    pub hedera_testnet: String,
    
    // ICP
    #[serde(default)]
    pub icp_mainnet: String,
    #[serde(default)]
    pub icp_local: String,
    
    // Base
    #[serde(default)]
    pub base_mainnet: String,
    #[serde(default)]
    pub base_sepolia: String,
    
    // Polygon
    #[serde(default)]
    pub polygon_mainnet: String,
    #[serde(default)]
    pub polygon_amoy: String,
    
    // Avalanche
    #[serde(default)]
    pub avalanche_mainnet: String,
    #[serde(default)]
    pub avalanche_fuji: String,
    
    // Arbitrum
    #[serde(default)]
    pub arbitrum_mainnet: String,
    #[serde(default)]
    pub arbitrum_sepolia: String,
    
    // Cosmos
    #[serde(default)]
    pub cosmos_mainnet: String,
    #[serde(default)]
    pub cosmos_testnet: String,
    
    // Near
    #[serde(default)]
    pub near_mainnet: String,
    #[serde(default)]
    pub near_testnet: String,
    
    // Tron
    #[serde(default)]
    pub tron_mainnet: String,
    #[serde(default)]
    pub tron_shasta: String,
    
    // Sui
    #[serde(default)]
    pub sui_mainnet: String,
    #[serde(default)]
    pub sui_testnet: String,
    
    // Aptos
    #[serde(default)]
    pub aptos_mainnet: String,
    #[serde(default)]
    pub aptos_testnet: String,
    
    // TON
    #[serde(default)]
    pub ton_mainnet: String,
    #[serde(default)]
    pub ton_testnet: String,
}

impl RpcEndpoints {
    pub fn with_defaults() -> Self {
        Self {
            // Bitcoin
            bitcoin_mainnet: "https://blockstream.info/api".to_string(),
            bitcoin_testnet: "https://blockstream.info/testnet/api".to_string(),
            
            // Ethereum
            ethereum_mainnet: "https://eth.llamarpc.com".to_string(),
            ethereum_sepolia: "https://sepolia.drpc.org".to_string(),
            
            // Solana
            solana_mainnet: "https://api.mainnet-beta.solana.com".to_string(),
            solana_devnet: "https://api.devnet.solana.com".to_string(),
            
            // Hedera
            hedera_mainnet: "https://mainnet.mirrornode.hedera.com".to_string(),
            hedera_testnet: "https://testnet.mirrornode.hedera.com".to_string(),
            
            // ICP
            icp_mainnet: "https://ic0.app".to_string(),
            icp_local: "http://localhost:4943".to_string(),
            
            // Base
            base_mainnet: "https://mainnet.base.org".to_string(),
            base_sepolia: "https://sepolia.base.org".to_string(),
            
            // Polygon
            polygon_mainnet: "https://polygon-rpc.com".to_string(),
            polygon_amoy: "https://rpc-amoy.polygon.technology".to_string(),
            
            // Avalanche
            avalanche_mainnet: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            avalanche_fuji: "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
            
            // Arbitrum
            arbitrum_mainnet: "https://arb1.arbitrum.io/rpc".to_string(),
            arbitrum_sepolia: "https://sepolia-rollup.arbitrum.io/rpc".to_string(),
            
            // Cosmos
            cosmos_mainnet: "https://cosmos-rpc.polkachu.com".to_string(),
            cosmos_testnet: "https://rpc.sentry-01.theta-testnet.polypore.xyz".to_string(),
            
            // Near
            near_mainnet: "https://rpc.mainnet.near.org".to_string(),
            near_testnet: "https://rpc.testnet.near.org".to_string(),
            
            // Tron
            tron_mainnet: "https://api.trongrid.io".to_string(),
            tron_shasta: "https://api.shasta.trongrid.io".to_string(),
            
            // Sui
            sui_mainnet: "https://fullnode.mainnet.sui.io:443".to_string(),
            sui_testnet: "https://fullnode.testnet.sui.io:443".to_string(),
            
            // Aptos
            aptos_mainnet: "https://fullnode.mainnet.aptoslabs.com".to_string(),
            aptos_testnet: "https://fullnode.testnet.aptoslabs.com".to_string(),
            
            // TON
            ton_mainnet: "https://toncenter.com/api/v2".to_string(),
            ton_testnet: "https://testnet.toncenter.com/api/v2".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub chain: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_key: Option<String>,
    pub created_at: String,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            version: "0.2.1".to_string(),
            default_mode: "testnet".to_string(),
            rpc_endpoints: RpcEndpoints::with_defaults(),
            testnet_faucets: default_faucets(),
            wallets: HashMap::new(),
        }
    }
}

fn default_faucets() -> HashMap<String, String> {
    let mut faucets = HashMap::new();
    faucets.insert("bitcoin".to_string(), "https://testnet-faucet.com/btc-testnet/".to_string());
    faucets.insert("ethereum".to_string(), "https://sepolia-faucet.pk910.de/".to_string());
    faucets.insert("solana".to_string(), "https://faucet.solana.com/".to_string());
    faucets.insert("hedera".to_string(), "https://portal.hedera.com/faucet".to_string());
    faucets.insert("base".to_string(), "https://www.coinbase.com/faucets/base-ethereum-sepolia-faucet".to_string());
    faucets.insert("polygon".to_string(), "https://faucet.polygon.technology/".to_string());
    faucets.insert("avalanche".to_string(), "https://faucet.avax.network/".to_string());
    faucets.insert("arbitrum".to_string(), "https://faucet.arbitrum.io/".to_string());
    faucets.insert("sui".to_string(), "https://faucet.testnet.sui.io/".to_string());
    faucets.insert("aptos".to_string(), "https://aptoslabs.com/testnet-faucet".to_string());
    faucets.insert("near".to_string(), "https://wallet.testnet.near.org/".to_string());
    faucets.insert("ton".to_string(), "https://testnet.toncenter.com/".to_string());
    faucets
}

impl WalletConfig {
    /// Get config file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("walletd")
            .join("walletd_config.json")
    }
    
    /// Load config from file or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            let config: WalletConfig = serde_json::from_str(&content)
                .with_context(|| "Failed to parse config file")?;
            Ok(config)
        } else {
            let config = WalletConfig::default();
            config.save()?;
            Ok(config)
        }
    }
    
    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        
        // Create directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;
        
        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, permissions)?;
        }
        
        Ok(())
    }
    
    /// Get RPC endpoint for a chain
    pub fn get_rpc(&self, chain: &str, testnet: bool) -> Option<String> {
        let endpoints = &self.rpc_endpoints;
        let endpoint = match (chain.to_lowercase().as_str(), testnet) {
            ("bitcoin" | "btc", true) => &endpoints.bitcoin_testnet,
            ("bitcoin" | "btc", false) => &endpoints.bitcoin_mainnet,
            ("ethereum" | "eth", true) => &endpoints.ethereum_sepolia,
            ("ethereum" | "eth", false) => &endpoints.ethereum_mainnet,
            ("solana" | "sol", true) => &endpoints.solana_devnet,
            ("solana" | "sol", false) => &endpoints.solana_mainnet,
            ("hedera" | "hbar", true) => &endpoints.hedera_testnet,
            ("hedera" | "hbar", false) => &endpoints.hedera_mainnet,
            ("icp", true) => &endpoints.icp_local,
            ("icp", false) => &endpoints.icp_mainnet,
            ("base", true) => &endpoints.base_sepolia,
            ("base", false) => &endpoints.base_mainnet,
            ("polygon" | "pol", true) => &endpoints.polygon_amoy,
            ("polygon" | "pol", false) => &endpoints.polygon_mainnet,
            ("avalanche" | "avax", true) => &endpoints.avalanche_fuji,
            ("avalanche" | "avax", false) => &endpoints.avalanche_mainnet,
            ("arbitrum" | "arb", true) => &endpoints.arbitrum_sepolia,
            ("arbitrum" | "arb", false) => &endpoints.arbitrum_mainnet,
            ("cosmos" | "atom", true) => &endpoints.cosmos_testnet,
            ("cosmos" | "atom", false) => &endpoints.cosmos_mainnet,
            ("near", true) => &endpoints.near_testnet,
            ("near", false) => &endpoints.near_mainnet,
            ("tron" | "trx", true) => &endpoints.tron_shasta,
            ("tron" | "trx", false) => &endpoints.tron_mainnet,
            ("sui", true) => &endpoints.sui_testnet,
            ("sui", false) => &endpoints.sui_mainnet,
            ("aptos" | "apt", true) => &endpoints.aptos_testnet,
            ("aptos" | "apt", false) => &endpoints.aptos_mainnet,
            ("ton", true) => &endpoints.ton_testnet,
            ("ton", false) => &endpoints.ton_mainnet,
            _ => return None,
        };
        
        if endpoint.is_empty() {
            None
        } else {
            Some(endpoint.clone())
        }
    }
    
    /// Get faucet URL for a chain
    pub fn get_faucet(&self, chain: &str) -> Option<String> {
        self.testnet_faucets.get(&chain.to_lowercase()).cloned()
    }
    
    /// Add a wallet entry
    pub fn add_wallet(&mut self, id: &str, entry: WalletEntry) {
        self.wallets.insert(id.to_string(), entry);
    }
    
    /// Get a wallet entry
    pub fn get_wallet(&self, id: &str) -> Option<&WalletEntry> {
        self.wallets.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = WalletConfig::default();
        assert_eq!(config.version, "0.2.1");
        assert!(!config.rpc_endpoints.ethereum_mainnet.is_empty());
    }
    
    #[test]
    fn test_get_rpc() {
        let config = WalletConfig::default();
        assert!(config.get_rpc("ethereum", false).is_some());
        assert!(config.get_rpc("ethereum", true).is_some());
        assert!(config.get_rpc("invalid", false).is_none());
    }
    
    #[test]
    fn test_get_faucet() {
        let config = WalletConfig::default();
        assert!(config.get_faucet("ethereum").is_some());
        assert!(config.get_faucet("solana").is_some());
    }
    
    #[test]
    fn test_serialization() {
        let config = WalletConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: WalletConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, config.version);
    }
}
