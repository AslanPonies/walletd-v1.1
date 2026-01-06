//! Wallet integration module - Central wallet manager for all chains
//!
//! This module provides the WalletManager that coordinates wallet operations
//! across all supported blockchain networks.

pub mod bitcoin_real;
pub mod ethereum_real;
pub mod solana_real;
pub mod hedera_real;
pub mod monero_real;
pub mod icp_real;
pub mod base_real;
pub mod evm_chains;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::WalletConfig;
use crate::types::{Chain, WalletMode, WalletInfo};

/// Wallet state for a single chain
#[derive(Debug, Clone)]
pub struct ChainWallet {
    pub chain: Chain,
    pub address: String,
    pub mnemonic: Option<String>,
    pub initialized: bool,
}

impl ChainWallet {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            address: String::new(),
            mnemonic: None,
            initialized: false,
        }
    }
}

/// Central wallet manager coordinating all chain wallets
pub struct WalletManager {
    pub mode: WalletMode,
    pub config: WalletConfig,
    wallets: HashMap<Chain, ChainWallet>,
    master_mnemonic: Option<String>,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new(mode: WalletMode) -> Result<Self> {
        let config = WalletConfig::load().unwrap_or_default();
        
        Ok(Self {
            mode,
            config,
            wallets: HashMap::new(),
            master_mnemonic: None,
        })
    }
    
    /// Initialize from master mnemonic (HD wallet)
    pub fn init_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        // Validate mnemonic
        let word_count = mnemonic.split_whitespace().count();
        if word_count != 12 && word_count != 24 {
            return Err(anyhow!("Invalid mnemonic: must be 12 or 24 words"));
        }
        
        self.master_mnemonic = Some(mnemonic.to_string());
        
        // Initialize all chain wallets from the master mnemonic
        for chain in Chain::all() {
            self.init_chain_wallet(*chain)?;
        }
        
