//! # WalletD Aptos
//!
//! Aptos blockchain support for the WalletD SDK.
//!
//! ## Features
//!
//! - Ed25519 key generation and management
//! - Aptos address derivation (0x prefixed, 64 hex chars)
//! - Transaction signing
//! - BIP-44 HD derivation (m/44'/637'/0'/0'/0')
//!
//! ## Example
//!
//! ```rust
//! use walletd_aptos::{AptosWallet, AptosNetwork};
//!
//! // Create from mnemonic
//! let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//! let wallet = AptosWallet::from_mnemonic(mnemonic, AptosNetwork::Mainnet)?;
//!
//! println!("Address: {}", wallet.address());
//! # Ok::<(), walletd_aptos::AptosError>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Digest};
use std::fmt;
use thiserror::Error;

// Re-export traits
pub use walletd_traits::WalletError;

/// Aptos-specific errors
#[derive(Error, Debug)]
pub enum AptosError {
    /// Invalid mnemonic phrase
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    /// Key derivation failed
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Signing error
    #[error("Signing failed: {0}")]
    SigningError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),
}

impl From<AptosError> for WalletError {
    fn from(e: AptosError) -> Self {
        WalletError::Other(e.to_string())
    }
}

/// Aptos network configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum AptosNetwork {
    /// Aptos Mainnet
    #[default]
    Mainnet,
    /// Aptos Testnet
    Testnet,
    /// Aptos Devnet
    Devnet,
    /// Local network
    Localnet,
}

impl AptosNetwork {
    /// Returns the REST API URL for this network
    pub fn rest_url(&self) -> &'static str {
        match self {
            AptosNetwork::Mainnet => "https://fullnode.mainnet.aptoslabs.com/v1",
            AptosNetwork::Testnet => "https://fullnode.testnet.aptoslabs.com/v1",
            AptosNetwork::Devnet => "https://fullnode.devnet.aptoslabs.com/v1",
            AptosNetwork::Localnet => "http://127.0.0.1:8080/v1",
        }
    }

    /// Returns the faucet URL (testnet/devnet only)
    pub fn faucet_url(&self) -> Option<&'static str> {
        match self {
            AptosNetwork::Testnet => Some("https://faucet.testnet.aptoslabs.com"),
            AptosNetwork::Devnet => Some("https://faucet.devnet.aptoslabs.com"),
            _ => None,
        }
    }

    /// Returns the indexer URL
    pub fn indexer_url(&self) -> &'static str {
        match self {
            AptosNetwork::Mainnet => "https://indexer.mainnet.aptoslabs.com/v1/graphql",
            AptosNetwork::Testnet => "https://indexer.testnet.aptoslabs.com/v1/graphql",
            AptosNetwork::Devnet => "https://indexer.devnet.aptoslabs.com/v1/graphql",
            AptosNetwork::Localnet => "http://127.0.0.1:8090/v1/graphql",
        }
    }

    /// Returns the chain ID
    pub fn chain_id(&self) -> u8 {
        match self {
            AptosNetwork::Mainnet => 1,
            AptosNetwork::Testnet => 2,
            AptosNetwork::Devnet => 3,
            AptosNetwork::Localnet => 4,
        }
    }

    /// Returns the explorer URL
    pub fn explorer_url(&self) -> &'static str {
        match self {
            AptosNetwork::Mainnet => "https://explorer.aptoslabs.com",
            AptosNetwork::Testnet => "https://explorer.aptoslabs.com/?network=testnet",
            AptosNetwork::Devnet => "https://explorer.aptoslabs.com/?network=devnet",
            AptosNetwork::Localnet => "https://explorer.aptoslabs.com/?network=local",
        }
    }
}


impl fmt::Display for AptosNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AptosNetwork::Mainnet => write!(f, "mainnet"),
            AptosNetwork::Testnet => write!(f, "testnet"),
            AptosNetwork::Devnet => write!(f, "devnet"),
            AptosNetwork::Localnet => write!(f, "localnet"),
        }
    }
}

/// Aptos amount (in Octas, 1 APT = 10^8 Octas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AptosAmount(u64);

