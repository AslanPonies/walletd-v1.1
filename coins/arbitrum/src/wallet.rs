//! Arbitrum wallet implementation

use crate::config::{NetworkConfig, ARBITRUM_ONE_CHAIN_ID, ARBITRUM_SEPOLIA_CHAIN_ID};
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use anyhow::Result;
use bip39::Mnemonic;
use std::str::FromStr;

/// Arbitrum wallet for managing accounts and transactions
pub struct ArbitrumWallet {
    signer: PrivateKeySigner,
    rpc_url: Option<String>,
    chain_id: u64,
}

impl ArbitrumWallet {
    /// Create a new random wallet
    pub fn new(chain_id: u64) -> Result<Self> {
        let signer = PrivateKeySigner::random();
        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    /// Create wallet for Arbitrum One mainnet
    pub fn mainnet() -> Result<Self> {
        Self::new(ARBITRUM_ONE_CHAIN_ID)
    }

    /// Create wallet for Arbitrum Sepolia testnet
    pub fn sepolia() -> Result<Self> {
        Self::new(ARBITRUM_SEPOLIA_CHAIN_ID)
    }

    /// Create wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, chain_id: u64) -> Result<Self> {
        Self::from_mnemonic_with_index(mnemonic, chain_id, 0)
    }

    /// Create wallet from mnemonic with specific derivation index
    pub fn from_mnemonic_with_index(mnemonic: &str, chain_id: u64, index: u32) -> Result<Self> {
        use bip32::{DerivationPath, XPrv};

        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");

        // Ethereum derivation path: m/44'/60'/0'/0/index
        let path = DerivationPath::from_str(&format!("m/44'/60'/0'/0/{}", index))?;
        let child_xprv = XPrv::derive_from_path(seed, &path)?;
        let private_key_bytes: [u8; 32] = child_xprv.private_key().to_bytes().into();

        let signer = PrivateKeySigner::from_slice(&private_key_bytes)?;

        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    /// Create wallet from private key (hex string)
    pub fn from_private_key(private_key: &str, chain_id: u64) -> Result<Self> {
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let bytes = hex::decode(key)?;
        let signer = PrivateKeySigner::from_slice(&bytes)?;
        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    /// Connect to an RPC provider
    pub fn connect(&mut self, rpc_url: &str) -> &mut Self {
        self.rpc_url = Some(rpc_url.to_string());
        self
    }

    /// Connect using network config
    pub fn connect_network(&mut self, config: &NetworkConfig) -> &mut Self {
        if let Some(rpc) = config.rpc_endpoints.first() {
            self.rpc_url = Some(rpc.clone());
            self.chain_id = config.chain_id;
        }
        self
    }

    /// Get the wallet address
    pub fn address(&self) -> String {
        format!("{:?}", self.signer.address())
    }

    /// Get the wallet address as Address type
    pub fn address_raw(&self) -> Address {
        self.signer.address()
    }

    /// Get the private key (hex with 0x prefix)
    /// ⚠️ Handle with care!
    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signer.to_bytes()))
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Check if connected to a provider
    pub fn is_connected(&self) -> bool {
        self.rpc_url.is_some()
    }

    /// Get the current RPC URL
    pub fn rpc_url(&self) -> Option<&str> {
        self.rpc_url.as_deref()
    }

