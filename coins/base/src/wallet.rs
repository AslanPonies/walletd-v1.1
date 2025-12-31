use anyhow::Result;
use bip39::Mnemonic;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::network::TransactionBuilder;
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;

pub struct BaseWallet {
    signer: PrivateKeySigner,
    rpc_url: Option<String>,
    chain_id: u64,
}

impl BaseWallet {
    pub fn new(chain_id: u64) -> Result<Self> {
        let signer = PrivateKeySigner::random();
        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    pub fn from_mnemonic(mnemonic: &str, chain_id: u64) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let _seed = mnemonic.to_seed("");

        // Use Ethereum's derivation path for now (Base is compatible)
        let _derivation_path = "m/44'/60'/0'/0/0";

        // This is simplified - in production, use proper HD wallet derivation
        let signer = PrivateKeySigner::random();

        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    pub fn from_private_key(private_key: &str, chain_id: u64) -> Result<Self> {
        // Strip 0x prefix if present
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let bytes = hex::decode(key)?;
        let signer = PrivateKeySigner::from_slice(&bytes)?;
        Ok(Self {
            signer,
            rpc_url: None,
            chain_id,
        })
    }

    pub fn connect_provider(&mut self, rpc_url: &str) -> Result<()> {
        self.rpc_url = Some(rpc_url.to_string());
        Ok(())
    }

    pub fn address(&self) -> String {
        format!("{:?}", self.signer.address())
    }

    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signer.to_bytes()))
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Base chain IDs
    const BASE_MAINNET: u64 = 8453;
    const BASE_SEPOLIA: u64 = 84532;

    // Test private key (DO NOT USE IN PRODUCTION)
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    // ============================================================================
    // Wallet Creation Tests
    // ============================================================================

    #[test]
    fn test_new_wallet_mainnet() {
        let wallet = BaseWallet::new(BASE_MAINNET).expect("Failed to create wallet");
        assert_eq!(wallet.chain_id, BASE_MAINNET);
    }

    #[test]
    fn test_new_wallet_sepolia() {
        let wallet = BaseWallet::new(BASE_SEPOLIA).expect("Failed to create wallet");
        assert_eq!(wallet.chain_id, BASE_SEPOLIA);
    }

    #[test]
    fn test_new_wallet_has_address() {
        let wallet = BaseWallet::new(BASE_MAINNET).expect("Failed to create wallet");
        let address = wallet.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_new_wallet_has_private_key() {
        let wallet = BaseWallet::new(BASE_MAINNET).expect("Failed to create wallet");
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_new_wallet_random_addresses() {
        let wallet1 = BaseWallet::new(BASE_MAINNET).expect("Failed to create wallet");
        let wallet2 = BaseWallet::new(BASE_MAINNET).expect("Failed to create wallet");
        // Random wallets should have different addresses
        assert_ne!(wallet1.address(), wallet2.address());
    }

    // ============================================================================
    // Private Key Import Tests
    // ============================================================================

    #[test]
    fn test_from_private_key_with_0x() {
        let wallet = BaseWallet::from_private_key(TEST_PRIVATE_KEY, BASE_MAINNET)
            .expect("Failed to create wallet from private key");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_without_0x() {
        let pk_without_prefix = TEST_PRIVATE_KEY.strip_prefix("0x").unwrap();
        let wallet = BaseWallet::from_private_key(pk_without_prefix, BASE_MAINNET)
            .expect("Failed to create wallet from private key");
        assert!(wallet.address().starts_with("0x"));
    }

    #[test]
    fn test_from_private_key_deterministic() {
        let wallet1 = BaseWallet::from_private_key(TEST_PRIVATE_KEY, BASE_MAINNET).unwrap();
        let wallet2 = BaseWallet::from_private_key(TEST_PRIVATE_KEY, BASE_MAINNET).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.private_key(), wallet2.private_key());
    }

    #[test]
    fn test_from_private_key_invalid() {
        let result = BaseWallet::from_private_key("not-a-valid-key", BASE_MAINNET);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_private_key_too_short() {
        let result = BaseWallet::from_private_key("0x1234", BASE_MAINNET);
        assert!(result.is_err());
    }

    // ============================================================================
    // Provider Connection Tests
    // ============================================================================

    #[test]
    fn test_connect_provider() {
        let mut wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        assert!(wallet.rpc_url.is_none());
        
        wallet.connect_provider("https://mainnet.base.org").unwrap();
        assert_eq!(wallet.rpc_url, Some("https://mainnet.base.org".to_string()));
    }

    #[test]
    fn test_connect_provider_updates_url() {
        let mut wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        wallet.connect_provider("https://old-url.com").unwrap();
        wallet.connect_provider("https://new-url.com").unwrap();
        assert_eq!(wallet.rpc_url, Some("https://new-url.com".to_string()));
    }

    // ============================================================================
    // Balance Tests (without network)
    // ============================================================================

    #[tokio::test]
    async fn test_get_balance_no_provider() {
        let wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        let balance = wallet.get_balance().await.unwrap();
        // Without provider, balance should be zero
        assert_eq!(balance, U256::ZERO);
    }

    // ============================================================================
    // Transaction Tests (without network)
    // ============================================================================

    #[tokio::test]
    async fn test_send_transaction_no_provider() {
        let wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        let result = wallet.send_transaction(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64)
        ).await;
        
        // Without provider, should fail
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No provider connected"));
    }

    // ============================================================================
    // Chain ID Tests
    // ============================================================================

    #[test]
    fn test_chain_id_preserved() {
        let mainnet = BaseWallet::new(BASE_MAINNET).unwrap();
        let sepolia = BaseWallet::new(BASE_SEPOLIA).unwrap();
        
        assert_eq!(mainnet.chain_id, 8453);
        assert_eq!(sepolia.chain_id, 84532);
    }
}

// ============================================================================
// Integration Tests (require network access)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    const BASE_MAINNET_RPC: &str = "https://mainnet.base.org";
    const BASE_MAINNET: u64 = 8453;

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_balance_mainnet() {
        let mut wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        wallet.connect_provider(BASE_MAINNET_RPC).unwrap();
        
        // New wallet should have 0 balance
        let balance = wallet.get_balance().await.expect("Failed to get balance");
        assert_eq!(balance, U256::ZERO);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_provider_connection_mainnet() {
        let mut wallet = BaseWallet::new(BASE_MAINNET).unwrap();
        wallet.connect_provider(BASE_MAINNET_RPC).unwrap();
        
        // Verify we can query balance (proves provider works)
        let result = wallet.get_balance().await;
        assert!(result.is_ok());
    }
}