impl AptosAmount {
    /// Number of Octas per APT
    pub const OCTAS_PER_APT: u64 = 100_000_000;

    /// Creates a new amount from Octas
    pub fn from_octas(octas: u64) -> Self {
        Self(octas)
    }

    /// Creates a new amount from APT
    pub fn from_apt(apt: f64) -> Self {
        Self((apt * Self::OCTAS_PER_APT as f64) as u64)
    }

    /// Returns the amount in Octas
    pub fn octas(&self) -> u64 {
        self.0
    }

    /// Returns the amount in APT
    pub fn apt(&self) -> f64 {
        self.0 as f64 / Self::OCTAS_PER_APT as f64
    }

    /// Zero amount
    pub fn zero() -> Self {
        Self(0)
    }

    /// Checked addition
    pub fn checked_add(&self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Checked subtraction
    pub fn checked_sub(&self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Decimals for APT
    pub fn decimals() -> u8 {
        8
    }

    /// Symbol
    pub fn symbol() -> &'static str {
        "APT"
    }
}

impl Default for AptosAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for AptosAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.8} APT", self.apt())
    }
}

impl std::ops::Add for AptosAmount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for AptosAmount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// Aptos address (32 bytes, displayed as 0x + 64 hex chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AptosAddress([u8; 32]);

impl AptosAddress {
    /// Creates an address from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the address bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates an address from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, AptosError> {
        let hex_str = hex_str.trim_start_matches("0x");
        
        // Aptos addresses can be shortened (leading zeros omitted)
        // Pad to 64 characters if needed
        let padded = format!("{:0>64}", hex_str);
        
        if padded.len() != 64 {
            return Err(AptosError::InvalidAddress(
                "Address must be at most 64 hex characters".to_string(),
            ));
        }
        
        let bytes = hex::decode(&padded)
            .map_err(|e| AptosError::InvalidAddress(e.to_string()))?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the address as hex string with 0x prefix
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    /// Returns the short form address (without leading zeros)
    pub fn to_short_hex(&self) -> String {
        let hex = hex::encode(self.0);
        let trimmed = hex.trim_start_matches('0');
        if trimmed.is_empty() {
            "0x0".to_string()
        } else {
            format!("0x{}", trimmed)
        }
    }

    /// Derives address from Ed25519 public key
    /// Aptos uses: SHA3-256(pubkey || 0x00)[0..32]
    pub fn from_ed25519_pubkey(pubkey: &VerifyingKey) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(pubkey.as_bytes());
        hasher.update([0x00]); // Single-key Ed25519 scheme identifier
        let hash = hasher.finalize();
        
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&hash[..32]);
        Self(addr)
    }

    /// Check if this is the zero address
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl fmt::Display for AptosAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl std::str::FromStr for AptosAddress {
    type Err = AptosError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/// Aptos wallet
pub struct AptosWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    address: AptosAddress,
    network: AptosNetwork,
}

impl AptosWallet {
    /// BIP-44 coin type for Aptos
    pub const COIN_TYPE: u32 = 637;

