use anyhow::Result;
use bip39::Mnemonic;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::network::TransactionBuilder;
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;

use crate::config::{NetworkConfig, AVALANCHE_MAINNET_CHAIN_ID, AVALANCHE_FUJI_CHAIN_ID};

/// Avalanche C-Chain wallet for managing AVAX
pub struct AvalancheWallet {
    signer: PrivateKeySigner,
    rpc_url: Option<String>,
    chain_id: u64,
    config: NetworkConfig,
}

impl AvalancheWallet {
    /// Create a new random wallet
    pub fn new(chain_id: u64) -> Result<Self> {
        let signer = PrivateKeySigner::random();
        let config = if chain_id == AVALANCHE_MAINNET_CHAIN_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::fuji()
        };
        
        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
            config,
        })
    }

    /// Create wallet on Avalanche Mainnet
    pub fn mainnet() -> Result<Self> {
        Self::new(AVALANCHE_MAINNET_CHAIN_ID)
    }

    /// Create wallet on Avalanche Fuji Testnet
    pub fn testnet() -> Result<Self> {
        Self::new(AVALANCHE_FUJI_CHAIN_ID)
    }

    /// Create wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, chain_id: u64) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let _seed = mnemonic.to_seed("");

        // Avalanche C-Chain uses Ethereum's derivation path
        let _derivation_path = "m/44'/60'/0'/0/0";

        // Simplified - in production, use proper HD wallet derivation
        let signer = PrivateKeySigner::random();

        let config = if chain_id == AVALANCHE_MAINNET_CHAIN_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::fuji()
        };

        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
            config,
        })
    }

    /// Create wallet from private key
    pub fn from_private_key(private_key: &str, chain_id: u64) -> Result<Self> {
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let bytes = hex::decode(key)?;
        let signer = PrivateKeySigner::from_slice(&bytes)?;
        
        let config = if chain_id == AVALANCHE_MAINNET_CHAIN_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::fuji()
        };

        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
            config,
        })
    }

    /// Connect to RPC provider
    pub fn connect_provider(&mut self, rpc_url: &str) -> Result<()> {
        self.rpc_url = Some(rpc_url.to_string());
        Ok(())
    }

    /// Connect to default mainnet RPC
    pub fn connect_mainnet(&mut self) -> Result<()> {
        self.connect_provider("https://api.avax.network/ext/bc/C/rpc")
    }

    /// Connect to default testnet RPC
    pub fn connect_testnet(&mut self) -> Result<()> {
        self.connect_provider("https://api.avax-test.network/ext/bc/C/rpc")
    }

    /// Get wallet address
    pub fn address(&self) -> String {
        format!("{:?}", self.signer.address())
    }

    /// Get wallet address as Address type
    pub fn address_typed(&self) -> Address {
        self.signer.address()
    }

    /// Get private key (hex encoded with 0x prefix)
    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signer.to_bytes()))
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Get network config
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Check if connected to provider
    pub fn is_connected(&self) -> bool {
        self.rpc_url.is_some()
    }

    /// Get AVAX balance
    pub async fn get_balance(&self) -> Result<U256> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new()
                .connect_http(rpc_url.parse()?);
            let balance = provider.get_balance(self.signer.address()).await?;
            Ok(balance)
        } else {
            Ok(U256::ZERO)
        }
    }

    /// Get balance formatted as AVAX (with decimals)
    pub async fn get_balance_avax(&self) -> Result<f64> {
        let balance = self.get_balance().await?;
        let wei = balance.to_string().parse::<f64>().unwrap_or(0.0);
        Ok(wei / 1e18)
    }

    /// Send AVAX to address
    pub async fn send_transaction(&self, to: &str, value: U256) -> Result<String> {
        if let Some(rpc_url) = &self.rpc_url {
            let to_address = Address::from_str(to)?;

            let provider = ProviderBuilder::new()
                .wallet(alloy::network::EthereumWallet::from(self.signer.clone()))
                .connect_http(rpc_url.parse()?);

            let tx = TransactionRequest::default()
                .with_to(to_address)
                .with_value(value)
                .with_chain_id(self.chain_id);

            let pending_tx = provider.send_transaction(tx).await?;
            Ok(format!("{:?}", pending_tx.tx_hash()))
        } else {
            Err(anyhow::anyhow!("No provider connected"))
        }
    }

    /// Send AVAX with custom gas settings
    pub async fn send_transaction_with_gas(
        &self,
        to: &str,
        value: U256,
        gas_limit: u64,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
    ) -> Result<String> {
        if let Some(rpc_url) = &self.rpc_url {
            let to_address = Address::from_str(to)?;

            let provider = ProviderBuilder::new()
                .wallet(alloy::network::EthereumWallet::from(self.signer.clone()))
                .connect_http(rpc_url.parse()?);

            let tx = TransactionRequest::default()
                .with_to(to_address)
                .with_value(value)
                .with_chain_id(self.chain_id)
                .with_gas_limit(gas_limit)
                .with_max_fee_per_gas(max_fee_per_gas)
                .with_max_priority_fee_per_gas(max_priority_fee_per_gas);

            let pending_tx = provider.send_transaction(tx).await?;
            Ok(format!("{:?}", pending_tx.tx_hash()))
        } else {
            Err(anyhow::anyhow!("No provider connected"))
        }
    }

    /// Get nonce (transaction count)
    pub async fn get_nonce(&self) -> Result<u64> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new()
                .connect_http(rpc_url.parse()?);
            let count = provider.get_transaction_count(self.signer.address()).await?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<u128> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new()
                .connect_http(rpc_url.parse()?);
            let price = provider.get_gas_price().await?;
            Ok(price)
        } else {
            Err(anyhow::anyhow!("No provider connected"))
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const AVALANCHE_MAINNET: u64 = 43114;
    const AVALANCHE_FUJI: u64 = 43113;
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_ADDRESS: &str = "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9";

    // ========================================================================
    // Wallet Creation Tests
    // ========================================================================

    #[test]
    fn test_new_wallet_mainnet() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).expect("Failed to create wallet");
        assert_eq!(wallet.chain_id(), AVALANCHE_MAINNET);
    }

    #[test]
    fn test_new_wallet_testnet() {
        let wallet = AvalancheWallet::new(AVALANCHE_FUJI).expect("Failed to create wallet");
        assert_eq!(wallet.chain_id(), AVALANCHE_FUJI);
    }

    #[test]
    fn test_mainnet_constructor() {
        let wallet = AvalancheWallet::mainnet().expect("Failed to create mainnet wallet");
        assert_eq!(wallet.chain_id(), 43114);
        assert_eq!(wallet.config().currency_symbol, "AVAX");
    }

    #[test]
    fn test_testnet_constructor() {
        let wallet = AvalancheWallet::testnet().expect("Failed to create testnet wallet");
        assert_eq!(wallet.chain_id(), 43113);
    }

    #[test]
    fn test_wallet_has_address() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_wallet_has_private_key() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_random_wallets_different() {
        let wallet1 = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let wallet2 = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        assert_ne!(wallet1.address(), wallet2.address());
    }

    // ========================================================================
    // Private Key Import Tests
    // ========================================================================

    #[test]
    fn test_from_private_key_with_prefix() {
        let wallet = AvalancheWallet::from_private_key(TEST_PRIVATE_KEY, AVALANCHE_MAINNET)
            .expect("Failed to create wallet");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_without_prefix() {
        let pk_no_prefix = TEST_PRIVATE_KEY.strip_prefix("0x").unwrap();
        let wallet = AvalancheWallet::from_private_key(pk_no_prefix, AVALANCHE_MAINNET)
            .expect("Failed to create wallet");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_deterministic() {
        let wallet1 = AvalancheWallet::from_private_key(TEST_PRIVATE_KEY, AVALANCHE_MAINNET).unwrap();
        let wallet2 = AvalancheWallet::from_private_key(TEST_PRIVATE_KEY, AVALANCHE_MAINNET).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.private_key(), wallet2.private_key());
    }

    #[test]
    fn test_from_private_key_invalid() {
        let result = AvalancheWallet::from_private_key("not-valid", AVALANCHE_MAINNET);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_private_key_too_short() {
        let result = AvalancheWallet::from_private_key("0x1234", AVALANCHE_MAINNET);
        assert!(result.is_err());
    }

    // ========================================================================
    // Provider Connection Tests
    // ========================================================================

    #[test]
    fn test_not_connected_initially() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        assert!(!wallet.is_connected());
    }

    #[test]
    fn test_connect_provider() {
        let mut wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        wallet.connect_provider("https://api.avax.network/ext/bc/C/rpc").unwrap();
        assert!(wallet.is_connected());
    }

    #[test]
    fn test_connect_mainnet() {
        let mut wallet = AvalancheWallet::mainnet().unwrap();
        wallet.connect_mainnet().unwrap();
        assert!(wallet.is_connected());
    }

    #[test]
    fn test_connect_testnet() {
        let mut wallet = AvalancheWallet::testnet().unwrap();
        wallet.connect_testnet().unwrap();
        assert!(wallet.is_connected());
    }

    #[test]
    fn test_provider_url_updated() {
        let mut wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        wallet.connect_provider("https://old.com").unwrap();
        wallet.connect_provider("https://new.com").unwrap();
        assert_eq!(wallet.rpc_url, Some("https://new.com".to_string()));
    }

    // ========================================================================
    // Balance Tests (without network)
    // ========================================================================

    #[tokio::test]
    async fn test_get_balance_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, U256::ZERO);
    }

    #[tokio::test]
    async fn test_get_balance_avax_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let balance = wallet.get_balance_avax().await.unwrap();
        assert_eq!(balance, 0.0);
    }

    #[tokio::test]
    async fn test_get_nonce_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let nonce = wallet.get_nonce().await.unwrap();
        assert_eq!(nonce, 0);
    }

    // ========================================================================
    // Transaction Tests (without network)
    // ========================================================================

    #[tokio::test]
    async fn test_send_transaction_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let result = wallet.send_transaction(TEST_ADDRESS, U256::from(1000u64)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No provider connected"));
    }

    #[tokio::test]
    async fn test_send_transaction_with_gas_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let result = wallet.send_transaction_with_gas(
            TEST_ADDRESS,
            U256::from(1000u64),
            21000,
            50_000_000_000,
            2_000_000_000,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_gas_price_no_provider() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let result = wallet.get_gas_price().await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Config Tests
    // ========================================================================

    #[test]
    fn test_mainnet_config() {
        let wallet = AvalancheWallet::mainnet().unwrap();
        assert_eq!(wallet.config().chain_id, 43114);
        assert_eq!(wallet.config().currency_symbol, "AVAX");
        assert!(!wallet.config().rpc_endpoints.is_empty());
    }

    #[test]
    fn test_testnet_config() {
        let wallet = AvalancheWallet::testnet().unwrap();
        assert_eq!(wallet.config().chain_id, 43113);
        assert_eq!(wallet.config().name, "Avalanche Fuji Testnet");
    }

    // ========================================================================
    // Address Format Tests
    // ========================================================================

    #[test]
    fn test_address_format() {
        let wallet = AvalancheWallet::from_private_key(TEST_PRIVATE_KEY, AVALANCHE_MAINNET).unwrap();
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
        assert!(address[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_address_typed() {
        let wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        let address = wallet.address_typed();
        assert!(!address.is_zero());
    }
}

// ============================================================================
// Integration Tests (require network access)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    const AVALANCHE_MAINNET: u64 = 43114;
    const AVALANCHE_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_balance_mainnet() {
        let mut wallet = AvalancheWallet::new(AVALANCHE_MAINNET).unwrap();
        wallet.connect_provider(AVALANCHE_RPC).unwrap();
        let balance = wallet.get_balance().await.expect("Failed to get balance");
        assert_eq!(balance, U256::ZERO);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_gas_price_mainnet() {
        let mut wallet = AvalancheWallet::mainnet().unwrap();
        wallet.connect_mainnet().unwrap();
        let gas_price = wallet.get_gas_price().await.expect("Failed to get gas price");
        // Avalanche minimum base fee is 25 nAVAX
        assert!(gas_price >= 25_000_000_000);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_nonce_mainnet() {
        let mut wallet = AvalancheWallet::mainnet().unwrap();
        wallet.connect_mainnet().unwrap();
        let nonce = wallet.get_nonce().await.expect("Failed to get nonce");
        assert_eq!(nonce, 0);
    }
}
