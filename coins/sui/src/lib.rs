//! # WalletD SUI
//!
//! SUI blockchain support for the WalletD SDK.
//!
//! ## Features
//!
//! - Ed25519 key generation and management
//! - SUI address derivation (0x prefixed, 64 hex chars)
//! - Transaction signing
//! - BIP-44 HD derivation (m/44'/784'/0'/0'/0')
//!
//! ## Example
//!
//! ```rust
//! use walletd_sui::{SuiWallet, SuiNetwork};
//!
//! // Create from mnemonic
//! let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//! let wallet = SuiWallet::from_mnemonic(mnemonic, SuiNetwork::Mainnet)?;
//!
//! println!("Address: {}", wallet.address());
//! # Ok::<(), walletd_sui::SuiError>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use blake2::{Blake2b, Digest};
use blake2::digest::consts::U32;

/// Type alias for Blake2b with 256-bit output
type Blake2b256 = Blake2b<U32>;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// Re-export traits
pub use walletd_traits::WalletError;

/// SUI-specific errors
#[derive(Error, Debug)]
pub enum SuiError {
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

impl From<SuiError> for WalletError {
    fn from(e: SuiError) -> Self {
        WalletError::Other(e.to_string())
    }
}

/// SUI network configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SuiNetwork {
    /// SUI Mainnet
    #[default]
    Mainnet,
    /// SUI Testnet
    Testnet,
    /// SUI Devnet
    Devnet,
    /// Local network
    Localnet,
}

impl SuiNetwork {
    /// Returns the RPC URL for this network
    pub fn rpc_url(&self) -> &'static str {
        match self {
            SuiNetwork::Mainnet => "https://fullnode.mainnet.sui.io:443",
            SuiNetwork::Testnet => "https://fullnode.testnet.sui.io:443",
            SuiNetwork::Devnet => "https://fullnode.devnet.sui.io:443",
            SuiNetwork::Localnet => "http://127.0.0.1:9000",
        }
    }

    /// Returns the faucet URL (testnet/devnet only)
    pub fn faucet_url(&self) -> Option<&'static str> {
        match self {
            SuiNetwork::Testnet => Some("https://faucet.testnet.sui.io"),
            SuiNetwork::Devnet => Some("https://faucet.devnet.sui.io"),
            _ => None,
        }
    }

    /// Returns the chain identifier
    pub fn chain_id(&self) -> &'static str {
        match self {
            SuiNetwork::Mainnet => "sui:mainnet",
            SuiNetwork::Testnet => "sui:testnet",
            SuiNetwork::Devnet => "sui:devnet",
            SuiNetwork::Localnet => "sui:localnet",
        }
    }
}


impl fmt::Display for SuiNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuiNetwork::Mainnet => write!(f, "mainnet"),
            SuiNetwork::Testnet => write!(f, "testnet"),
            SuiNetwork::Devnet => write!(f, "devnet"),
            SuiNetwork::Localnet => write!(f, "localnet"),
        }
    }
}

/// SUI amount (in MIST, 1 SUI = 10^9 MIST)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SuiAmount(u64);

impl SuiAmount {
    /// Number of MIST per SUI
    pub const MIST_PER_SUI: u64 = 1_000_000_000;

    /// Creates a new amount from MIST
    pub fn from_mist(mist: u64) -> Self {
        Self(mist)
    }

    /// Creates a new amount from SUI
    pub fn from_sui(sui: f64) -> Self {
        Self((sui * Self::MIST_PER_SUI as f64) as u64)
    }

    /// Returns the amount in MIST
    pub fn mist(&self) -> u64 {
        self.0
    }

    /// Returns the amount in SUI
    pub fn sui(&self) -> f64 {
        self.0 as f64 / Self::MIST_PER_SUI as f64
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
}

impl Default for SuiAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for SuiAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.9} SUI", self.sui())
    }
}

impl std::ops::Add for SuiAmount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for SuiAmount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// SUI address (32 bytes, displayed as 0x + 64 hex chars)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SuiAddress([u8; 32]);

