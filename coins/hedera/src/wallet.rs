use crate::core::config::HederaConfig;
use crate::HederaClient;
use anyhow::Result;
use hedera::{Hbar, PrivateKey};

pub struct RealHederaWallet {
    pub network: String,
    pub account_id: Option<String>,
    pub public_key: String,
    pub private_key: String,
    pub client: Option<HederaClient>, // Make this public
}

impl RealHederaWallet {
    pub fn new(network: &str) -> Result<Self> {
        let private_key = PrivateKey::generate_ed25519();
        let public_key = private_key.public_key();

        Ok(Self {
            network: network.to_string(),
            account_id: None,
            public_key: public_key.to_string(),
            private_key: private_key.to_string(),
            client: None,
        })
    }

    // New method to initialize with existing credentials
    pub async fn init_with_existing_account(&mut self) -> Result<()> {
        // Try to load config and create client
        match HederaConfig::load() {
            Ok(config) => {
                match HederaClient::new(config) {
                    Ok(client) => {
                        self.client = Some(client);

                        // If we have an account ID, validate it exists
                        if let Some(account_id) = &self.account_id {
                            println!("✅ Initialized client for account: {account_id}");

                            // Try to get balance to verify account works
                            match self.get_balance().await {
                                Ok(balance) => {
                                    println!("💰 Account balance: {balance} HBAR");
                                }
                                Err(e) => {
                                    println!("⚠️  Could not verify balance: {e}");
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(anyhow::anyhow!("Failed to create client: {}", e)),
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to load config: {}", e)),
        }
    }

    pub async fn create_testnet_account(&mut self, initial_balance: Hbar) -> Result<String> {
        // Initialize client if not already done
        if self.client.is_none() {
            let config = HederaConfig::load()?;
            self.client = Some(HederaClient::new(config)?);
        }

        let client = self.client.as_ref().unwrap();

        let account_info = client.create_new_account(initial_balance).await?;
        self.account_id = Some(account_info.account_id.to_string());

        Ok(account_info.account_id.to_string())
    }

    pub async fn get_balance(&self) -> Result<f64> {
        // If we have a client and account ID, get real balance
        if let (Some(client), Some(account_id)) = (&self.client, &self.account_id) {
            // Use the client to get real balance
            let balance = client.get_account_balance(account_id).await?;
            Ok(balance)
        } else {
            Ok(0.0)
        }
    }

    pub async fn send_hbar(&self, to_account: &str, amount: f64) -> Result<String> {
        if let (Some(client), Some(from_account)) = (&self.client, &self.account_id) {
            // Use the client to send real transaction
            let tx_id = client
                .transfer_hbar(from_account, to_account, amount)
                .await?;
            Ok(tx_id)
        } else {
            Err(anyhow::anyhow!("Wallet not properly initialized"))
        }
    }
}

// Convenience method with default balance

// Add method to ensure client is initialized

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Wallet Creation Tests
    // ============================================================================

    #[test]
    fn test_new_wallet_mainnet() {
        let wallet = RealHederaWallet::new("mainnet");
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert_eq!(wallet.network, "mainnet");
    }

    #[test]
    fn test_new_wallet_testnet() {
        let wallet = RealHederaWallet::new("testnet");
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert_eq!(wallet.network, "testnet");
    }

    #[test]
    fn test_new_wallet_previewnet() {
        let wallet = RealHederaWallet::new("previewnet");
        assert!(wallet.is_ok());
        
        let wallet = wallet.unwrap();
        assert_eq!(wallet.network, "previewnet");
    }

    #[test]
    fn test_new_wallet_has_keys() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Should have generated keys
        assert!(!wallet.public_key.is_empty());
        assert!(!wallet.private_key.is_empty());
    }

    #[test]
    fn test_new_wallet_no_account_id() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // New wallet doesn't have an account ID yet
        assert!(wallet.account_id.is_none());
    }

    #[test]
    fn test_new_wallet_no_client() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // New wallet doesn't have a client yet
        assert!(wallet.client.is_none());
    }

    // ============================================================================
    // Key Generation Tests
    // ============================================================================

    #[test]
    fn test_unique_keys_per_wallet() {
        let wallet1 = RealHederaWallet::new("testnet").unwrap();
        let wallet2 = RealHederaWallet::new("testnet").unwrap();
        
        // Each wallet should have unique keys
        assert_ne!(wallet1.public_key, wallet2.public_key);
        assert_ne!(wallet1.private_key, wallet2.private_key);
    }

    #[test]
    fn test_key_format() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Ed25519 public keys in Hedera format
        // Should be hex encoded
        assert!(wallet.public_key.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(wallet.private_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_public_key_length() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Hedera SDK returns keys in DER format, not raw hex
        // The length varies based on the format used
        assert!(!wallet.public_key.is_empty());
        // Typically 88 chars for DER encoded or 64 for raw
        assert!(wallet.public_key.len() >= 64);
    }

    #[test]
    fn test_private_key_length() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Hedera SDK returns keys in DER format, not raw hex
        // The length varies based on the format used
        assert!(!wallet.private_key.is_empty());
        // Typically 96 chars for DER encoded or 64 for raw
        assert!(wallet.private_key.len() >= 64);
    }

    #[test]
    fn test_multiple_wallet_generation() {
        // Generate 10 wallets and ensure all unique
        let wallets: Vec<RealHederaWallet> = (0..10)
            .map(|_| RealHederaWallet::new("testnet").unwrap())
            .collect();
        
        for i in 0..wallets.len() {
            for j in (i+1)..wallets.len() {
                assert_ne!(wallets[i].public_key, wallets[j].public_key);
                assert_ne!(wallets[i].private_key, wallets[j].private_key);
            }
        }
    }

    // ============================================================================
    // Balance Tests (without network)
    // ============================================================================

    #[tokio::test]
    async fn test_get_balance_no_client() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Without client, balance should be 0
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0.0);
    }

