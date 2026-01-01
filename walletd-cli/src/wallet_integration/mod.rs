//! Wallet Integration Module
//!
//! Central wallet manager that integrates with WalletD SDK crates.
//! This is the core module that connects the CLI to actual blockchain operations.

use crate::config::WalletDConfig;
use crate::types::WalletMode;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod bitcoin_real;
pub mod ethereum_real;
pub mod solana_real;
pub mod monero_real;
pub mod hedera_real;
pub mod icp_real;

use bitcoin_real::RealBitcoinWallet;
use ethereum_real::RealEthereumWallet;
use solana_real::RealSolanaWallet;
use monero_real::RealMoneroWallet;
use hedera_real::RealHederaWallet;
use icp_real::RealIcpWallet;

/// Global wallet manager instance
pub static WALLET_MANAGER: Lazy<Arc<RwLock<WalletManager>>> = Lazy::new(|| {
    let config = WalletDConfig::load();
    Arc::new(RwLock::new(WalletManager::new(config)))
});

/// Central wallet manager for all blockchain integrations
pub struct WalletManager {
    pub config: WalletDConfig,
    pub mode: WalletMode,
    
    // Core chain wallets
    pub bitcoin: Option<RealBitcoinWallet>,
    pub ethereum: Option<RealEthereumWallet>,
    pub solana: Option<RealSolanaWallet>,
    pub hedera: Option<RealHederaWallet>,
    pub monero: Option<RealMoneroWallet>,
    pub icp: Option<RealIcpWallet>,
    
    // Extended chain wallets (to be added when SDK crates exist)
    // pub polygon: Option<RealPolygonWallet>,
    // pub avalanche: Option<RealAvalancheWallet>,
    // pub cardano: Option<RealCardanoWallet>,
    // etc.
}

/// Balance information
#[derive(Debug, Clone)]
pub struct Balance {
    pub confirmed: u64,
    pub unconfirmed: u64,
    pub total: u64,
}

impl WalletManager {
    /// Create a new wallet manager with configuration
    pub fn new(config: WalletDConfig) -> Self {
        let mode = if config.demo_mode {
            WalletMode::Demo
        } else if config.bitcoin.network == "testnet" {
            WalletMode::Testnet
        } else {
            WalletMode::Mainnet
        };

        Self {
            config,
            mode,
            bitcoin: None,
            ethereum: None,
            solana: None,
            hedera: None,
            monero: None,
            icp: None,
        }
    }

    /// Initialize all wallets based on mode
    pub async fn init_all(&mut self) -> Result<()> {
        println!("🔄 Initializing all wallets...\n");
        
        // Core chains
        if let Err(e) = self.init_bitcoin().await {
            println!("⚠️  Bitcoin: {e}");
        }
        
        if let Err(e) = self.init_ethereum().await {
            println!("⚠️  Ethereum: {e}");
        }
        
        if let Err(e) = self.init_solana().await {
            println!("⚠️  Solana: {e}");
        }
        
        if let Err(e) = self.init_hedera().await {
            println!("⚠️  Hedera: {e}");
        }
        
        if let Err(e) = self.init_monero().await {
            println!("⚠️  Monero: {e}");
        }
        
        if let Err(e) = self.init_icp().await {
            println!("⚠️  ICP: {e}");
        }
        
        println!("\n✅ Wallet initialization complete!");
        Ok(())
    }

    // =========================================================================
    // Bitcoin Integration
    // =========================================================================
    
