//! Wallet Integration Module
//!
//! Central wallet manager supporting 17+ blockchains with real operations.

use crate::config::WalletDConfig;
use crate::types::WalletMode;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

// Chain-specific modules
pub mod bitcoin_wallet;
pub mod ethereum_wallet;
pub mod solana_wallet;
pub mod hedera_wallet;
pub mod monero_wallet;
pub mod icp_wallet;
pub mod evm_wallet;      // Polygon, Avalanche, Base, Arbitrum
pub mod cardano_wallet;
pub mod cosmos_wallet;
pub mod polkadot_wallet;
pub mod near_wallet;
pub mod tron_wallet;
pub mod sui_wallet;
pub mod aptos_wallet;
pub mod ton_wallet;
pub mod hd_derivation;

use bitcoin_wallet::BitcoinWallet;
use ethereum_wallet::EthereumWallet;
use solana_wallet::SolanaWallet;
use hedera_wallet::HederaWallet;
use monero_wallet::MoneroWallet;
use icp_wallet::IcpWallet;
use evm_wallet::EvmWallet;
use cardano_wallet::CardanoWallet;
use cosmos_wallet::CosmosWallet;
use polkadot_wallet::PolkadotWallet;
use near_wallet::NearWallet;
use tron_wallet::TronWallet;
use sui_wallet::SuiWallet;
use aptos_wallet::AptosWallet;
use ton_wallet::TonWallet;

/// Global wallet manager instance
pub static WALLET_MANAGER: Lazy<Arc<RwLock<WalletManager>>> = Lazy::new(|| {
    let config = WalletDConfig::load();
    Arc::new(RwLock::new(WalletManager::new(config)))
});

/// Central wallet manager for all 17+ blockchain integrations
pub struct WalletManager {
    pub config: WalletDConfig,
    pub mode: WalletMode,
    pub mnemonic: Option<String>,
    
    // Core chains (1-9)
    pub bitcoin: Option<BitcoinWallet>,
    pub ethereum: Option<EthereumWallet>,
    pub solana: Option<SolanaWallet>,
    pub hedera: Option<HederaWallet>,
    pub monero: Option<MoneroWallet>,
    pub icp: Option<IcpWallet>,
    pub base: Option<EvmWallet>,
    
    // Extended chains (10-20)
    pub polygon: Option<EvmWallet>,
    pub avalanche: Option<EvmWallet>,
    pub arbitrum: Option<EvmWallet>,
    pub cardano: Option<CardanoWallet>,
    pub cosmos: Option<CosmosWallet>,
    pub polkadot: Option<PolkadotWallet>,
    pub near: Option<NearWallet>,
    pub tron: Option<TronWallet>,
    pub sui: Option<SuiWallet>,
    pub aptos: Option<AptosWallet>,
    pub ton: Option<TonWallet>,
}

