//! Polkadot (DOT) wallet support for WalletD
//!
//! Supports Polkadot, Kusama, and other Substrate-based chains.
//! Uses Ed25519 for signing (production would use Sr25519).

use anyhow::Result;
use bip39::Mnemonic;
use blake2::{Blake2b, Digest};
use blake2::digest::consts::U64;
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Error, Debug)]
pub enum PolkadotError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

// ============================================================================
// CONFIG
// ============================================================================

/// SS58 address prefixes for different networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SS58Prefix {
    Polkadot = 0,
    Kusama = 2,
    Westend = 42,  // Testnet
}

impl SS58Prefix {
    pub fn generic() -> u8 {
        42
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub ss58_prefix: u8,
    pub rpc_endpoints: Vec<String>,
    pub explorer: String,
    pub is_mainnet: bool,
}

pub const PLANCK_PER_DOT: u128 = 10_000_000_000; // 10^10

impl NetworkConfig {
    pub fn polkadot() -> Self {
        Self {
            name: "Polkadot".to_string(),
            token_symbol: "DOT".to_string(),
            decimals: 10,
            ss58_prefix: SS58Prefix::Polkadot as u8,
            rpc_endpoints: vec![
                "wss://rpc.polkadot.io".to_string(),
                "wss://polkadot.api.onfinality.io/public-ws".to_string(),
            ],
            explorer: "https://polkadot.subscan.io".to_string(),
            is_mainnet: true,
        }
    }

    pub fn kusama() -> Self {
        Self {
            name: "Kusama".to_string(),
            token_symbol: "KSM".to_string(),
            decimals: 12,
            ss58_prefix: SS58Prefix::Kusama as u8,
            rpc_endpoints: vec![
                "wss://kusama-rpc.polkadot.io".to_string(),
            ],
            explorer: "https://kusama.subscan.io".to_string(),
            is_mainnet: true,
        }
    }

    pub fn westend() -> Self {
        Self {
            name: "Westend Testnet".to_string(),
            token_symbol: "WND".to_string(),
            decimals: 12,
            ss58_prefix: SS58Prefix::Westend as u8,
            rpc_endpoints: vec![
                "wss://westend-rpc.polkadot.io".to_string(),
            ],
            explorer: "https://westend.subscan.io".to_string(),
            is_mainnet: false,
        }
    }

    pub fn testnet() -> Self {
        Self::westend()
    }

    pub fn dot_to_planck(dot: f64) -> u128 {
        (dot * PLANCK_PER_DOT as f64) as u128
    }

    pub fn planck_to_dot(planck: u128) -> f64 {
        planck as f64 / PLANCK_PER_DOT as f64
    }
}

// ============================================================================
// SS58 ENCODING
// ============================================================================

const SS58_PREFIX: &[u8] = b"SS58PRE";

fn ss58_checksum(data: &[u8]) -> [u8; 2] {
    let mut hasher = Blake2b::<U64>::new();
    hasher.update(SS58_PREFIX);
    hasher.update(data);
    let hash = hasher.finalize();
    [hash[0], hash[1]]
}

fn encode_ss58(prefix: u8, pubkey: &[u8; 32]) -> String {
    let mut data = Vec::with_capacity(35);
    data.push(prefix);
    data.extend_from_slice(pubkey);
    
    let checksum = ss58_checksum(&data);
    data.extend_from_slice(&checksum);
    
    bs58::encode(data).into_string()
}

fn decode_ss58(address: &str) -> Result<(u8, [u8; 32])> {
    let decoded = bs58::decode(address)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Invalid base58: {}", e))?;
    
    if decoded.len() != 35 {
        return Err(anyhow::anyhow!("Invalid address length"));
    }
    
    let prefix = decoded[0];
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&decoded[1..33]);
    
    // Verify checksum
    let checksum = ss58_checksum(&decoded[..33]);
    if checksum != [decoded[33], decoded[34]] {
        return Err(anyhow::anyhow!("Invalid checksum"));
    }
    
    Ok((prefix, pubkey))
}

// ============================================================================
// WALLET
// ============================================================================

pub struct PolkadotWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    config: NetworkConfig,
    api_endpoint: Option<String>,
}