    pub async fn init_bitcoin(&mut self) -> Result<()> {
        println!("🔄 Initializing Bitcoin wallet...");
        
        let network = match self.config.bitcoin.network.as_str() {
            "testnet" => bitcoin::Network::Testnet,
            "mainnet" => bitcoin::Network::Bitcoin,
            "regtest" => bitcoin::Network::Regtest,
            _ => bitcoin::Network::Testnet,
        };
        
        let wallet = RealBitcoinWallet::new(network)?;
        
        println!("✅ Bitcoin wallet initialized ({:?})", network);
        println!("📍 Address: {}", wallet.address);
        
        // Try to fetch balance
        match wallet.get_balance().await {
            Ok(balance) => {
                let btc = balance as f64 / 100_000_000.0;
                println!("💰 Balance: {:.8} BTC", btc);
            }
            Err(_) => println!("💰 Balance: Unable to fetch"),
        }
        
        if network == bitcoin::Network::Testnet {
            println!("💡 Get testnet BTC: https://coinfaucet.eu/en/btc-testnet/");
        }
        
        self.bitcoin = Some(wallet);
        Ok(())
    }
    
    pub async fn get_bitcoin_wallet(&self, _user_id: &str) -> Result<(String, String)> {
        if let Some(wallet) = &self.bitcoin {
            let balance = wallet.get_balance().await.unwrap_or(0);
            let btc = balance as f64 / 100_000_000.0;
            Ok((wallet.address.to_string(), format!("{:.8}", btc)))
        } else {
            Err(anyhow::anyhow!("Bitcoin wallet not initialized"))
        }
    }
    
    pub async fn send_bitcoin(&self, to_address: &str, amount_btc: f64) -> Result<String> {
        if let Some(wallet) = &self.bitcoin {
            let amount_sats = (amount_btc * 100_000_000.0) as u64;
            wallet.create_and_send_transaction(to_address, amount_sats).await
        } else {
            Err(anyhow::anyhow!("Bitcoin wallet not initialized"))
        }
    }

    // =========================================================================
    // Ethereum Integration
    // =========================================================================
    
    pub async fn init_ethereum(&mut self) -> Result<()> {
        println!("🔄 Initializing Ethereum wallet...");
        
        let mut wallet = RealEthereumWallet::new(self.config.ethereum.chain_id)?;
        
        if let Err(e) = wallet.connect(&self.config.ethereum.rpc_url).await {
            println!("⚠️  Could not connect: {e}");
        }
        
        println!("✅ Ethereum wallet initialized (chain {})", self.config.ethereum.chain_id);
        println!("📍 Address: {}", wallet.address_string());
        
        if self.config.ethereum.chain_id == 11155111 {
            println!("💡 Get Sepolia ETH: https://sepoliafaucet.com/");
        }
        
        self.ethereum = Some(wallet);
        Ok(())
    }
    
    pub async fn get_ethereum_wallet(&self) -> Result<(String, String)> {
        if let Some(wallet) = &self.ethereum {
            let balance = wallet.get_balance().await.unwrap_or(0);
            let eth = balance as f64 / 1e18;
            Ok((wallet.address_string(), format!("{:.6}", eth)))
        } else {
            Err(anyhow::anyhow!("Ethereum wallet not initialized"))
        }
    }
    
    pub async fn send_ethereum(&self, to_address: &str, amount_eth: f64) -> Result<String> {
        if let Some(wallet) = &self.ethereum {
            wallet.send_transaction(to_address, amount_eth).await
        } else {
            Err(anyhow::anyhow!("Ethereum wallet not initialized"))
        }
    }

    // =========================================================================
    // Solana Integration
    // =========================================================================
    
    pub async fn init_solana(&mut self) -> Result<()> {
        println!("🔄 Initializing Solana wallet...");
        
        let wallet = RealSolanaWallet::new(&self.config.solana.cluster)?;
        
        println!("✅ Solana wallet initialized ({})", self.config.solana.cluster);
        println!("📍 Address: {}", wallet.address);
        
        match wallet.get_balance().await {
            Ok(balance) => {
                let sol = balance as f64 / 1_000_000_000.0;
                println!("💰 Balance: {:.9} SOL", sol);
            }
            Err(_) => println!("💰 Balance: Unable to fetch"),
        }
        
        if self.config.solana.cluster == "devnet" {
            println!("💡 Get devnet SOL: https://faucet.solana.com/");
        }
        
        self.solana = Some(wallet);
        Ok(())
    }
    