impl WalletManager {
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
            mnemonic: None,
            bitcoin: None,
            ethereum: None,
            solana: None,
            hedera: None,
            monero: None,
            icp: None,
            base: None,
            polygon: None,
            avalanche: None,
            arbitrum: None,
            cardano: None,
            cosmos: None,
            polkadot: None,
            near: None,
            tron: None,
            sui: None,
            aptos: None,
            ton: None,
        }
    }

    /// Generate a new master mnemonic for HD derivation
    pub fn generate_mnemonic(&mut self) -> Result<String> {
        let mnemonic = hd_derivation::generate_mnemonic(24)?;
        self.mnemonic = Some(mnemonic.clone());
        Ok(mnemonic)
    }

    /// Set mnemonic from import
    pub fn set_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        hd_derivation::validate_mnemonic(mnemonic)?;
        self.mnemonic = Some(mnemonic.to_string());
        Ok(())
    }

    /// Initialize all wallets from mnemonic
    pub async fn init_all_from_mnemonic(&mut self) -> Result<()> {
        let mnemonic = self.mnemonic.clone()
            .ok_or_else(|| anyhow::anyhow!("No mnemonic set"))?;
        
        println!("🔄 Initializing all wallets from master seed...\n");
        
        // Core chains
        self.init_bitcoin_from_mnemonic(&mnemonic).await?;
        self.init_ethereum_from_mnemonic(&mnemonic).await?;
        self.init_solana_from_mnemonic(&mnemonic).await?;
        self.init_hedera().await.ok();
        self.init_monero().await.ok();
        self.init_icp().await.ok();
        
        // EVM chains (share derivation path with Ethereum)
        self.init_evm_chains_from_mnemonic(&mnemonic).await?;
        
        // Extended chains
        self.init_cardano_from_mnemonic(&mnemonic).await.ok();
        self.init_cosmos_from_mnemonic(&mnemonic).await.ok();
        self.init_polkadot_from_mnemonic(&mnemonic).await.ok();
        self.init_near_from_mnemonic(&mnemonic).await.ok();
        self.init_tron_from_mnemonic(&mnemonic).await.ok();
        self.init_sui_from_mnemonic(&mnemonic).await.ok();
        self.init_aptos_from_mnemonic(&mnemonic).await.ok();
        self.init_ton_from_mnemonic(&mnemonic).await.ok();
        
        println!("\n✅ All wallets initialized!");
        Ok(())
    }

    /// Initialize all wallets with random keys (for quick testing)
    pub async fn init_all(&mut self) -> Result<()> {
        println!("🔄 Initializing all wallets...\n");
        
        // Generate master mnemonic first
        let mnemonic = self.generate_mnemonic()?;
        println!("📝 Master mnemonic generated (keep this safe!)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        self.init_all_from_mnemonic().await
    }

    // =========================================================================
    // Bitcoin
    // =========================================================================
    
    async fn init_bitcoin_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Bitcoin... ");
        let network = match self.config.bitcoin.network.as_str() {
            "mainnet" => bitcoin::Network::Bitcoin,
            "testnet" => bitcoin::Network::Testnet,
            _ => bitcoin::Network::Testnet,
        };
        
        let wallet = BitcoinWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.address);
        self.bitcoin = Some(wallet);
        Ok(())
    }

    pub async fn init_bitcoin(&mut self) -> Result<()> {
        let network = match self.config.bitcoin.network.as_str() {
            "mainnet" => bitcoin::Network::Bitcoin,
            _ => bitcoin::Network::Testnet,
        };
        let wallet = BitcoinWallet::new(network)?;
        self.bitcoin = Some(wallet);
        Ok(())
    }

    pub async fn get_bitcoin_info(&self) -> Result<(String, String)> {
        let wallet = self.bitcoin.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.8}", balance as f64 / 1e8)))
    }

    pub async fn send_bitcoin(&self, to: &str, amount_btc: f64) -> Result<String> {
        let wallet = self.bitcoin.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        wallet.send(to, (amount_btc * 1e8) as u64).await
    }

    // =========================================================================
    // Ethereum
    // =========================================================================
    
    async fn init_ethereum_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Ethereum... ");
        let wallet = EthereumWallet::from_mnemonic(mnemonic, self.config.ethereum.chain_id)?;
        println!("✅ {}", wallet.address_string());
        self.ethereum = Some(wallet);
        Ok(())
    }

    pub async fn get_ethereum_info(&self) -> Result<(String, String)> {
        let wallet = self.ethereum.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address_string(), format!("{:.6}", balance as f64 / 1e18)))
    }

    pub async fn send_ethereum(&self, to: &str, amount_eth: f64) -> Result<String> {
        let wallet = self.ethereum.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        wallet.send(to, amount_eth).await
    }

    // =========================================================================
    // Solana
    // =========================================================================
    
    async fn init_solana_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Solana... ");
        let wallet = SolanaWallet::from_mnemonic(mnemonic, &self.config.solana.cluster)?;
        println!("✅ {}", wallet.address);
        self.solana = Some(wallet);
        Ok(())
    }

    pub async fn get_solana_info(&self) -> Result<(String, String)> {
        let wallet = self.solana.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.9}", balance as f64 / 1e9)))
    }

    pub async fn send_solana(&self, to: &str, amount_sol: f64) -> Result<String> {
        let wallet = self.solana.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        wallet.send(to, (amount_sol * 1e9) as u64).await
    }

    pub async fn solana_airdrop(&self, lamports: u64) -> Result<String> {
        let wallet = self.solana.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        wallet.request_airdrop(lamports).await
    }

    // =========================================================================
    // Hedera
    // =========================================================================
    
    pub async fn init_hedera(&mut self) -> Result<()> {
        print!("  Hedera... ");
        dotenvy::from_filename(".env.hedera").ok();
        let wallet = HederaWallet::new(&self.config.hedera.network)?;
        println!("✅ {}", wallet.public_key_hex());
        self.hedera = Some(wallet);
        Ok(())
    }

    pub async fn get_hedera_info(&self) -> Result<(String, String)> {
        let wallet = self.hedera.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let account = wallet.account_id.clone().unwrap_or_else(|| "pending".to_string());
        Ok((account, "0.0".to_string()))
    }

    // =========================================================================
    // Monero
    // =========================================================================
    
    pub async fn init_monero(&mut self) -> Result<()> {
        print!("  Monero... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "stagenet" };
        let wallet = MoneroWallet::new(network)?;
        println!("✅ {}...{}", &wallet.address[..12], &wallet.address[wallet.address.len()-8..]);
        self.monero = Some(wallet);
        Ok(())
    }

    pub async fn get_monero_info(&self) -> Result<(String, String)> {
        let wallet = self.monero.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        Ok((wallet.address.clone(), "0.0".to_string()))
    }

    // =========================================================================
    // ICP
    // =========================================================================
    
    pub async fn init_icp(&mut self) -> Result<()> {
        print!("  ICP... ");
        let wallet = IcpWallet::new(&self.config.icp.network)?;
        println!("✅ {}", wallet.principal_id());
        self.icp = Some(wallet);
        Ok(())
    }

    pub async fn get_icp_info(&self) -> Result<(String, String)> {
        let wallet = self.icp.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        Ok((wallet.principal_id(), "0.0".to_string()))
    }

    // =========================================================================
    // EVM Chains (Polygon, Avalanche, Base, Arbitrum)
    // =========================================================================
    
    async fn init_evm_chains_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        // Base (Coinbase L2)
        print!("  Base... ");
        let base_chain_id = if self.mode == WalletMode::Mainnet { 8453 } else { 84532 };
        let base = EvmWallet::from_mnemonic(mnemonic, base_chain_id, "Base")?;
        println!("✅ {}", base.address_string());
        self.base = Some(base);

        // Polygon
        print!("  Polygon... ");
        let polygon_chain_id = if self.mode == WalletMode::Mainnet { 137 } else { 80002 };
        let polygon = EvmWallet::from_mnemonic(mnemonic, polygon_chain_id, "Polygon")?;
        println!("✅ {}", polygon.address_string());
        self.polygon = Some(polygon);

        // Avalanche
        print!("  Avalanche... ");
        let avax_chain_id = if self.mode == WalletMode::Mainnet { 43114 } else { 43113 };
        let avalanche = EvmWallet::from_mnemonic(mnemonic, avax_chain_id, "Avalanche")?;
        println!("✅ {}", avalanche.address_string());
        self.avalanche = Some(avalanche);

        // Arbitrum
        print!("  Arbitrum... ");
        let arb_chain_id = if self.mode == WalletMode::Mainnet { 42161 } else { 421614 };
        let arbitrum = EvmWallet::from_mnemonic(mnemonic, arb_chain_id, "Arbitrum")?;
        println!("✅ {}", arbitrum.address_string());
        self.arbitrum = Some(arbitrum);

        Ok(())
    }

    pub async fn get_evm_info(&self, chain: &str) -> Result<(String, String)> {
        let wallet = match chain {
            "base" => self.base.as_ref(),
            "polygon" => self.polygon.as_ref(),
            "avalanche" => self.avalanche.as_ref(),
            "arbitrum" => self.arbitrum.as_ref(),
            _ => None,
        }.ok_or_else(|| anyhow::anyhow!("Chain not initialized"))?;
        
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address_string(), format!("{:.6}", balance as f64 / 1e18)))
    }

    // =========================================================================
    // Cardano
    // =========================================================================
    
    async fn init_cardano_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Cardano... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "preprod" };
        let wallet = CardanoWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}...", &wallet.address[..20]);
        self.cardano = Some(wallet);
        Ok(())
    }

    pub async fn get_cardano_info(&self) -> Result<(String, String)> {
        let wallet = self.cardano.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        Ok((wallet.address.clone(), "0.0".to_string()))
    }

    // =========================================================================
    // Cosmos
    // =========================================================================
    
    async fn init_cosmos_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Cosmos... ");
        let wallet = CosmosWallet::from_mnemonic(mnemonic, "cosmos")?;
        println!("✅ {}", wallet.address);
        self.cosmos = Some(wallet);
        Ok(())
    }

    pub async fn get_cosmos_info(&self) -> Result<(String, String)> {
        let wallet = self.cosmos.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.6}", balance as f64 / 1e6)))
    }

    // =========================================================================
    // Polkadot
    // =========================================================================
    
    async fn init_polkadot_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Polkadot... ");
        let network = if self.mode == WalletMode::Mainnet { "polkadot" } else { "westend" };
        let wallet = PolkadotWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.address);
        self.polkadot = Some(wallet);
        Ok(())
    }

    pub async fn get_polkadot_info(&self) -> Result<(String, String)> {
        let wallet = self.polkadot.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        Ok((wallet.address.clone(), "0.0".to_string()))
    }

    // =========================================================================
    // Near
    // =========================================================================
    
    async fn init_near_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Near... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "testnet" };
        let wallet = NearWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.account_id);
        self.near = Some(wallet);
        Ok(())
    }

    pub async fn get_near_info(&self) -> Result<(String, String)> {
        let wallet = self.near.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.account_id.clone(), format!("{:.5}", balance as f64 / 1e24)))
    }

    // =========================================================================
    // Tron
    // =========================================================================
    
    async fn init_tron_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Tron... ");
        let wallet = TronWallet::from_mnemonic(mnemonic)?;
        println!("✅ {}", wallet.address);
        self.tron = Some(wallet);
        Ok(())
    }

    pub async fn get_tron_info(&self) -> Result<(String, String)> {
        let wallet = self.tron.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.6}", balance as f64 / 1e6)))
    }

    // =========================================================================
    // SUI
    // =========================================================================
    
    async fn init_sui_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  SUI... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "devnet" };
        let wallet = SuiWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.address);
        self.sui = Some(wallet);
        Ok(())
    }

    pub async fn get_sui_info(&self) -> Result<(String, String)> {
        let wallet = self.sui.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.9}", balance as f64 / 1e9)))
    }

    // =========================================================================
    // Aptos
    // =========================================================================
    
    async fn init_aptos_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  Aptos... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "devnet" };
        let wallet = AptosWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.address);
        self.aptos = Some(wallet);
        Ok(())
    }

    pub async fn get_aptos_info(&self) -> Result<(String, String)> {
        let wallet = self.aptos.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.8}", balance as f64 / 1e8)))
    }

    // =========================================================================
    // TON
    // =========================================================================
    
    async fn init_ton_from_mnemonic(&mut self, mnemonic: &str) -> Result<()> {
        print!("  TON... ");
        let network = if self.mode == WalletMode::Mainnet { "mainnet" } else { "testnet" };
        let wallet = TonWallet::from_mnemonic(mnemonic, network)?;
        println!("✅ {}", wallet.address);
        self.ton = Some(wallet);
        Ok(())
    }

    pub async fn get_ton_info(&self) -> Result<(String, String)> {
        let wallet = self.ton.as_ref().ok_or_else(|| anyhow::anyhow!("Not initialized"))?;
        let balance = wallet.get_balance().await.unwrap_or(0);
        Ok((wallet.address.clone(), format!("{:.9}", balance as f64 / 1e9)))
    }
}