impl PolkadotWallet {
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
            api_endpoint: None,
        })
    }

    pub fn polkadot() -> Result<Self> {
        Self::new(NetworkConfig::polkadot())
    }

    pub fn kusama() -> Result<Self> {
        Self::new(NetworkConfig::kusama())
    }

    pub fn testnet() -> Result<Self> {
        Self::new(NetworkConfig::testnet())
    }

    pub fn from_mnemonic(mnemonic: &str, config: NetworkConfig) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[..32]);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            config,
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
            api_endpoint: None,
        })
    }

    pub fn set_api_endpoint(&mut self, endpoint: &str) {
        self.api_endpoint = Some(endpoint.to_string());
    }

    /// Get SS58-encoded address
    pub fn address(&self) -> String {
        let pubkey_bytes: [u8; 32] = self.verifying_key.to_bytes();
        encode_ss58(self.config.ss58_prefix, &pubkey_bytes)
    }

    pub fn public_key(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signing_key.as_bytes()))
    }

    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    pub fn ss58_prefix(&self) -> u8 {
        self.config.ss58_prefix
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

    pub async fn get_balance_dot(&self) -> Result<f64> {
        let planck = self.get_balance().await?;
        Ok(NetworkConfig::planck_to_dot(planck))
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

    /// Validate an SS58 address
    pub fn validate_address(address: &str) -> bool {
        decode_ss58(address).is_ok()
    }

    /// Validate address for specific network
    pub fn validate_address_for_network(address: &str, prefix: u8) -> bool {
        match decode_ss58(address) {
            Ok((p, _)) => p == prefix,
            Err(_) => false,
        }
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
    fn test_new_wallet_polkadot() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        assert!(wallet.address().starts_with('1')); // Polkadot addresses start with 1
    }

    #[test]
    fn test_new_wallet_kusama() {
        let wallet = PolkadotWallet::kusama().unwrap();
        // Kusama addresses start with capital letters
        let addr = wallet.address();
        assert!(addr.chars().next().unwrap().is_alphabetic());
    }

    #[test]
    fn test_new_wallet_westend() {
        let wallet = PolkadotWallet::testnet().unwrap();
        let addr = wallet.address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_from_mnemonic() {
        let wallet = PolkadotWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::polkadot()).unwrap();
        assert!(wallet.address().starts_with('1'));
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let w1 = PolkadotWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::polkadot()).unwrap();
        let w2 = PolkadotWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::polkadot()).unwrap();
        assert_eq!(w1.address(), w2.address());
    }

    #[test]
    fn test_random_wallets_different() {
        let w1 = PolkadotWallet::polkadot().unwrap();
        let w2 = PolkadotWallet::polkadot().unwrap();
        assert_ne!(w1.address(), w2.address());
    }

    #[test]
    fn test_validate_address() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        assert!(PolkadotWallet::validate_address(&wallet.address()));
    }

    #[test]
    fn test_validate_invalid_address() {
        assert!(!PolkadotWallet::validate_address("invalid"));
        assert!(!PolkadotWallet::validate_address("0x1234"));
    }

    #[test]
    fn test_validate_address_for_network() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        assert!(PolkadotWallet::validate_address_for_network(&wallet.address(), 0));
        assert!(!PolkadotWallet::validate_address_for_network(&wallet.address(), 2));
    }

    #[test]
    fn test_public_key_format() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let pk = wallet.public_key();
        assert_eq!(pk.len(), 64);
    }

    #[test]
    fn test_private_key_format() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_sign_message() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let sig = wallet.sign(b"Hello Polkadot!");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_verify_signature() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let msg = b"Hello Polkadot!";
        let sig = wallet.sign(msg);
        assert!(wallet.verify(msg, &sig));
    }

    #[test]
    fn test_verify_wrong_message() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let sig = wallet.sign(b"Hello Polkadot!");
        assert!(!wallet.verify(b"Wrong message", &sig));
    }

    #[test]
    fn test_config_polkadot() {
        let config = NetworkConfig::polkadot();
        assert_eq!(config.token_symbol, "DOT");
        assert_eq!(config.decimals, 10);
        assert_eq!(config.ss58_prefix, 0);
    }

    #[test]
    fn test_config_kusama() {
        let config = NetworkConfig::kusama();
        assert_eq!(config.token_symbol, "KSM");
        assert_eq!(config.decimals, 12);
        assert_eq!(config.ss58_prefix, 2);
    }

    #[test]
    fn test_dot_conversion() {
        assert_eq!(NetworkConfig::dot_to_planck(1.0), 10_000_000_000);
        assert_eq!(NetworkConfig::planck_to_dot(10_000_000_000), 1.0);
    }

    #[tokio::test]
    async fn test_get_balance_no_api() {
        let wallet = PolkadotWallet::polkadot().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_ss58_prefix() {
        let polkadot = PolkadotWallet::polkadot().unwrap();
        let kusama = PolkadotWallet::kusama().unwrap();
        assert_eq!(polkadot.ss58_prefix(), 0);
        assert_eq!(kusama.ss58_prefix(), 2);
    }

    #[test]
    fn test_is_mainnet() {
        let mainnet = PolkadotWallet::polkadot().unwrap();
        let testnet = PolkadotWallet::testnet().unwrap();
        assert!(mainnet.is_mainnet());
        assert!(!testnet.is_mainnet());
    }
}
