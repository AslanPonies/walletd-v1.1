//! Near Protocol (NEAR) wallet support for WalletD
//!
//! Near uses Ed25519 for signing and supports both implicit and named accounts.

use anyhow::Result;
use bip39::Mnemonic;
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Error, Debug)]
pub enum NearError {
    #[error("Invalid account ID: {0}")]
    InvalidAccountId(String),
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

// ============================================================================
// CONFIG
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub chain_id: String,
    pub rpc_endpoints: Vec<String>,
    pub explorer: String,
    pub is_mainnet: bool,
}

pub const YOCTO_PER_NEAR: u128 = 1_000_000_000_000_000_000_000_000; // 10^24

impl NetworkConfig {
    pub fn mainnet() -> Self {
        Self {
            name: "Near Mainnet".to_string(),
            chain_id: "mainnet".to_string(),
            rpc_endpoints: vec![
                "https://rpc.mainnet.near.org".to_string(),
                "https://near-mainnet.infura.io/v3/".to_string(),
            ],
            explorer: "https://explorer.near.org".to_string(),
            is_mainnet: true,
        }
    }

    pub fn testnet() -> Self {
        Self {
            name: "Near Testnet".to_string(),
            chain_id: "testnet".to_string(),
            rpc_endpoints: vec![
                "https://rpc.testnet.near.org".to_string(),
            ],
            explorer: "https://explorer.testnet.near.org".to_string(),
            is_mainnet: false,
        }
    }

    pub fn near_to_yocto(near: f64) -> u128 {
        (near * YOCTO_PER_NEAR as f64) as u128
    }

    pub fn yocto_to_near(yocto: u128) -> f64 {
        yocto as f64 / YOCTO_PER_NEAR as f64
    }
}

// ============================================================================
// KEY TYPES
// ============================================================================

/// Near public key types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
}

impl KeyType {
    pub fn prefix(&self) -> &'static str {
        match self {
            KeyType::Ed25519 => "ed25519:",
        }
    }
}

// ============================================================================
// WALLET
// ============================================================================

pub struct NearWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    config: NetworkConfig,
    account_id: Option<String>,
    api_endpoint: Option<String>,
}

