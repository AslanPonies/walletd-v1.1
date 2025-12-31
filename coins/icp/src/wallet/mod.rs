use anyhow::Result;
use candid::Principal;
use ic_agent::{Agent, Identity};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod hd_wallet;
pub mod security;
pub mod transaction;

pub use error::IcpWalletError;
pub use hd_wallet::HDWallet;
pub use security::SecureKeyStore;
pub use transaction::{Transaction, TransactionBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcpWallet {
    pub principal: Principal,
    pub account_id: String,
    pub public_key: Vec<u8>,
    #[serde(skip_serializing)]
    ____private_key: Option<Vec<u8>>,
}

impl IcpWallet {
    pub fn new(identity: Box<dyn Identity>) -> Result<Self> {
        let principal = identity.sender().map_err(|e| anyhow::anyhow!(e))?;
        let account_id = Self::principal_to_account_id(&principal);

        Ok(Self {
            principal,
            account_id,
            public_key: vec![],
            ____private_key: None,
        })
    }

    pub fn from_principal(principal: Principal, _network: crate::HDNetworkType) -> Self {
        Self {
            principal,
            account_id: Self::principal_to_account_id(&principal),
            public_key: vec![],
            ____private_key: None,
        }
    }

    pub fn principal(&self) -> Principal {
        self.principal
    }

    pub fn address(&self) -> &str {
        &self.account_id
    }

    pub fn principal_to_account_id(principal: &Principal) -> String {
        use sha2::{Digest, Sha224};
        let mut hasher = Sha224::new();
        hasher.update(b"\x0Aaccount-id");
        hasher.update(principal.as_slice());
        hasher.update([0u8; 32]);
        hex::encode(hasher.finalize())
    }

    pub async fn get_balance(&self, _agent: &Agent) -> Result<u64> {
        // Simplified implementation
        Ok(1_000_000_000) // 10 ICP
    }

    pub fn create_transaction(
        &self,
        to: Principal,
        amount: u64,
        memo: Option<u64>,
    ) -> Result<Transaction> {
        Ok(Transaction {
            from: self.principal,
            to,
            amount,
            fee: Some(10_000),
            memo,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    pub async fn transfer(
        &self,
        _agent: &Agent,
        _to: Principal,
        _amount: u64,
        _memo: Option<u64>,
    ) -> Result<u64> {
        // Simplified implementation
        Ok(12345) // Mock block height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Principal Tests
    // ============================================================================

    #[test]
    fn test_principal_to_account_id() {
        // Anonymous principal
        let anon = Principal::anonymous();
        let account_id = IcpWallet::principal_to_account_id(&anon);
        
        // Account ID should be a hex string
        assert!(!account_id.is_empty());
        // SHA224 produces 28 bytes = 56 hex chars
        assert_eq!(account_id.len(), 56);
    }

    #[test]
    fn test_principal_to_account_id_deterministic() {
        let anon = Principal::anonymous();
        let account_id1 = IcpWallet::principal_to_account_id(&anon);
        let account_id2 = IcpWallet::principal_to_account_id(&anon);
        
        assert_eq!(account_id1, account_id2);
    }

    #[test]
    fn test_principal_to_account_id_different_principals() {
        let anon = Principal::anonymous();
        let mgmt = Principal::management_canister();
        
        let account_id1 = IcpWallet::principal_to_account_id(&anon);
        let account_id2 = IcpWallet::principal_to_account_id(&mgmt);
        
        assert_ne!(account_id1, account_id2);
    }

    // ============================================================================
    // Wallet Creation Tests
    // ============================================================================

    #[test]
    fn test_wallet_from_principal() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        assert_eq!(wallet.principal(), principal);
    }

    #[test]
    fn test_wallet_address() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let address = wallet.address();
        assert!(!address.is_empty());
        assert_eq!(address.len(), 56); // SHA224 hex
    }

    #[test]
    fn test_wallet_principal_accessor() {
        let principal = Principal::management_canister();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::TestNet);
        
        assert_eq!(wallet.principal(), principal);
    }

    #[test]
    fn test_wallet_mainnet_testnet_same_address() {
        let principal = Principal::anonymous();
        
        let mainnet_wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        let testnet_wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::TestNet);
        
        // Same principal should have same address regardless of network
        assert_eq!(mainnet_wallet.address(), testnet_wallet.address());
    }

    // ============================================================================
    // Transaction Tests
    // ============================================================================

    #[test]
    fn test_create_transaction() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let to = Principal::management_canister();
        let amount = 1_000_000u64; // 0.01 ICP
        
        let tx = wallet.create_transaction(to, amount, None).unwrap();
        
        assert_eq!(tx.from, principal);
        assert_eq!(tx.to, to);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.fee, Some(10_000));
        assert!(tx.memo.is_none());
    }

    #[test]
    fn test_create_transaction_with_memo() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let to = Principal::management_canister();
        let amount = 1_000_000u64;
        let memo = 12345u64;
        
        let tx = wallet.create_transaction(to, amount, Some(memo)).unwrap();
        
        assert_eq!(tx.memo, Some(memo));
    }

    #[test]
    fn test_transaction_timestamp() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let tx = wallet.create_transaction(
            Principal::management_canister(),
            1_000_000,
            None
        ).unwrap();
        
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        assert!(tx.created_at >= before);
        assert!(tx.created_at <= after);
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_wallet_serialization() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let json = serde_json::to_string(&wallet).unwrap();
        let deserialized: IcpWallet = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.principal(), wallet.principal());
        assert_eq!(deserialized.address(), wallet.address());
    }

    #[test]
    fn test_wallet_private_key_not_serialized() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let json = serde_json::to_string(&wallet).unwrap();
        
        // Private key field should not appear in serialized output
        assert!(!json.contains("private_key"));
    }

    // ============================================================================
    // Balance Tests (Mocked)
    // ============================================================================

    #[tokio::test]
    async fn test_get_balance_mocked() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        // Note: This uses the mock implementation
        let agent = Agent::builder()
            .with_url("http://localhost:8000")
            .build()
            .unwrap();
        
        let balance = wallet.get_balance(&agent).await.unwrap();
        assert_eq!(balance, 1_000_000_000); // Mock returns 10 ICP
    }

    // ============================================================================
    // Transfer Tests (Mocked)
    // ============================================================================

    #[tokio::test]
    async fn test_transfer_mocked() {
        let principal = Principal::anonymous();
        let wallet = IcpWallet::from_principal(principal, crate::HDNetworkType::MainNet);
        
        let agent = Agent::builder()
            .with_url("http://localhost:8000")
            .build()
            .unwrap();
        
        let block_height = wallet.transfer(
            &agent,
            Principal::management_canister(),
            1_000_000,
            None
        ).await.unwrap();
        
        // Mock returns 12345
        assert_eq!(block_height, 12345);
    }
}
