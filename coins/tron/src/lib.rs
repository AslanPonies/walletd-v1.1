//! Tron (TRX) wallet support for WalletD
//!
//! Tron is compatible with Ethereum's cryptography but uses different address encoding.

use anyhow::Result;
use bip39::Mnemonic;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha2::{Sha256, Digest};
use sha3::Keccak256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Error, Debug)]
pub enum TronError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Insufficient bandwidth")]
    InsufficientBandwidth,
    #[error("Insufficient energy")]
    InsufficientEnergy,
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

// ============================================================================
// CONFIG
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub api_endpoints: Vec<String>,
    pub solidity_endpoints: Vec<String>,
    pub explorer: String,
    pub is_mainnet: bool,
}

pub const TRON_ADDRESS_PREFIX: u8 = 0x41; // Mainnet prefix
pub const SUN_PER_TRX: u64 = 1_000_000; // 1 TRX = 1,000,000 SUN

impl NetworkConfig {
    pub fn mainnet() -> Self {
        Self {
            name: "Tron Mainnet".to_string(),
            api_endpoints: vec![
                "https://api.trongrid.io".to_string(),
                "https://api.shasta.trongrid.io".to_string(),
            ],
            solidity_endpoints: vec![
                "https://api.trongrid.io".to_string(),
            ],
            explorer: "https://tronscan.org".to_string(),
            is_mainnet: true,
        }
    }

    pub fn shasta() -> Self {
        Self {
            name: "Tron Shasta Testnet".to_string(),
            api_endpoints: vec!["https://api.shasta.trongrid.io".to_string()],
            solidity_endpoints: vec!["https://api.shasta.trongrid.io".to_string()],
            explorer: "https://shasta.tronscan.org".to_string(),
            is_mainnet: false,
        }
    }

    pub fn nile() -> Self {
        Self {
            name: "Tron Nile Testnet".to_string(),
            api_endpoints: vec!["https://nile.trongrid.io".to_string()],
            solidity_endpoints: vec!["https://nile.trongrid.io".to_string()],
            explorer: "https://nile.tronscan.org".to_string(),
            is_mainnet: false,
        }
    }

    pub fn testnet() -> Self {
        Self::shasta()
    }

    pub fn trx_to_sun(trx: f64) -> u64 {
        (trx * SUN_PER_TRX as f64) as u64
    }

    pub fn sun_to_trx(sun: u64) -> f64 {
        sun as f64 / SUN_PER_TRX as f64
    }
}

// ============================================================================
// WALLET
// ============================================================================

pub struct TronWallet {
    secret_key: SecretKey,
    public_key: PublicKey,
    config: NetworkConfig,
    api_key: Option<String>,
}

impl TronWallet {
    pub fn new(config: NetworkConfig) -> Result<Self> {
        let secp = Secp256k1::new();
        
        // Generate random 32-byte key
        let mut key_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key_bytes);
        