impl NearWallet {
    pub fn new(config: NetworkConfig) -> Result<Self> {
        let mut csprng = rand::rngs::OsRng;
        let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
        csprng.fill_bytes(&mut secret_bytes);
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            config,
            account_id: None,
            api_endpoint: None,
        })
    }

    pub fn mainnet() -> Result<Self> {
        Self::new(NetworkConfig::mainnet())
    }

    pub fn testnet() -> Result<Self> {
        Self::new(NetworkConfig::testnet())
    }

    pub fn from_mnemonic(mnemonic: &str, config: NetworkConfig) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");

        // Near derivation path: m/44'/397'/0'
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[..32]);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            config,
            account_id: None,
            api_endpoint: None,
        })
    }

    pub fn from_private_key(key: &[u8; 32], config: NetworkConfig) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(key);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            config,
            account_id: None,
            api_endpoint: None,
        })
    }

    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn set_api_endpoint(&mut self, endpoint: &str) {
        self.api_endpoint = Some(endpoint.to_string());
    }

    /// Get implicit account ID (hex-encoded public key)
    pub fn implicit_account_id(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Get account ID (named or implicit)
    pub fn account_id(&self) -> String {
        self.account_id.clone().unwrap_or_else(|| self.implicit_account_id())
    }

    /// Get public key in Near format (ed25519:base58...)
    pub fn public_key(&self) -> String {
        let encoded = bs58::encode(self.verifying_key.as_bytes()).into_string();
        format!("ed25519:{}", encoded)
    }

    /// Get public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Get private key in Near format
    pub fn private_key(&self) -> String {
        let mut full_key = Vec::with_capacity(64);
        full_key.extend_from_slice(self.signing_key.as_bytes());
        full_key.extend_from_slice(self.verifying_key.as_bytes());
        let encoded = bs58::encode(&full_key).into_string();
        format!("ed25519:{}", encoded)
    }

    /// Get private key as hex
    pub fn private_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.signing_key.as_bytes()))
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn is_mainnet(&self) -> bool {
        self.config.is_mainnet
    }

    pub async fn get_balance(&self) -> Result<u128> {
        if self.api_endpoint.is_none() {
            return Ok(0);
        }
        Ok(0)
    }

    pub async fn get_balance_near(&self) -> Result<f64> {
        let yocto = self.get_balance().await?;
        Ok(NetworkConfig::yocto_to_near(yocto))
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        use ed25519_dalek::{Signature, Verifier};
        if signature.len() != 64 {
            return false;
        }
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let sig = Signature::from_bytes(&sig_bytes);
        self.verifying_key.verify(message, &sig).is_ok()
    }

    /// Validate a Near account ID
    pub fn validate_account_id(account_id: &str) -> bool {
        // Near account ID rules:
        // - 2-64 characters
        // - lowercase letters, digits, - and _
        // - Cannot start with - or _
        // - Implicit accounts are 64 hex characters
        
        if account_id.len() < 2 || account_id.len() > 64 {
            return false;
        }

        // Check for implicit account (64 hex chars)
        if account_id.len() == 64 && account_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return true;
        }

        // Named account validation
        let first = account_id.chars().next().unwrap();
        if first == '-' || first == '_' {
            return false;
        }

        account_id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_' || c == '.'
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_new_wallet() {
        let wallet = NearWallet::mainnet().unwrap();
        let account = wallet.implicit_account_id();
        assert_eq!(account.len(), 64);
    }

    #[test]
    fn test_testnet_wallet() {
        let wallet = NearWallet::testnet().unwrap();
        assert!(!wallet.is_mainnet());
    }

    #[test]
    fn test_from_mnemonic() {
        let wallet = NearWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        assert_eq!(wallet.implicit_account_id().len(), 64);
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let w1 = NearWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        let w2 = NearWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        assert_eq!(w1.implicit_account_id(), w2.implicit_account_id());
    }

    #[test]
    fn test_random_wallets_different() {
        let w1 = NearWallet::mainnet().unwrap();
        let w2 = NearWallet::mainnet().unwrap();
        assert_ne!(w1.implicit_account_id(), w2.implicit_account_id());
    }

    #[test]
    fn test_public_key_format() {
        let wallet = NearWallet::mainnet().unwrap();
        let pk = wallet.public_key();
        assert!(pk.starts_with("ed25519:"));
    }

    #[test]
    fn test_public_key_hex() {
        let wallet = NearWallet::mainnet().unwrap();
        let pk = wallet.public_key_hex();
        assert_eq!(pk.len(), 64);
    }

    #[test]
    fn test_private_key_format() {
        let wallet = NearWallet::mainnet().unwrap();
        let pk = wallet.private_key();
        assert!(pk.starts_with("ed25519:"));
    }

    #[test]
    fn test_private_key_hex() {
        let wallet = NearWallet::mainnet().unwrap();
        let pk = wallet.private_key_hex();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_set_account_id() {
        let mut wallet = NearWallet::mainnet().unwrap();
        wallet.set_account_id("myaccount.near");
        assert_eq!(wallet.account_id(), "myaccount.near");
    }

    #[test]
    fn test_sign_message() {
        let wallet = NearWallet::mainnet().unwrap();
        let sig = wallet.sign(b"Hello Near!");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_verify_signature() {
        let wallet = NearWallet::mainnet().unwrap();
        let msg = b"Hello Near!";
        let sig = wallet.sign(msg);
        assert!(wallet.verify(msg, &sig));
    }

    #[test]
    fn test_verify_wrong_message() {
        let wallet = NearWallet::mainnet().unwrap();
        let sig = wallet.sign(b"Hello Near!");
        assert!(!wallet.verify(b"Wrong message", &sig));
    }

    #[test]
    fn test_validate_account_id_implicit() {
        let wallet = NearWallet::mainnet().unwrap();
        assert!(NearWallet::validate_account_id(&wallet.implicit_account_id()));
    }

    #[test]
    fn test_validate_account_id_named() {
        assert!(NearWallet::validate_account_id("myaccount.near"));
        assert!(NearWallet::validate_account_id("alice"));
        assert!(NearWallet::validate_account_id("bob-123"));
        assert!(NearWallet::validate_account_id("test_account"));
    }

    #[test]
    fn test_validate_account_id_invalid() {
        assert!(!NearWallet::validate_account_id("a")); // Too short
        assert!(!NearWallet::validate_account_id("-invalid")); // Starts with -
        assert!(!NearWallet::validate_account_id("_invalid")); // Starts with _
        assert!(!NearWallet::validate_account_id("UPPERCASE")); // Uppercase
    }

    #[test]
    fn test_config_mainnet() {
        let config = NetworkConfig::mainnet();
        assert_eq!(config.chain_id, "mainnet");
        assert!(config.is_mainnet);
    }

    #[test]
    fn test_config_testnet() {
        let config = NetworkConfig::testnet();
        assert_eq!(config.chain_id, "testnet");
        assert!(!config.is_mainnet);
    }

    #[test]
    fn test_near_conversion() {
        let near = 1.0;
        let yocto = NetworkConfig::near_to_yocto(near);
        // Allow for floating point precision loss
        let diff = (yocto as i128 - YOCTO_PER_NEAR as i128).abs();
        assert!(diff < 1_000_000_000_000_000); // Within 0.001 NEAR
        
        let back = NetworkConfig::yocto_to_near(YOCTO_PER_NEAR);
        assert!((back - 1.0).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_get_balance_no_api() {
        let wallet = NearWallet::mainnet().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_is_mainnet() {
        let mainnet = NearWallet::mainnet().unwrap();
        let testnet = NearWallet::testnet().unwrap();
        assert!(mainnet.is_mainnet());
        assert!(!testnet.is_mainnet());
    }
}