    /// Get ETH balance
    pub async fn get_balance(&self) -> Result<U256> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
            let balance = provider.get_balance(self.signer.address()).await?;
            Ok(balance)
        } else {
            Ok(U256::ZERO)
        }
    }

    /// Get balance formatted as ETH string
    pub async fn get_balance_eth(&self) -> Result<String> {
        let balance = self.get_balance().await?;
        let eth = balance.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
        Ok(format!("{:.6} ETH", eth))
    }

    /// Get transaction count (nonce)
    pub async fn get_nonce(&self) -> Result<u64> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
            let nonce = provider.get_transaction_count(self.signer.address()).await?;
            Ok(nonce)
        } else {
            Ok(0)
        }
    }

    /// Send ETH to an address
    pub async fn send_eth(&self, to: &str, amount: U256) -> Result<String> {
        self.send_transaction(to, amount, None).await
    }

    /// Send a transaction with optional data
    pub async fn send_transaction(
        &self,
        to: &str,
        value: U256,
        data: Option<Vec<u8>>,
    ) -> Result<String> {
        let rpc_url = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No provider connected"))?;

        let to_address = Address::from_str(to)?;

        let provider = ProviderBuilder::new()
            .wallet(alloy::network::EthereumWallet::from(self.signer.clone()))
            .connect_http(rpc_url.parse()?);

        let mut tx = TransactionRequest::default()
            .with_to(to_address)
            .with_value(value)
            .with_chain_id(self.chain_id);

        if let Some(data) = data {
            tx = tx.with_input(data);
        }

        let pending_tx = provider.send_transaction(tx).await?;
        Ok(format!("{:?}", pending_tx.tx_hash()))
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, to: &str, value: U256, data: Option<Vec<u8>>) -> Result<u64> {
        let rpc_url = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No provider connected"))?;

        let to_address = Address::from_str(to)?;

        let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);

        let mut tx = TransactionRequest::default()
            .with_from(self.signer.address())
            .with_to(to_address)
            .with_value(value);

        if let Some(data) = data {
            tx = tx.with_input(data);
        }

        let gas = provider.estimate_gas(tx).await?;
        Ok(gas)
    }

    /// Get current gas price
    pub async fn gas_price(&self) -> Result<u128> {
        let rpc_url = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No provider connected"))?;

        let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
        let price = provider.get_gas_price().await?;
        Ok(price)
    }

    /// Sign a message (EIP-191 personal sign)
    pub async fn sign_message(&self, message: &str) -> Result<String> {
        use alloy::signers::Signer;
        let signature = self.signer.sign_message(message.as_bytes()).await?;
        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }
}