        Ok(())
    }
    
    /// Initialize a single chain wallet
    fn init_chain_wallet(&mut self, chain: Chain) -> Result<()> {
        let mnemonic = self.master_mnemonic.as_ref()
            .ok_or_else(|| anyhow!("No master mnemonic set"))?;
        
        let address = self.derive_address(chain, mnemonic)?;
        
        let mut wallet = ChainWallet::new(chain);
        wallet.address = address;
        wallet.mnemonic = Some(mnemonic.clone());
        wallet.initialized = true;
        
        self.wallets.insert(chain, wallet);
        
        Ok(())
    }
    
    /// Derive address for a chain from mnemonic
    fn derive_address(&self, chain: Chain, mnemonic: &str) -> Result<String> {
        // Use chain-specific derivation
        match chain {
            Chain::Bitcoin => bitcoin_real::derive_address(mnemonic, self.mode),
            Chain::Ethereum => ethereum_real::derive_address(mnemonic, self.mode),
            Chain::Solana => solana_real::derive_address(mnemonic, self.mode),
            Chain::Hedera => hedera_real::derive_address(mnemonic, self.mode),
            Chain::Monero => monero_real::derive_address(mnemonic, self.mode),
            Chain::Icp => icp_real::derive_address(mnemonic, self.mode),
            Chain::Base => evm_chains::derive_address(mnemonic, chain, self.mode),
            Chain::Polygon => evm_chains::derive_address(mnemonic, chain, self.mode),
            Chain::Avalanche => evm_chains::derive_address(mnemonic, chain, self.mode),
            Chain::Arbitrum => evm_chains::derive_address(mnemonic, chain, self.mode),
            Chain::Erc20 => ethereum_real::derive_address(mnemonic, self.mode),
            Chain::Prasaga => evm_chains::derive_address(mnemonic, chain, self.mode),
            Chain::Cardano => evm_chains::derive_cardano_address(mnemonic, self.mode),
            Chain::Cosmos => evm_chains::derive_cosmos_address(mnemonic, self.mode),
            Chain::Polkadot => evm_chains::derive_polkadot_address(mnemonic, self.mode),
            Chain::Near => evm_chains::derive_near_address(mnemonic, self.mode),
            Chain::Tron => evm_chains::derive_tron_address(mnemonic, self.mode),
            Chain::Sui => evm_chains::derive_sui_address(mnemonic, self.mode),
            Chain::Aptos => evm_chains::derive_aptos_address(mnemonic, self.mode),
            Chain::Ton => evm_chains::derive_ton_address(mnemonic, self.mode),
        }
    }
    
    /// Get wallet for a chain
    pub fn get_wallet(&self, chain: Chain) -> Option<&ChainWallet> {
        self.wallets.get(&chain)
    }
    
    /// Get address for a chain
    pub fn get_address(&self, chain: Chain) -> Option<String> {
        self.wallets.get(&chain).map(|w| w.address.clone())
    }
    
    /// Check if a chain wallet is initialized
    pub fn is_initialized(&self, chain: Chain) -> bool {
        self.wallets.get(&chain).map(|w| w.initialized).unwrap_or(false)
    }
    
    /// Get balance for a chain
    pub async fn get_balance(&self, chain: Chain) -> Result<String> {
        let wallet = self.wallets.get(&chain)
            .ok_or_else(|| anyhow!("Wallet not initialized for {:?}", chain))?;
        
        if !wallet.initialized {
            return Err(anyhow!("Wallet not initialized"));
        }
        
        let rpc = self.config.get_rpc(chain.symbol(), self.mode.is_testnet())
            .ok_or_else(|| anyhow!("No RPC endpoint configured for {:?}", chain))?;
        
        match chain {
            Chain::Bitcoin => bitcoin_real::get_balance(&wallet.address, &rpc).await,
            Chain::Ethereum => ethereum_real::get_balance(&wallet.address, &rpc).await,
            Chain::Solana => solana_real::get_balance(&wallet.address, &rpc).await,
            Chain::Hedera => hedera_real::get_balance(&wallet.address, &rpc).await,
            Chain::Monero => monero_real::get_balance(&wallet.address, &rpc).await,
            Chain::Icp => icp_real::get_balance(&wallet.address, &rpc).await,
            Chain::Base | Chain::Polygon | Chain::Avalanche | Chain::Arbitrum | Chain::Prasaga => {
                evm_chains::get_balance(&wallet.address, &rpc).await
            }
            Chain::Erc20 => ethereum_real::get_balance(&wallet.address, &rpc).await,
            Chain::Cardano => evm_chains::get_cardano_balance(&wallet.address, &rpc).await,
            Chain::Cosmos => evm_chains::get_cosmos_balance(&wallet.address, &rpc).await,
            Chain::Polkadot => evm_chains::get_polkadot_balance(&wallet.address, &rpc).await,
            Chain::Near => evm_chains::get_near_balance(&wallet.address, &rpc).await,
            Chain::Tron => evm_chains::get_tron_balance(&wallet.address, &rpc).await,
            Chain::Sui => evm_chains::get_sui_balance(&wallet.address, &rpc).await,
            Chain::Aptos => evm_chains::get_aptos_balance(&wallet.address, &rpc).await,
            Chain::Ton => evm_chains::get_ton_balance(&wallet.address, &rpc).await,
        }
    }
    
    /// Send transaction
    pub async fn send_transaction(
        &self,
        chain: Chain,
        to: &str,
        amount: &str,
    ) -> Result<String> {
        let wallet = self.wallets.get(&chain)
            .ok_or_else(|| anyhow!("Wallet not initialized for {:?}", chain))?;
        
        if !wallet.initialized {
            return Err(anyhow!("Wallet not initialized"));
        }
        
        let mnemonic = wallet.mnemonic.as_ref()
            .ok_or_else(|| anyhow!("No mnemonic available"))?;
        
        let rpc = self.config.get_rpc(chain.symbol(), self.mode.is_testnet())
            .ok_or_else(|| anyhow!("No RPC endpoint configured for {:?}", chain))?;
        
        match chain {
            Chain::Bitcoin => bitcoin_real::send_transaction(mnemonic, to, amount, &rpc, self.mode).await,
            Chain::Ethereum => ethereum_real::send_transaction(mnemonic, to, amount, &rpc, self.mode).await,
            Chain::Base | Chain::Polygon | Chain::Avalanche | Chain::Arbitrum | Chain::Prasaga => {
                evm_chains::send_transaction(mnemonic, to, amount, chain, &rpc, self.mode).await
            }
            _ => Err(anyhow!("Send not yet implemented for {:?}", chain)),
        }
    }
    
    /// Get wallet info
    pub fn get_wallet_info(&self, chain: Chain) -> Option<WalletInfo> {
        self.wallets.get(&chain).map(|w| WalletInfo {
            chain: chain.name().to_string(),
            address: w.address.clone(),
            balance: "Unknown".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Get all initialized wallets
    pub fn get_all_wallets(&self) -> Vec<WalletInfo> {
        self.wallets.iter()
            .filter(|(_, w)| w.initialized)
            .map(|(chain, w)| WalletInfo {
                chain: chain.name().to_string(),
                address: w.address.clone(),
                balance: "Unknown".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
            .collect()
    }
    
    /// Generate new mnemonic
    pub fn generate_mnemonic() -> Result<String> {
        use bip39::{Mnemonic, Language};
        let mnemonic = Mnemonic::generate_in(Language::English, 12)
            .map_err(|e| anyhow!("Failed to generate mnemonic: {:?}", e))?;
        Ok(mnemonic.to_string())
    }
    
    /// Save configuration
    pub fn save_config(&self) -> Result<()> {
        self.config.save()
    }
}

/// Thread-safe wrapper for WalletManager
pub type SharedWalletManager = Arc<RwLock<WalletManager>>;

pub fn create_shared_manager(mode: WalletMode) -> Result<SharedWalletManager> {
    let manager = WalletManager::new(mode)?;
    Ok(Arc::new(RwLock::new(manager)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_manager_creation() {
        let manager = WalletManager::new(WalletMode::Testnet).unwrap();
        assert_eq!(manager.mode, WalletMode::Testnet);
    }
    
    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = WalletManager::generate_mnemonic().unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 12);
    }
    
    #[test]
    fn test_chain_wallet_new() {
        let wallet = ChainWallet::new(Chain::Bitcoin);
        assert_eq!(wallet.chain, Chain::Bitcoin);
        assert!(!wallet.initialized);
    }
}