    #[tokio::test]
    async fn test_get_balance_no_account_id() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = None;
        
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0.0);
    }

    #[tokio::test]
    async fn test_get_balance_with_account_id_no_client() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        wallet.client = None;
        
        // Still should return 0 because no client
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0.0);
    }

    // ============================================================================
    // Send Tests (without network)
    // ============================================================================

    #[tokio::test]
    async fn test_send_hbar_no_client() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Without client, should fail
        let result = wallet.send_hbar("0.0.12345", 1.0).await;
        assert!(result.is_err());
        
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not properly initialized"));
    }

    #[tokio::test]
    async fn test_send_hbar_no_account_id() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = None;
        
        let result = wallet.send_hbar("0.0.12345", 1.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_hbar_zero_amount() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        
        // Should fail because no client, not because of zero amount
        let result = wallet.send_hbar("0.0.54321", 0.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_hbar_negative_amount() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        
        // Should fail because no client
        let result = wallet.send_hbar("0.0.54321", -1.0).await;
        assert!(result.is_err());
    }

    // ============================================================================
    // Network Configuration Tests
    // ============================================================================

    #[test]
    fn test_network_preserved() {
        let mainnet = RealHederaWallet::new("mainnet").unwrap();
        let testnet = RealHederaWallet::new("testnet").unwrap();
        let previewnet = RealHederaWallet::new("previewnet").unwrap();
        
        assert_eq!(mainnet.network, "mainnet");
        assert_eq!(testnet.network, "testnet");
        assert_eq!(previewnet.network, "previewnet");
    }

    #[test]
    fn test_custom_network_name() {
        let wallet = RealHederaWallet::new("custom-network").unwrap();
        assert_eq!(wallet.network, "custom-network");
    }

    #[test]
    fn test_empty_network_name() {
        let wallet = RealHederaWallet::new("").unwrap();
        assert_eq!(wallet.network, "");
    }

    // ============================================================================
    // Account ID Tests
    // ============================================================================

    #[test]
    fn test_account_id_can_be_set() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        
        assert_eq!(wallet.account_id, Some("0.0.12345".to_string()));
    }

    #[test]
    fn test_account_id_format() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Valid Hedera account ID format
        wallet.account_id = Some("0.0.12345".to_string());
        let id = wallet.account_id.unwrap();
        
        // Should be shard.realm.account format
        let parts: Vec<&str> = id.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_account_id_zero() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.0".to_string());
        
        assert!(wallet.account_id.is_some());
    }

    #[test]
    fn test_account_id_large_number() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.999999999".to_string());
        
        assert_eq!(wallet.account_id, Some("0.0.999999999".to_string()));
    }

    // ============================================================================
    // Wallet State Tests
    // ============================================================================

    #[test]
    fn test_wallet_initial_state() {
        let wallet = RealHederaWallet::new("testnet").unwrap();
        
        assert_eq!(wallet.network, "testnet");
        assert!(wallet.account_id.is_none());
        assert!(wallet.client.is_none());
        assert!(!wallet.public_key.is_empty());
        assert!(!wallet.private_key.is_empty());
    }

    #[test]
    fn test_wallet_mutable() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        
        // Can modify fields
        wallet.account_id = Some("0.0.12345".to_string());
        assert!(wallet.account_id.is_some());
        
        wallet.account_id = None;
        assert!(wallet.account_id.is_none());
    }
}

// ============================================================================
// Integration Tests (require network access and config)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires network access and Hedera config"]
    async fn test_init_with_existing_account() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        
        // This will fail without proper config, but shouldn't panic
        let result = wallet.init_with_existing_account().await;
        println!("Init result: {:?}", result);
    }

    #[tokio::test]
    #[ignore = "Requires network access and funded account"]
    async fn test_create_testnet_account() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        
        let result = wallet.create_testnet_account(Hbar::new(10)).await;
        println!("Create account result: {:?}", result);
    }

    #[tokio::test]
    #[ignore = "Requires network access and funded account"]
    async fn test_real_balance_query() {
        let mut wallet = RealHederaWallet::new("testnet").unwrap();
        wallet.account_id = Some("0.0.12345".to_string());
        
        let _ = wallet.init_with_existing_account().await;
        let balance = wallet.get_balance().await;
        println!("Balance result: {:?}", balance);
    }
}