impl SuiAddress {
    /// Creates an address from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the address bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Creates an address from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, SuiError> {
        let hex_str = hex_str.trim_start_matches("0x");
        if hex_str.len() != 64 {
            return Err(SuiError::InvalidAddress(
                "Address must be 64 hex characters".to_string(),
            ));
        }
        let bytes = hex::decode(hex_str)
            .map_err(|e| SuiError::InvalidAddress(e.to_string()))?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Returns the address as hex string with 0x prefix
    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    /// Derives address from Ed25519 public key
    pub fn from_ed25519_pubkey(pubkey: &VerifyingKey) -> Self {
        // SUI address = Blake2b256(0x00 || pubkey)[0..32]
        // 0x00 is the signature scheme flag for Ed25519
        let mut hasher = Blake2b256::new();
        hasher.update([0x00]); // Ed25519 flag
        hasher.update(pubkey.as_bytes());
        let hash = hasher.finalize();
        
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&hash[..32]);
        Self(addr)
    }
}

impl fmt::Display for SuiAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl std::str::FromStr for SuiAddress {
    type Err = SuiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

/// SUI wallet
pub struct SuiWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    address: SuiAddress,
    network: SuiNetwork,
}

impl SuiWallet {
    /// BIP-44 coin type for SUI
    pub const COIN_TYPE: u32 = 784;

    /// Creates a new random wallet
    pub fn new(network: SuiNetwork) -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let address = SuiAddress::from_ed25519_pubkey(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            address,
            network,
        }
    }

    /// Creates a wallet from a mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, network: SuiNetwork) -> Result<Self, SuiError> {
        Self::from_mnemonic_with_path(mnemonic, network, 0, 0)
    }

    /// Creates a wallet from a mnemonic with custom derivation path
    /// Path: m/44'/784'/account'/change'/address_index'
    pub fn from_mnemonic_with_path(
        mnemonic: &str,
        network: SuiNetwork,
        account: u32,
        address_index: u32,
    ) -> Result<Self, SuiError> {
        use bip39::{Mnemonic, Language, Seed};

        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| SuiError::InvalidMnemonic(e.to_string()))?;

        let seed = Seed::new(&mnemonic, "");

        // Use SLIP-10 for Ed25519 derivation
        // Path: m/44'/784'/account'/0'/address_index'
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
    pub fn from_private_key_bytes(bytes: &[u8], network: SuiNetwork) -> Result<Self, SuiError> {
        if bytes.len() != 32 {
            return Err(SuiError::InvalidPrivateKey(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = SuiAddress::from_ed25519_pubkey(&verifying_key);

        Ok(Self {
            signing_key,
            verifying_key,
            address,
            network,
        })
    }

    /// Creates a wallet from a hex-encoded private key
    pub fn from_private_key_hex(hex_str: &str, network: SuiNetwork) -> Result<Self, SuiError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)
            .map_err(|e| SuiError::InvalidPrivateKey(e.to_string()))?;
        Self::from_private_key_bytes(&bytes, network)
    }

    /// Returns the wallet address
    pub fn address(&self) -> &SuiAddress {
        &self.address
    }

    /// Returns the network
    pub fn network(&self) -> SuiNetwork {
        self.network
    }

    /// Returns the public key bytes
    pub fn public_key(&self) -> &[u8; 32] {
        self.verifying_key.as_bytes()
    }

    /// Returns the public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Returns the private key bytes
    /// ⚠️ Handle with care!
    pub fn private_key(&self) -> &[u8; 32] {
        self.signing_key.as_bytes()
    }

    /// Returns the private key as hex
    /// ⚠️ Handle with care!
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.signing_key.as_bytes())
    }

    /// Signs arbitrary data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Signs data and returns the signature as bytes
    pub fn sign_bytes(&self, data: &[u8]) -> [u8; 64] {
        self.sign(data).to_bytes()
    }

    /// Signs a transaction intent message
    /// The intent message includes the intent scope (TransactionData = 0)
    pub fn sign_transaction(&self, tx_bytes: &[u8]) -> Result<SuiSignature, SuiError> {
        // Create intent message: intent_scope (1 byte) || intent_version (1 byte) || app_id (1 byte) || tx_bytes
        // For TransactionData: scope=0, version=0, app_id=0
        let mut intent_msg = vec![0u8, 0u8, 0u8];
        intent_msg.extend_from_slice(tx_bytes);

        // Hash the intent message
        let mut hasher = Blake2b256::new();
        hasher.update(&intent_msg);
        let digest = hasher.finalize();

        // Sign the digest
        let signature = self.signing_key.sign(&digest);

        Ok(SuiSignature {
            scheme: SignatureScheme::Ed25519,
            signature: signature.to_bytes().to_vec(),
            public_key: self.verifying_key.as_bytes().to_vec(),
        })
    }

    /// Exports the wallet as a SUI keystore format
    pub fn to_keystore(&self) -> String {
        // SUI keystore format: base64(flag || private_key || public_key)
        let mut bytes = vec![0x00]; // Ed25519 flag
        bytes.extend_from_slice(self.signing_key.as_bytes());
        bytes.extend_from_slice(self.verifying_key.as_bytes());
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
    }
}