impl std::fmt::Debug for ArbitrumWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArbitrumWallet")
            .field("address", &self.address())
            .field("chain_id", &self.chain_id)
            .field("connected", &self.is_connected())
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_PRIVATE_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    // ============================================================================
    // Wallet Creation Tests
    // ============================================================================

    #[test]
    fn test_new_wallet_mainnet() {
        let wallet = ArbitrumWallet::mainnet().expect("Failed to create wallet");
        assert_eq!(wallet.chain_id(), ARBITRUM_ONE_CHAIN_ID);
    }

    #[test]
    fn test_new_wallet_sepolia() {
        let wallet = ArbitrumWallet::sepolia().expect("Failed to create wallet");
        assert_eq!(wallet.chain_id(), ARBITRUM_SEPOLIA_CHAIN_ID);
    }

    #[test]
    fn test_new_wallet_has_address() {
        let wallet = ArbitrumWallet::mainnet().expect("Failed to create wallet");
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_new_wallet_has_private_key() {
        let wallet = ArbitrumWallet::mainnet().expect("Failed to create wallet");
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_new_wallet_random() {
        let wallet1 = ArbitrumWallet::mainnet().expect("Failed to create wallet");
        let wallet2 = ArbitrumWallet::mainnet().expect("Failed to create wallet");
        assert_ne!(wallet1.address(), wallet2.address());
    }

    // ============================================================================
    // Mnemonic Tests
    // ============================================================================

    #[test]
    fn test_from_mnemonic() {
        let wallet =
            ArbitrumWallet::from_mnemonic(TEST_MNEMONIC, ARBITRUM_ONE_CHAIN_ID).expect("Failed");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let wallet1 =
            ArbitrumWallet::from_mnemonic(TEST_MNEMONIC, ARBITRUM_ONE_CHAIN_ID).expect("Failed");
        let wallet2 =
            ArbitrumWallet::from_mnemonic(TEST_MNEMONIC, ARBITRUM_ONE_CHAIN_ID).expect("Failed");
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_from_mnemonic_different_indices() {
        let wallet0 =
            ArbitrumWallet::from_mnemonic_with_index(TEST_MNEMONIC, ARBITRUM_ONE_CHAIN_ID, 0)
                .expect("Failed");
        let wallet1 =
            ArbitrumWallet::from_mnemonic_with_index(TEST_MNEMONIC, ARBITRUM_ONE_CHAIN_ID, 1)
                .expect("Failed");
        assert_ne!(wallet0.address(), wallet1.address());
    }

    #[test]
    fn test_from_mnemonic_invalid() {
        let result = ArbitrumWallet::from_mnemonic("invalid mnemonic", ARBITRUM_ONE_CHAIN_ID);
        assert!(result.is_err());
    }

    // ============================================================================
    // Private Key Tests
    // ============================================================================

    #[test]
    fn test_from_private_key_with_0x() {
        let wallet = ArbitrumWallet::from_private_key(TEST_PRIVATE_KEY, ARBITRUM_ONE_CHAIN_ID)
            .expect("Failed");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_without_0x() {
        let pk_without_prefix = TEST_PRIVATE_KEY.strip_prefix("0x").unwrap();
        let wallet =
            ArbitrumWallet::from_private_key(pk_without_prefix, ARBITRUM_ONE_CHAIN_ID).expect("Failed");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_deterministic() {
        let wallet1 = ArbitrumWallet::from_private_key(TEST_PRIVATE_KEY, ARBITRUM_ONE_CHAIN_ID)
            .expect("Failed");
        let wallet2 = ArbitrumWallet::from_private_key(TEST_PRIVATE_KEY, ARBITRUM_ONE_CHAIN_ID)
            .expect("Failed");
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.private_key(), wallet2.private_key());
    }

    #[test]
    fn test_from_private_key_invalid() {
        let result = ArbitrumWallet::from_private_key("not-a-key", ARBITRUM_ONE_CHAIN_ID);
        assert!(result.is_err());
    }

    // ============================================================================
    // Connection Tests
    // ============================================================================

    #[test]
    fn test_connect() {
        let mut wallet = ArbitrumWallet::mainnet().unwrap();
        assert!(!wallet.is_connected());

        wallet.connect("https://arb1.arbitrum.io/rpc");
        assert!(wallet.is_connected());
        assert_eq!(wallet.rpc_url(), Some("https://arb1.arbitrum.io/rpc"));
    }

    #[test]
    fn test_connect_network() {
        let mut wallet = ArbitrumWallet::new(1).unwrap(); // Start with wrong chain
        let config = NetworkConfig::mainnet();

        wallet.connect_network(&config);

        assert!(wallet.is_connected());
        assert_eq!(wallet.chain_id(), ARBITRUM_ONE_CHAIN_ID);
    }

    // ============================================================================
    // Balance Tests (offline)
    // ============================================================================

    #[tokio::test]
    async fn test_get_balance_no_provider() {
        let wallet = ArbitrumWallet::mainnet().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, U256::ZERO);
    }

    // ============================================================================
    // Transaction Tests (offline)
    // ============================================================================

    #[tokio::test]
    async fn test_send_transaction_no_provider() {
        let wallet = ArbitrumWallet::mainnet().unwrap();
        let result = wallet
            .send_eth("0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9", U256::from(1000u64))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No provider connected"));
    }

    // ============================================================================
    // Message Signing Tests
    // ============================================================================

    #[tokio::test]
    async fn test_sign_message() {
        let wallet = ArbitrumWallet::from_private_key(TEST_PRIVATE_KEY, ARBITRUM_ONE_CHAIN_ID)
            .expect("Failed");
        let signature = wallet.sign_message("Hello Arbitrum!").await.unwrap();
        assert!(signature.starts_with("0x"));
        assert_eq!(signature.len(), 132); // 0x + 65 bytes * 2
    }

    #[tokio::test]
    async fn test_sign_message_deterministic() {
        let wallet = ArbitrumWallet::from_private_key(TEST_PRIVATE_KEY, ARBITRUM_ONE_CHAIN_ID)
            .expect("Failed");
        let sig1 = wallet.sign_message("Test").await.unwrap();
        let sig2 = wallet.sign_message("Test").await.unwrap();
        assert_eq!(sig1, sig2);
    }

    // ============================================================================
    // Debug/Display Tests
    // ============================================================================

    #[test]
    fn test_debug_format() {
        let wallet = ArbitrumWallet::mainnet().unwrap();
        let debug = format!("{:?}", wallet);
        assert!(debug.contains("ArbitrumWallet"));
        assert!(debug.contains("address"));
        assert!(debug.contains("chain_id"));
    }
}

// ============================================================================
// Integration Tests (require network)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_balance_mainnet() {
        let mut wallet = ArbitrumWallet::mainnet().unwrap();
        wallet.connect("https://arb1.arbitrum.io/rpc");

        let balance = wallet.get_balance().await.expect("Failed to get balance");
        // New wallet should have 0 balance
        assert_eq!(balance, U256::ZERO);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_gas_price_mainnet() {
        let mut wallet = ArbitrumWallet::mainnet().unwrap();
        wallet.connect("https://arb1.arbitrum.io/rpc");

        let gas_price = wallet.gas_price().await.expect("Failed to get gas price");
        assert!(gas_price > 0);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_estimate_gas_mainnet() {
        let mut wallet = ArbitrumWallet::mainnet().unwrap();
        wallet.connect("https://arb1.arbitrum.io/rpc");

        let gas = wallet
            .estimate_gas(
                "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
                U256::from(0u64),
                None,
            )
            .await
            .expect("Failed to estimate gas");

        // Basic transfer should cost ~21000 gas
        assert!(gas >= 21000);
    }
}