    /// Creates a new random wallet
    pub fn new(network: AptosNetwork) -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let address = AptosAddress::from_ed25519_pubkey(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            address,
            network,
        }
    }

    /// Creates a wallet from a mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, network: AptosNetwork) -> Result<Self, AptosError> {
        Self::from_mnemonic_with_path(mnemonic, network, 0, 0)
    }

    /// Creates a wallet from a mnemonic with custom derivation path
    /// Path: m/44'/637'/account'/0'/address_index'
    pub fn from_mnemonic_with_path(
        mnemonic: &str,
        network: AptosNetwork,
        account: u32,
        address_index: u32,
    ) -> Result<Self, AptosError> {
        use bip39::{Mnemonic, Language, Seed};

        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| AptosError::InvalidMnemonic(e.to_string()))?;

        let seed = Seed::new(&mnemonic, "");

        // Use SLIP-10 for Ed25519 derivation
        // Path: m/44'/637'/account'/0'/address_index'
        // All indices are hardened (add 0x80000000)
        let indices: [u32; 5] = [
            44 | 0x80000000,              // purpose (hardened)
            Self::COIN_TYPE | 0x80000000, // coin type (hardened)
            account | 0x80000000,         // account (hardened)
            0x80000000,               // change (hardened)
            address_index | 0x80000000,   // address index (hardened)
        ];

        let derived_key = slip10_ed25519::derive_ed25519_private_key(seed.as_bytes(), &indices);

        Self::from_private_key_bytes(&derived_key, network)
    }

    /// Creates a wallet from a private key (32 bytes)
    pub fn from_private_key_bytes(bytes: &[u8], network: AptosNetwork) -> Result<Self, AptosError> {
        if bytes.len() != 32 {
            return Err(AptosError::InvalidPrivateKey(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = AptosAddress::from_ed25519_pubkey(&verifying_key);

        Ok(Self {
            signing_key,
            verifying_key,
            address,
            network,
        })
    }

    /// Creates a wallet from a hex-encoded private key
    pub fn from_private_key_hex(hex_str: &str, network: AptosNetwork) -> Result<Self, AptosError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)
            .map_err(|e| AptosError::InvalidPrivateKey(e.to_string()))?;
        Self::from_private_key_bytes(&bytes, network)
    }

    /// Returns the wallet address
    pub fn address(&self) -> &AptosAddress {
        &self.address
    }

    /// Returns the network
    pub fn network(&self) -> AptosNetwork {
        self.network
    }

    /// Returns the public key bytes
    pub fn public_key(&self) -> &[u8; 32] {
        self.verifying_key.as_bytes()
    }

    /// Returns the public key as hex
    pub fn public_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.verifying_key.as_bytes()))
    }

    /// Returns the private key bytes
    /// ⚠️ Handle with care!
    pub fn private_key(&self) -> &[u8; 32] {
        self.signing_key.as_bytes()
    }

    /// Returns the private key as hex
    /// ⚠️ Handle with care!
    pub fn private_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.signing_key.as_bytes()))
    }

    /// Signs arbitrary data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Signs data and returns the signature as bytes
    pub fn sign_bytes(&self, data: &[u8]) -> [u8; 64] {
        self.sign(data).to_bytes()
    }

    /// Signs a transaction (raw signing message)
    /// In Aptos, signing_message = prefix || sha3_256(raw_txn) 
    pub fn sign_transaction(&self, signing_message: &[u8]) -> Result<AptosSignature, AptosError> {
        let signature = self.signing_key.sign(signing_message);

        Ok(AptosSignature {
            public_key: self.verifying_key.as_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(),
        })
    }

    /// Creates an authenticator for transaction submission
    pub fn create_authenticator(&self, signature: &AptosSignature) -> TransactionAuthenticator {
        TransactionAuthenticator::Ed25519 {
            public_key: signature.public_key.clone(),
            signature: signature.signature.clone(),
        }
    }

    /// Returns the auth key for this account
    /// Auth key = SHA3-256(pubkey || 0x00)
    pub fn auth_key(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(self.verifying_key.as_bytes());
        hasher.update([0x00]);
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }

    /// Returns the auth key as hex
    pub fn auth_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.auth_key()))
    }
}

impl fmt::Debug for AptosWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AptosWallet")
            .field("address", &self.address.to_hex())
            .field("network", &self.network)
            .finish_non_exhaustive()
    }
}

/// Aptos signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosSignature {
    /// Public key bytes (32 bytes)
    pub public_key: Vec<u8>,
    /// Signature bytes (64 bytes)
    pub signature: Vec<u8>,
}

impl AptosSignature {
    /// Returns the public key as hex
    pub fn public_key_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.public_key))
    }

    /// Returns the signature as hex
    pub fn signature_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.signature))
    }
}

/// Transaction authenticator for Aptos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Ed25519 single-key authenticator
    Ed25519 {
        /// Public key
        public_key: Vec<u8>,
        /// Signature
        signature: Vec<u8>,
    },
    /// Multi-Ed25519 authenticator (for multi-sig)
    MultiEd25519 {
        /// Public keys
        public_keys: Vec<Vec<u8>>,
        /// Signatures
        signatures: Vec<Vec<u8>>,
        /// Bitmap of which keys signed
        bitmap: Vec<u8>,
    },
}