impl fmt::Debug for SuiWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SuiWallet")
            .field("address", &self.address.to_hex())
            .field("network", &self.network)
            .finish_non_exhaustive()
    }
}

/// Signature scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureScheme {
    /// Ed25519
    Ed25519 = 0,
    /// Secp256k1
    Secp256k1 = 1,
    /// Secp256r1
    Secp256r1 = 2,
}

/// SUI signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiSignature {
    /// Signature scheme
    pub scheme: SignatureScheme,
    /// Raw signature bytes
    pub signature: Vec<u8>,
    /// Public key bytes
    pub public_key: Vec<u8>,
}

impl SuiSignature {
    /// Serializes the signature for RPC submission
    /// Format: flag (1 byte) || signature (64 bytes) || public_key (32 bytes)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![self.scheme as u8];
        bytes.extend_from_slice(&self.signature);
        bytes.extend_from_slice(&self.public_key);
        bytes
    }

    /// Encodes the signature as base64
    pub fn to_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, self.to_bytes())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_sui_amount_from_mist() {
        let amount = SuiAmount::from_mist(1_000_000_000);
        assert_eq!(amount.sui(), 1.0);
        assert_eq!(amount.mist(), 1_000_000_000);
    }

    #[test]
    fn test_sui_amount_from_sui() {
        let amount = SuiAmount::from_sui(1.5);
        assert_eq!(amount.mist(), 1_500_000_000);
        assert!((amount.sui() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_sui_amount_display() {
        let amount = SuiAmount::from_sui(1.234);
        let display = format!("{}", amount);
        assert!(display.contains("SUI"));
    }

    #[test]
    fn test_sui_amount_arithmetic() {
        let a = SuiAmount::from_sui(1.0);
        let b = SuiAmount::from_sui(0.5);
        
        let sum = a + b;
        assert!((sum.sui() - 1.5).abs() < 0.0001);
        
        let diff = a - b;
        assert!((diff.sui() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_sui_amount_checked_ops() {
        let a = SuiAmount::from_mist(100);
        let b = SuiAmount::from_mist(50);
        
        assert!(a.checked_add(b).is_some());
        assert!(a.checked_sub(b).is_some());
        assert!(b.checked_sub(a).is_none());
    }

    #[test]
    fn test_sui_address_from_hex() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let addr = SuiAddress::from_hex(hex).unwrap();
        assert_eq!(addr.to_hex(), hex);
    }

    #[test]
    fn test_sui_address_invalid() {
        // Too short
        assert!(SuiAddress::from_hex("0x123").is_err());
        // Invalid hex
        assert!(SuiAddress::from_hex("0xGGGG").is_err());
    }

    #[test]
    fn test_sui_network_rpc_url() {
        assert!(SuiNetwork::Mainnet.rpc_url().contains("mainnet"));
        assert!(SuiNetwork::Testnet.rpc_url().contains("testnet"));
        assert!(SuiNetwork::Devnet.rpc_url().contains("devnet"));
    }

    #[test]
    fn test_sui_network_faucet() {
        assert!(SuiNetwork::Testnet.faucet_url().is_some());
        assert!(SuiNetwork::Devnet.faucet_url().is_some());
        assert!(SuiNetwork::Mainnet.faucet_url().is_none());
    }

    #[test]
    fn test_sui_wallet_new() {
        let wallet = SuiWallet::new(SuiNetwork::Testnet);
        assert!(wallet.address().to_hex().starts_with("0x"));
        assert_eq!(wallet.address().to_hex().len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_sui_wallet_from_mnemonic() {
        let wallet = SuiWallet::from_mnemonic(TEST_MNEMONIC, SuiNetwork::Mainnet).unwrap();
        
        // Address should be deterministic
        let addr = wallet.address().to_hex();
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 66);
        
        // Same mnemonic should produce same address
        let wallet2 = SuiWallet::from_mnemonic(TEST_MNEMONIC, SuiNetwork::Mainnet).unwrap();
        assert_eq!(wallet.address(), wallet2.address());
    }

    #[test]
    fn test_sui_wallet_different_accounts() {
        let wallet0 = SuiWallet::from_mnemonic_with_path(TEST_MNEMONIC, SuiNetwork::Mainnet, 0, 0).unwrap();
        let wallet1 = SuiWallet::from_mnemonic_with_path(TEST_MNEMONIC, SuiNetwork::Mainnet, 0, 1).unwrap();
        let wallet2 = SuiWallet::from_mnemonic_with_path(TEST_MNEMONIC, SuiNetwork::Mainnet, 1, 0).unwrap();
        
        // Different indices should produce different addresses
        assert_ne!(wallet0.address(), wallet1.address());
        assert_ne!(wallet0.address(), wallet2.address());
        assert_ne!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_sui_wallet_from_private_key() {
        let wallet1 = SuiWallet::new(SuiNetwork::Testnet);
        let private_key = wallet1.private_key_hex();
        
        let wallet2 = SuiWallet::from_private_key_hex(&private_key, SuiNetwork::Testnet).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_sui_wallet_sign() {
        let wallet = SuiWallet::from_mnemonic(TEST_MNEMONIC, SuiNetwork::Mainnet).unwrap();
        
        let message = b"Hello, SUI!";
        let signature = wallet.sign(message);
        
        // Verify signature
        use ed25519_dalek::Verifier;
        assert!(wallet.verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_sui_wallet_sign_transaction() {
        let wallet = SuiWallet::from_mnemonic(TEST_MNEMONIC, SuiNetwork::Mainnet).unwrap();
        
        let tx_bytes = vec![1, 2, 3, 4, 5];
        let sig = wallet.sign_transaction(&tx_bytes).unwrap();
        
        assert_eq!(sig.scheme, SignatureScheme::Ed25519);
        assert_eq!(sig.signature.len(), 64);
        assert_eq!(sig.public_key.len(), 32);
    }

    #[test]
    fn test_sui_signature_serialization() {
        let wallet = SuiWallet::new(SuiNetwork::Testnet);
        let sig = wallet.sign_transaction(&[1, 2, 3]).unwrap();
        
        let bytes = sig.to_bytes();
        assert_eq!(bytes.len(), 1 + 64 + 32); // flag + sig + pubkey
        assert_eq!(bytes[0], 0x00); // Ed25519 flag
        
        let base64 = sig.to_base64();
        assert!(!base64.is_empty());
    }

    #[test]
    fn test_sui_wallet_keystore_export() {
        let wallet = SuiWallet::from_mnemonic(TEST_MNEMONIC, SuiNetwork::Mainnet).unwrap();
        let keystore = wallet.to_keystore();
        
        // Should be valid base64
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &keystore);
        assert!(decoded.is_ok());
        
        let bytes = decoded.unwrap();
        assert_eq!(bytes.len(), 1 + 32 + 32); // flag + private + public
        assert_eq!(bytes[0], 0x00); // Ed25519 flag
    }

    #[test]
    fn test_sui_wallet_invalid_mnemonic() {
        let result = SuiWallet::from_mnemonic("invalid mnemonic", SuiNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_sui_wallet_invalid_private_key() {
        let result = SuiWallet::from_private_key_bytes(&[0u8; 16], SuiNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_sui_amount_conversions() {
        // Test round-trip conversion
        let original = SuiAmount::from_sui(2.5);
        let mist = original.mist();
        let back = SuiAmount::from_mist(mist);
        assert_eq!(original, back);
    }
}