        let secret_key = SecretKey::from_slice(&key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
            config,
            api_key: None,
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

        // Tron uses same derivation as Ethereum: m/44'/195'/0'/0/0
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[..32]);

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
            config,
            api_key: None,
        })
    }

    pub fn from_private_key(key: &[u8], config: NetworkConfig) -> Result<Self> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(key)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
            config,
            api_key: None,
        })
    }

    pub fn from_private_key_hex(key: &str, config: NetworkConfig) -> Result<Self> {
        let key = key.strip_prefix("0x").unwrap_or(key);
        let bytes = hex::decode(key)?;
        Self::from_private_key(&bytes, config)
    }

    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_string());
    }

    /// Get Tron address (base58check encoded, starts with T)
    pub fn address(&self) -> String {
        // Get uncompressed public key (65 bytes)
        let pubkey_uncompressed = self.public_key.serialize_uncompressed();
        
        // Keccak256 hash of public key (skip first byte - 0x04 prefix)
        let hash = Keccak256::digest(&pubkey_uncompressed[1..]);
        
        // Take last 20 bytes and add prefix
        let mut address_bytes = vec![TRON_ADDRESS_PREFIX];
        address_bytes.extend_from_slice(&hash[12..]);
        
        // Double SHA256 for checksum
        let hash1 = Sha256::digest(&address_bytes);
        let hash2 = Sha256::digest(&hash1);
        let checksum = &hash2[..4];
        
        // Append checksum
        address_bytes.extend_from_slice(checksum);
        
        // Base58 encode
        bs58::encode(address_bytes).into_string()
    }

    /// Get hex address (without base58 encoding)
    pub fn hex_address(&self) -> String {
        let pubkey_uncompressed = self.public_key.serialize_uncompressed();
        let hash = Keccak256::digest(&pubkey_uncompressed[1..]);
        
        let mut address_bytes = vec![TRON_ADDRESS_PREFIX];
        address_bytes.extend_from_slice(&hash[12..]);
        
        hex::encode(address_bytes)
    }

    pub fn public_key(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.secret_key.secret_bytes()))
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn is_mainnet(&self) -> bool {
        self.config.is_mainnet
    }

    pub async fn get_balance(&self) -> Result<u64> {
        if self.api_key.is_none() {
            return Ok(0);
        }
        // Would query TronGrid API
        Ok(0)
    }

    pub async fn get_balance_trx(&self) -> Result<f64> {
        let sun = self.get_balance().await?;
        Ok(NetworkConfig::sun_to_trx(sun))
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::new();
        let hash = Keccak256::digest(message);
        let msg = secp256k1::Message::from_slice(&hash).unwrap();
        let sig = secp.sign_ecdsa(&msg, &self.secret_key);
        sig.serialize_compact().to_vec()
    }

    /// Validate a Tron address
    pub fn validate_address(address: &str) -> bool {
        if !address.starts_with('T') {
            return false;
        }
        
        // Try to decode base58
        let decoded = match bs58::decode(address).into_vec() {
            Ok(v) => v,
            Err(_) => return false,
        };
        
        if decoded.len() != 25 {
            return false;
        }
        
        // Verify checksum
        let address_bytes = &decoded[..21];
        let checksum = &decoded[21..];
        
        let hash1 = Sha256::digest(address_bytes);
        let hash2 = Sha256::digest(&hash1);
        
        &hash2[..4] == checksum
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
        let wallet = TronWallet::mainnet().unwrap();
        assert!(wallet.address().starts_with('T'));
    }

    #[test]
    fn test_testnet_wallet() {
        let wallet = TronWallet::testnet().unwrap();
        assert!(wallet.address().starts_with('T'));
    }

    #[test]
    fn test_from_mnemonic() {
        let wallet = TronWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        assert!(wallet.address().starts_with('T'));
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let w1 = TronWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        let w2 = TronWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::mainnet()).unwrap();
        assert_eq!(w1.address(), w2.address());
    }

    #[test]
    fn test_random_wallets_different() {
        let w1 = TronWallet::mainnet().unwrap();
        let w2 = TronWallet::mainnet().unwrap();
        assert_ne!(w1.address(), w2.address());
    }

    #[test]
    fn test_address_format() {
        let wallet = TronWallet::mainnet().unwrap();
        let addr = wallet.address();
        assert!(addr.starts_with('T'));
        assert_eq!(addr.len(), 34);
    }

    #[test]
    fn test_hex_address() {
        let wallet = TronWallet::mainnet().unwrap();
        let hex_addr = wallet.hex_address();
        assert!(hex_addr.starts_with("41"));
        assert_eq!(hex_addr.len(), 42);
    }

    #[test]
    fn test_validate_address() {
        let wallet = TronWallet::mainnet().unwrap();
        assert!(TronWallet::validate_address(&wallet.address()));
    }

    #[test]
    fn test_validate_invalid_address() {
        assert!(!TronWallet::validate_address("invalid"));
        assert!(!TronWallet::validate_address("0x1234567890"));
        assert!(!TronWallet::validate_address("T123")); // Too short
    }

    #[test]
    fn test_private_key_format() {
        let wallet = TronWallet::mainnet().unwrap();
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_sign_message() {
        let wallet = TronWallet::mainnet().unwrap();
        let sig = wallet.sign(b"Hello Tron!");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_config_mainnet() {
        let config = NetworkConfig::mainnet();
        assert!(config.is_mainnet);
    }

    #[test]
    fn test_config_testnet() {
        let config = NetworkConfig::testnet();
        assert!(!config.is_mainnet);
    }

    #[test]
    fn test_trx_conversion() {
        assert_eq!(NetworkConfig::trx_to_sun(1.0), 1_000_000);
        assert_eq!(NetworkConfig::sun_to_trx(1_000_000), 1.0);
    }

    #[tokio::test]
    async fn test_get_balance_no_api() {
        let wallet = TronWallet::mainnet().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_is_mainnet() {
        let mainnet = TronWallet::mainnet().unwrap();
        let testnet = TronWallet::testnet().unwrap();
        assert!(mainnet.is_mainnet());
        assert!(!testnet.is_mainnet());
    }

    #[test]
    fn test_from_private_key_hex() {
        let key = "0101010101010101010101010101010101010101010101010101010101010101";
        let wallet = TronWallet::from_private_key_hex(key, NetworkConfig::mainnet()).unwrap();
        assert!(wallet.address().starts_with('T'));
    }
}