impl TransactionAuthenticator {
    /// Returns the type tag for BCS serialization
    pub fn type_tag(&self) -> u8 {
        match self {
            TransactionAuthenticator::Ed25519 { .. } => 0,
            TransactionAuthenticator::MultiEd25519 { .. } => 1,
        }
    }
}

// ============================================================================
// Account Resource Types
// ============================================================================

/// Aptos account resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResource {
    /// Sequence number (nonce)
    pub sequence_number: u64,
    /// Authentication key
    pub authentication_key: String,
}

/// Coin store resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinStoreResource {
    /// Coin balance
    pub coin: CoinValue,
    /// Frozen status
    pub frozen: bool,
}

/// Coin value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinValue {
    /// Value in base units
    pub value: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_aptos_amount_from_octas() {
        let amount = AptosAmount::from_octas(100_000_000);
        assert_eq!(amount.apt(), 1.0);
        assert_eq!(amount.octas(), 100_000_000);
    }

    #[test]
    fn test_aptos_amount_from_apt() {
        let amount = AptosAmount::from_apt(1.5);
        assert_eq!(amount.octas(), 150_000_000);
        assert!((amount.apt() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_aptos_amount_display() {
        let amount = AptosAmount::from_apt(1.234);
        let display = format!("{}", amount);
        assert!(display.contains("APT"));
    }

    #[test]
    fn test_aptos_amount_arithmetic() {
        let a = AptosAmount::from_apt(1.0);
        let b = AptosAmount::from_apt(0.5);
        
        let sum = a + b;
        assert!((sum.apt() - 1.5).abs() < 0.0001);
        
        let diff = a - b;
        assert!((diff.apt() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_aptos_amount_checked_ops() {
        let a = AptosAmount::from_octas(100);
        let b = AptosAmount::from_octas(50);
        
        assert!(a.checked_add(b).is_some());
        assert!(a.checked_sub(b).is_some());
        assert!(b.checked_sub(a).is_none());
    }

    #[test]
    fn test_aptos_amount_decimals() {
        assert_eq!(AptosAmount::decimals(), 8);
        assert_eq!(AptosAmount::symbol(), "APT");
    }

    #[test]
    fn test_aptos_address_from_hex() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let addr = AptosAddress::from_hex(hex).unwrap();
        assert_eq!(addr.to_hex(), hex);
    }

    #[test]
    fn test_aptos_address_short_form() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let addr = AptosAddress::from_hex(hex).unwrap();
        assert_eq!(addr.to_short_hex(), "0x1");
    }

    #[test]
    fn test_aptos_address_padding() {
        // Short address should be padded
        let addr = AptosAddress::from_hex("0x1").unwrap();
        assert_eq!(addr.to_short_hex(), "0x1");
        assert_eq!(addr.to_hex().len(), 66); // 0x + 64 chars
    }

    #[test]
    fn test_aptos_address_invalid() {
        // Invalid hex
        assert!(AptosAddress::from_hex("0xGGGG").is_err());
    }

    #[test]
    fn test_aptos_network_rest_url() {
        assert!(AptosNetwork::Mainnet.rest_url().contains("mainnet"));
        assert!(AptosNetwork::Testnet.rest_url().contains("testnet"));
        assert!(AptosNetwork::Devnet.rest_url().contains("devnet"));
    }

    #[test]
    fn test_aptos_network_faucet() {
        assert!(AptosNetwork::Testnet.faucet_url().is_some());
        assert!(AptosNetwork::Devnet.faucet_url().is_some());
        assert!(AptosNetwork::Mainnet.faucet_url().is_none());
    }

    #[test]
    fn test_aptos_network_chain_id() {
        assert_eq!(AptosNetwork::Mainnet.chain_id(), 1);
        assert_eq!(AptosNetwork::Testnet.chain_id(), 2);
        assert_eq!(AptosNetwork::Devnet.chain_id(), 3);
    }

    #[test]
    fn test_aptos_wallet_new() {
        let wallet = AptosWallet::new(AptosNetwork::Testnet);
        assert!(wallet.address().to_hex().starts_with("0x"));
        assert_eq!(wallet.address().to_hex().len(), 66);
    }

    #[test]
    fn test_aptos_wallet_from_mnemonic() {
        let wallet = AptosWallet::from_mnemonic(TEST_MNEMONIC, AptosNetwork::Mainnet).unwrap();
        
        let addr = wallet.address().to_hex();
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 66);
        
        // Same mnemonic should produce same address
        let wallet2 = AptosWallet::from_mnemonic(TEST_MNEMONIC, AptosNetwork::Mainnet).unwrap();
        assert_eq!(wallet.address(), wallet2.address());
    }

    #[test]
    fn test_aptos_wallet_different_accounts() {
        let wallet0 = AptosWallet::from_mnemonic_with_path(TEST_MNEMONIC, AptosNetwork::Mainnet, 0, 0).unwrap();
        let wallet1 = AptosWallet::from_mnemonic_with_path(TEST_MNEMONIC, AptosNetwork::Mainnet, 0, 1).unwrap();
        let wallet2 = AptosWallet::from_mnemonic_with_path(TEST_MNEMONIC, AptosNetwork::Mainnet, 1, 0).unwrap();
        
        assert_ne!(wallet0.address(), wallet1.address());
        assert_ne!(wallet0.address(), wallet2.address());
        assert_ne!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_aptos_wallet_from_private_key() {
        let wallet1 = AptosWallet::new(AptosNetwork::Testnet);
        let private_key = wallet1.private_key_hex();
        
        let wallet2 = AptosWallet::from_private_key_hex(&private_key, AptosNetwork::Testnet).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_aptos_wallet_sign() {
        let wallet = AptosWallet::from_mnemonic(TEST_MNEMONIC, AptosNetwork::Mainnet).unwrap();
        
        let message = b"Hello, Aptos!";
        let signature = wallet.sign(message);
        
        // Verify signature
        use ed25519_dalek::Verifier;
        assert!(wallet.verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_aptos_wallet_sign_transaction() {
        let wallet = AptosWallet::from_mnemonic(TEST_MNEMONIC, AptosNetwork::Mainnet).unwrap();
        
        let signing_message = vec![1, 2, 3, 4, 5];
        let sig = wallet.sign_transaction(&signing_message).unwrap();
        
        assert_eq!(sig.signature.len(), 64);
        assert_eq!(sig.public_key.len(), 32);
    }

    #[test]
    fn test_aptos_auth_key() {
        let wallet = AptosWallet::from_mnemonic(TEST_MNEMONIC, AptosNetwork::Mainnet).unwrap();
        
        let auth_key = wallet.auth_key();
        assert_eq!(auth_key.len(), 32);
        
        // Auth key should match address for Ed25519 single-key accounts
        assert_eq!(auth_key, *wallet.address().as_bytes());
    }

    #[test]
    fn test_aptos_wallet_invalid_mnemonic() {
        let result = AptosWallet::from_mnemonic("invalid mnemonic", AptosNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_aptos_wallet_invalid_private_key() {
        let result = AptosWallet::from_private_key_bytes(&[0u8; 16], AptosNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_authenticator() {
        let wallet = AptosWallet::new(AptosNetwork::Mainnet);
        let sig = wallet.sign_transaction(&[1, 2, 3]).unwrap();
        let auth = wallet.create_authenticator(&sig);
        
        assert_eq!(auth.type_tag(), 0); // Ed25519
    }

    #[test]
    fn test_aptos_signature_hex() {
        let wallet = AptosWallet::new(AptosNetwork::Testnet);
        let sig = wallet.sign_transaction(&[1, 2, 3]).unwrap();
        
        assert!(sig.public_key_hex().starts_with("0x"));
        assert!(sig.signature_hex().starts_with("0x"));
        assert_eq!(sig.public_key_hex().len(), 66);
        assert_eq!(sig.signature_hex().len(), 130);
    }
}