    pub async fn get_solana_wallet(&self, _user_id: &str) -> Result<(String, String)> {
        if let Some(wallet) = &self.solana {
            let balance = wallet.get_balance().await.unwrap_or(0);
            let sol = balance as f64 / 1_000_000_000.0;
            Ok((wallet.address.clone(), format!("{:.9}", sol)))
        } else {
            Err(anyhow::anyhow!("Solana wallet not initialized"))
        }
    }
    
    pub async fn send_solana(&self, to_address: &str, amount_sol: f64) -> Result<String> {
        if let Some(wallet) = &self.solana {
            let lamports = (amount_sol * 1_000_000_000.0) as u64;
            wallet.send_transaction(to_address, lamports).await
        } else {
            Err(anyhow::anyhow!("Solana wallet not initialized"))
        }
    }

    // =========================================================================
    // Hedera Integration
    // =========================================================================
    
    pub async fn init_hedera(&mut self) -> Result<()> {
        println!("🔄 Initializing Hedera wallet...");
        
        // Load environment for Hedera credentials
        dotenvy::from_filename(".env.hedera").ok();
        
        let wallet = RealHederaWallet::new(&self.config.hedera.network)?;
        
        println!("✅ Hedera wallet initialized ({})", self.config.hedera.network);
        println!("📍 Public Key: {}", wallet.public_key_hex());
        
        if self.config.hedera.network == "testnet" {
            println!("💡 Get testnet HBAR: https://portal.hedera.com/");
        }
        
        self.hedera = Some(wallet);
        Ok(())
    }
    
    pub async fn get_hedera_wallet(&self, _user_id: &str) -> Result<(String, String)> {
        if let Some(wallet) = &self.hedera {
            let account_id = wallet.account_id.clone().unwrap_or_else(|| "0.0.pending".to_string());
            let balance = wallet.get_balance().await.unwrap_or(0);
            let hbar = balance as f64 / 100_000_000.0;
            Ok((account_id, format!("{:.2}", hbar)))
        } else {
            Err(anyhow::anyhow!("Hedera wallet not initialized"))
        }
    }

    // =========================================================================
    // Monero Integration
    // =========================================================================
    
    pub async fn init_monero(&mut self) -> Result<()> {
        println!("🔄 Initializing Monero wallet...");
        
        let network = match self.mode {
            WalletMode::Testnet => "stagenet",
            WalletMode::Mainnet => "mainnet",
            _ => "stagenet",
        };
        
        let wallet = RealMoneroWallet::new(network)?;
        
        println!("✅ Monero wallet initialized ({})", network);
        println!("📍 Address: {}...{}", &wallet.address[..12], &wallet.address[wallet.address.len()-12..]);
        
        self.monero = Some(wallet);
        Ok(())
    }
    
    pub async fn get_monero_wallet(&self, _user_id: &str) -> Result<(String, String)> {
        if let Some(wallet) = &self.monero {
            Ok((wallet.address.clone(), "0.0".to_string()))
        } else {
            Err(anyhow::anyhow!("Monero wallet not initialized"))
        }
    }

    // =========================================================================
    // ICP Integration
    // =========================================================================
    
    pub async fn init_icp(&mut self) -> Result<()> {
        println!("🔄 Initializing ICP wallet...");
        
        let wallet = RealIcpWallet::new(&self.config.icp.network)?;
        
        println!("✅ ICP wallet initialized ({})", self.config.icp.network);
        println!("📍 Principal: {}", wallet.principal_id());
        
        self.icp = Some(wallet);
        Ok(())
    }
    
    pub async fn get_icp_wallet(&self, _user_id: &str) -> Result<(String, String)> {
        if let Some(wallet) = &self.icp {
            Ok((wallet.principal_id(), "0.0".to_string()))
        } else {
            Err(anyhow::anyhow!("ICP wallet not initialized"))
        }
    }
}
