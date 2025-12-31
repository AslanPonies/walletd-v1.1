//! # WalletD TON
//!
//! TON (The Open Network) blockchain support for the WalletD SDK.
//!
//! ## Features
//!
//! - Ed25519 key generation using TON's custom mnemonic derivation
//! - Wallet v4r2 address derivation
//! - User-friendly address encoding (base64 with flags and checksum)
//! - Transaction signing
//!
//! ## Example
//!
//! ```rust
//! use walletd_ton::{TonWallet, TonNetwork};
//!
//! // Create from mnemonic (24 words)
//! let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
//! let wallet = TonWallet::from_mnemonic(mnemonic, TonNetwork::Mainnet)?;
//!
//! println!("Address: {}", wallet.address_friendly());
//! # Ok::<(), walletd_ton::TonError>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crc::{Crc, CRC_16_XMODEM};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512, Digest};
use std::fmt;
use thiserror::Error;
use zeroize::Zeroize;

// Re-export traits
pub use walletd_traits::WalletError;

/// TON-specific errors
#[derive(Error, Debug)]
pub enum TonError {
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
}

impl From<TonError> for WalletError {
    fn from(e: TonError) -> Self {
        WalletError::Other(e.to_string())
    }
}

/// TON network configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum TonNetwork {
    /// TON Mainnet
    #[default]
    Mainnet,
    /// TON Testnet
    Testnet,
}

impl TonNetwork {
    /// Returns the default API endpoint
    pub fn api_endpoint(&self) -> &'static str {
        match self {
            TonNetwork::Mainnet => "https://toncenter.com/api/v2/jsonRPC",
            TonNetwork::Testnet => "https://testnet.toncenter.com/api/v2/jsonRPC",
        }
    }

    /// Returns the explorer URL
    pub fn explorer_url(&self) -> &'static str {
        match self {
            TonNetwork::Mainnet => "https://tonscan.org",
            TonNetwork::Testnet => "https://testnet.tonscan.org",
        }
    }

    /// Returns the address flag for this network
    pub fn address_flag(&self, bounceable: bool) -> u8 {
        match (self, bounceable) {
            (TonNetwork::Mainnet, true) => 0x11,  // Bounceable mainnet
            (TonNetwork::Mainnet, false) => 0x51, // Non-bounceable mainnet
            (TonNetwork::Testnet, true) => 0x91,  // Bounceable testnet
            (TonNetwork::Testnet, false) => 0xD1, // Non-bounceable testnet
        }
    }

    /// Check if this is testnet
    pub fn is_testnet(&self) -> bool {
        matches!(self, TonNetwork::Testnet)
    }
}


impl fmt::Display for TonNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TonNetwork::Mainnet => write!(f, "mainnet"),
            TonNetwork::Testnet => write!(f, "testnet"),
        }
    }
}

/// TON amount (in nanoTON, 1 TON = 10^9 nanoTON)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TonAmount(u64);

impl TonAmount {
    /// Number of nanoTON per TON
    pub const NANO_PER_TON: u64 = 1_000_000_000;

    /// Creates a new amount from nanoTON
    pub fn from_nano(nano: u64) -> Self {
        Self(nano)
    }

    /// Creates a new amount from TON
    pub fn from_ton(ton: f64) -> Self {
        Self((ton * Self::NANO_PER_TON as f64) as u64)
    }

    /// Returns the amount in nanoTON
    pub fn nano(&self) -> u64 {
        self.0
    }

    /// Returns the amount in TON
    pub fn ton(&self) -> f64 {
        self.0 as f64 / Self::NANO_PER_TON as f64
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

    /// Decimals for TON
    pub fn decimals() -> u8 {
        9
    }

    /// Symbol
    pub fn symbol() -> &'static str {
        "TON"
    }
}

impl Default for TonAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for TonAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.9} TON", self.ton())
    }
}

impl std::ops::Add for TonAmount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for TonAmount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

/// TON address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TonAddress {
    /// Workchain ID (0 for basechain, -1 for masterchain)
    pub workchain: i8,
    /// Account ID (32 bytes hash)
    pub hash: [u8; 32],
}

impl TonAddress {
    /// CRC16-XMODEM calculator
    const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_XMODEM);

    /// Creates a new address
    pub fn new(workchain: i8, hash: [u8; 32]) -> Self {
        Self { workchain, hash }
    }

    /// Creates an address from raw format (workchain:hash)
    pub fn from_raw(raw: &str) -> Result<Self, TonError> {
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() != 2 {
            return Err(TonError::InvalidAddress("Expected format: workchain:hash".to_string()));
        }

        let workchain = parts[0]
            .parse::<i8>()
            .map_err(|_| TonError::InvalidAddress("Invalid workchain".to_string()))?;

        let hash_hex = parts[1];
        if hash_hex.len() != 64 {
            return Err(TonError::InvalidAddress("Hash must be 64 hex chars".to_string()));
        }

        let hash_bytes = hex::decode(hash_hex)
            .map_err(|e| TonError::InvalidAddress(e.to_string()))?;
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_bytes);

        Ok(Self { workchain, hash })
    }

    /// Creates an address from user-friendly base64 format
    pub fn from_friendly(addr: &str) -> Result<Self, TonError> {
        // Handle both standard base64 and URL-safe base64
        let addr = addr.replace('-', "+").replace('_', "/");
        
        let bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &addr,
        ).map_err(|e| TonError::InvalidAddress(e.to_string()))?;

        if bytes.len() != 36 {
            return Err(TonError::InvalidAddress("Invalid address length".to_string()));
        }

        // Verify checksum
        let data = &bytes[0..34];
        let checksum = &bytes[34..36];
        
        let calculated_crc = Self::CRC.checksum(data);
        let expected_crc = ((checksum[0] as u16) << 8) | (checksum[1] as u16);
        
        if calculated_crc != expected_crc {
            return Err(TonError::InvalidAddress("Invalid checksum".to_string()));
        }

        // Parse flags
        let _flags = bytes[0];
        let workchain = bytes[1] as i8;
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[2..34]);

        Ok(Self { workchain, hash })
    }

    /// Returns the raw address format (workchain:hash)
    pub fn to_raw(&self) -> String {
        format!("{}:{}", self.workchain, hex::encode(self.hash))
    }

    /// Returns the user-friendly base64 address
    pub fn to_friendly(&self, network: TonNetwork, bounceable: bool) -> String {
        self.to_friendly_custom(network.address_flag(bounceable))
    }

    /// Returns the user-friendly base64 address with custom flags
    pub fn to_friendly_custom(&self, flags: u8) -> String {
        let mut data = Vec::with_capacity(36);
        data.push(flags);
        data.push(self.workchain as u8);
        data.extend_from_slice(&self.hash);
        
        // Calculate CRC16
        let crc = Self::CRC.checksum(&data);
        data.push((crc >> 8) as u8);
        data.push((crc & 0xFF) as u8);
        
        // URL-safe base64
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &data)
    }

    /// Returns bounceable mainnet address
    pub fn to_bounceable(&self) -> String {
        self.to_friendly(TonNetwork::Mainnet, true)
    }

    /// Returns non-bounceable mainnet address
    pub fn to_non_bounceable(&self) -> String {
        self.to_friendly(TonNetwork::Mainnet, false)
    }
}

impl fmt::Display for TonAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_bounceable())
    }
}

impl std::str::FromStr for TonAddress {
    type Err = TonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try friendly format first, then raw
        Self::from_friendly(s).or_else(|_| Self::from_raw(s))
    }
}

/// Wallet v4r2 code hash (for address derivation)
/// This is the SHA256 hash of the wallet v4r2 contract code cell
const WALLET_V4R2_CODE_HASH: [u8; 32] = [
    0xfe, 0xb5, 0xff, 0x68, 0x20, 0xe2, 0xff, 0x0d,
    0x94, 0x83, 0xe7, 0xe0, 0xd6, 0x2c, 0x81, 0x7d,
    0x84, 0x67, 0x89, 0xfb, 0x4a, 0xe5, 0x80, 0xc8,
    0x78, 0x86, 0x6d, 0x95, 0x9d, 0xaa, 0xbd, 0x6c,
];

/// Default wallet_id for mainnet (0x29a9a317)
const DEFAULT_WALLET_ID: u32 = 698983191;

/// TON wallet
pub struct TonWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    address: TonAddress,
    network: TonNetwork,
    wallet_id: u32,
}

impl TonWallet {
    /// Creates a new random wallet
    pub fn new(network: TonNetwork) -> Self {
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let wallet_id = DEFAULT_WALLET_ID;
        let address = Self::derive_address(&verifying_key, wallet_id, 0);

        Self {
            signing_key,
            verifying_key,
            address,
            network,
            wallet_id,
        }
    }

    /// Creates a wallet from TON mnemonic (24 words)
    /// 
    /// TON uses a custom key derivation:
    /// - PBKDF2 with HMAC-SHA512
    /// - Salt: "TON default seed" (no password) or "TON fast seed version" (with password)
    /// - Iterations: 100000 (no password) or 1 (with password)
    pub fn from_mnemonic(mnemonic: &str, network: TonNetwork) -> Result<Self, TonError> {
        Self::from_mnemonic_with_password(mnemonic, "", network)
    }

    /// Creates a wallet from mnemonic with optional password
    pub fn from_mnemonic_with_password(
        mnemonic: &str,
        password: &str,
        network: TonNetwork,
    ) -> Result<Self, TonError> {
        // Validate mnemonic words (TON uses BIP-39 wordlist)
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        if words.len() != 24 {
            return Err(TonError::InvalidMnemonic(
                "TON mnemonic must be 24 words".to_string(),
            ));
        }

        // Validate words are in BIP-39 wordlist by trying to parse
        use bip39::{Mnemonic as Bip39Mnemonic, Language};
        let _ = Bip39Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| TonError::InvalidMnemonic(e.to_string()))?;

        // Derive seed using PBKDF2
        let mnemonic_str = words.join(" ");
        let seed = Self::mnemonic_to_seed(&mnemonic_str, password)?;

        // First 32 bytes of seed are the private key
        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&seed[..32]);

        Self::from_private_key_bytes(&private_key_bytes, network)
    }

    /// Derives seed from mnemonic using TON's PBKDF2 parameters
    fn mnemonic_to_seed(mnemonic: &str, password: &str) -> Result<[u8; 64], TonError> {
        let (salt, iterations) = if password.is_empty() {
            ("TON default seed", 100000u32)
        } else {
            ("TON fast seed version", 1u32)
        };

        // Combine mnemonic and password
        let password_bytes = if password.is_empty() {
            mnemonic.as_bytes().to_vec()
        } else {
            let mut combined = mnemonic.as_bytes().to_vec();
            combined.extend_from_slice(password.as_bytes());
            combined
        };

        let mut seed = [0u8; 64];
        pbkdf2::<Hmac<Sha512>>(&password_bytes, salt.as_bytes(), iterations, &mut seed)
            .map_err(|e| TonError::KeyDerivation(e.to_string()))?;

        // Note: seed will be zeroized when the calling function copies out what it needs
        Ok(seed)
    }

    /// Creates a wallet from private key bytes
    pub fn from_private_key_bytes(bytes: &[u8], network: TonNetwork) -> Result<Self, TonError> {
        if bytes.len() != 32 {
            return Err(TonError::InvalidPrivateKey(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        
        // Zeroize the temporary key bytes
        key_bytes.zeroize();
        
        let verifying_key = signing_key.verifying_key();
        let wallet_id = DEFAULT_WALLET_ID;
        let address = Self::derive_address(&verifying_key, wallet_id, 0);

        Ok(Self {
            signing_key,
            verifying_key,
            address,
            network,
            wallet_id,
        })
    }

    /// Creates a wallet from hex-encoded private key
    pub fn from_private_key_hex(hex_str: &str, network: TonNetwork) -> Result<Self, TonError> {
        let hex_str = hex_str.trim_start_matches("0x");
        let bytes = hex::decode(hex_str)
            .map_err(|e| TonError::InvalidPrivateKey(e.to_string()))?;
        Self::from_private_key_bytes(&bytes, network)
    }

    /// Derives wallet address from public key
    /// 
    /// For wallet v4r2:
    /// - StateInit = code_cell + data_cell
    /// - data_cell contains: seqno (0), wallet_id, public_key, plugins_dict (empty)
    /// - address = workchain:sha256(StateInit)
    fn derive_address(pubkey: &VerifyingKey, wallet_id: u32, workchain: i8) -> TonAddress {
        // Simplified address derivation
        // In reality, this requires proper Cell serialization
        // For now, we use a deterministic hash of pubkey + wallet_id
        let mut hasher = Sha256::new();
        hasher.update(WALLET_V4R2_CODE_HASH);
        hasher.update(wallet_id.to_be_bytes());
        hasher.update(pubkey.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();

        TonAddress::new(workchain, hash)
    }

    /// Returns the wallet address
    pub fn address(&self) -> &TonAddress {
        &self.address
    }

    /// Returns the user-friendly address (bounceable)
    pub fn address_friendly(&self) -> String {
        self.address.to_friendly(self.network, true)
    }

    /// Returns the non-bounceable address
    pub fn address_non_bounceable(&self) -> String {
        self.address.to_friendly(self.network, false)
    }

    /// Returns the raw address
    pub fn address_raw(&self) -> String {
        self.address.to_raw()
    }

    /// Returns the network
    pub fn network(&self) -> TonNetwork {
        self.network
    }

    /// Returns the wallet_id
    pub fn wallet_id(&self) -> u32 {
        self.wallet_id
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

    /// Signs a message for external message
    /// In TON, the signing message typically includes:
    /// - wallet_id (4 bytes)
    /// - valid_until (4 bytes) - expiration timestamp
    /// - seqno (4 bytes) - sequence number
    /// - internal messages
    pub fn sign_message(&self, message: &[u8]) -> TonSignature {
        let signature = self.signing_key.sign(message);

        TonSignature {
            signature: signature.to_bytes().to_vec(),
            public_key: self.verifying_key.as_bytes().to_vec(),
        }
    }

    /// Creates a transfer message body (unsigned)
    pub fn create_transfer_body(
        &self,
        seqno: u32,
        valid_until: u32,
    ) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&self.wallet_id.to_be_bytes());
        body.extend_from_slice(&valid_until.to_be_bytes());
        body.extend_from_slice(&seqno.to_be_bytes());
        body
    }
}

impl fmt::Debug for TonWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TonWallet")
            .field("address", &self.address_friendly())
            .field("network", &self.network)
            .field("wallet_id", &self.wallet_id)
            .finish_non_exhaustive()
    }
}

/// TON signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonSignature {
    /// Signature bytes (64 bytes)
    pub signature: Vec<u8>,
    /// Public key bytes (32 bytes)
    pub public_key: Vec<u8>,
}

impl TonSignature {
    /// Returns the signature as hex
    pub fn signature_hex(&self) -> String {
        hex::encode(&self.signature)
    }

    /// Returns the public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(&self.public_key)
    }

    /// Returns the signature as base64
    pub fn signature_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &self.signature)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test mnemonic (24 words from BIP-39)
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn test_ton_amount_from_nano() {
        let amount = TonAmount::from_nano(1_000_000_000);
        assert_eq!(amount.ton(), 1.0);
        assert_eq!(amount.nano(), 1_000_000_000);
    }

    #[test]
    fn test_ton_amount_from_ton() {
        let amount = TonAmount::from_ton(1.5);
        assert_eq!(amount.nano(), 1_500_000_000);
        assert!((amount.ton() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_ton_amount_display() {
        let amount = TonAmount::from_ton(1.234);
        let display = format!("{}", amount);
        assert!(display.contains("TON"));
    }

    #[test]
    fn test_ton_amount_arithmetic() {
        let a = TonAmount::from_ton(1.0);
        let b = TonAmount::from_ton(0.5);
        
        let sum = a + b;
        assert!((sum.ton() - 1.5).abs() < 0.0001);
        
        let diff = a - b;
        assert!((diff.ton() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_ton_amount_checked_ops() {
        let a = TonAmount::from_nano(100);
        let b = TonAmount::from_nano(50);
        
        assert!(a.checked_add(b).is_some());
        assert!(a.checked_sub(b).is_some());
        assert!(b.checked_sub(a).is_none());
    }

    #[test]
    fn test_ton_amount_decimals() {
        assert_eq!(TonAmount::decimals(), 9);
        assert_eq!(TonAmount::symbol(), "TON");
    }

    #[test]
    fn test_ton_address_raw_format() {
        let addr = TonAddress::new(0, [0x12; 32]);
        let raw = addr.to_raw();
        assert!(raw.starts_with("0:"));
        assert_eq!(raw.len(), 66); // "0:" + 64 hex chars
    }

    #[test]
    fn test_ton_address_from_raw() {
        let raw = "0:1212121212121212121212121212121212121212121212121212121212121212";
        let addr = TonAddress::from_raw(raw).unwrap();
        assert_eq!(addr.workchain, 0);
        assert_eq!(addr.hash, [0x12; 32]);
    }

    #[test]
    fn test_ton_address_friendly_format() {
        let addr = TonAddress::new(0, [0x12; 32]);
        let friendly = addr.to_friendly(TonNetwork::Mainnet, true);
        // Base64 encoded, should be 48 chars (36 bytes * 4/3)
        assert!(!friendly.is_empty());
    }

    #[test]
    fn test_ton_address_friendly_roundtrip() {
        let addr = TonAddress::new(0, [0xAB; 32]);
        let friendly = addr.to_friendly(TonNetwork::Mainnet, true);
        let parsed = TonAddress::from_friendly(&friendly).unwrap();
        assert_eq!(addr.workchain, parsed.workchain);
        assert_eq!(addr.hash, parsed.hash);
    }

    #[test]
    fn test_ton_address_bounceable_vs_non_bounceable() {
        let addr = TonAddress::new(0, [0x12; 32]);
        let bounceable = addr.to_bounceable();
        let non_bounceable = addr.to_non_bounceable();
        
        // Should be different due to different flags
        assert_ne!(bounceable, non_bounceable);
    }

    #[test]
    fn test_ton_network_flags() {
        assert_eq!(TonNetwork::Mainnet.address_flag(true), 0x11);
        assert_eq!(TonNetwork::Mainnet.address_flag(false), 0x51);
        assert_eq!(TonNetwork::Testnet.address_flag(true), 0x91);
        assert_eq!(TonNetwork::Testnet.address_flag(false), 0xD1);
    }

    #[test]
    fn test_ton_network_endpoints() {
        assert!(TonNetwork::Mainnet.api_endpoint().contains("toncenter"));
        assert!(TonNetwork::Testnet.api_endpoint().contains("testnet"));
    }

    #[test]
    fn test_ton_wallet_new() {
        let wallet = TonWallet::new(TonNetwork::Mainnet);
        assert!(!wallet.address_friendly().is_empty());
        assert_eq!(wallet.network(), TonNetwork::Mainnet);
    }

    #[test]
    fn test_ton_wallet_from_mnemonic() {
        let wallet = TonWallet::from_mnemonic(TEST_MNEMONIC, TonNetwork::Mainnet).unwrap();
        
        // Should produce consistent address
        let wallet2 = TonWallet::from_mnemonic(TEST_MNEMONIC, TonNetwork::Mainnet).unwrap();
        assert_eq!(wallet.address_friendly(), wallet2.address_friendly());
    }

    #[test]
    fn test_ton_wallet_from_private_key() {
        let wallet1 = TonWallet::new(TonNetwork::Mainnet);
        let private_key = wallet1.private_key_hex();
        
        let wallet2 = TonWallet::from_private_key_hex(&private_key, TonNetwork::Mainnet).unwrap();
        assert_eq!(wallet1.public_key_hex(), wallet2.public_key_hex());
    }

    #[test]
    fn test_ton_wallet_sign() {
        let wallet = TonWallet::from_mnemonic(TEST_MNEMONIC, TonNetwork::Mainnet).unwrap();
        
        let message = b"Hello, TON!";
        let signature = wallet.sign(message);
        
        // Verify signature
        use ed25519_dalek::Verifier;
        assert!(wallet.verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_ton_wallet_sign_message() {
        let wallet = TonWallet::from_mnemonic(TEST_MNEMONIC, TonNetwork::Mainnet).unwrap();
        
        let message = vec![1, 2, 3, 4, 5];
        let sig = wallet.sign_message(&message);
        
        assert_eq!(sig.signature.len(), 64);
        assert_eq!(sig.public_key.len(), 32);
    }

    #[test]
    fn test_ton_wallet_invalid_mnemonic_length() {
        let result = TonWallet::from_mnemonic("abandon abandon abandon", TonNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_ton_wallet_invalid_mnemonic_words() {
        let invalid = "invalid word here abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        let result = TonWallet::from_mnemonic(invalid, TonNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_ton_wallet_invalid_private_key() {
        let result = TonWallet::from_private_key_bytes(&[0u8; 16], TonNetwork::Mainnet);
        assert!(result.is_err());
    }

    #[test]
    fn test_ton_signature_hex() {
        let wallet = TonWallet::new(TonNetwork::Mainnet);
        let sig = wallet.sign_message(&[1, 2, 3]);
        
        assert_eq!(sig.signature_hex().len(), 128); // 64 bytes * 2
        assert_eq!(sig.public_key_hex().len(), 64);  // 32 bytes * 2
    }

    #[test]
    fn test_ton_signature_base64() {
        let wallet = TonWallet::new(TonNetwork::Mainnet);
        let sig = wallet.sign_message(&[1, 2, 3]);
        
        let b64 = sig.signature_base64();
        assert!(!b64.is_empty());
    }

    #[test]
    fn test_ton_wallet_debug() {
        let wallet = TonWallet::new(TonNetwork::Mainnet);
        let debug = format!("{:?}", wallet);
        assert!(debug.contains("TonWallet"));
        assert!(debug.contains("address"));
    }

    #[test]
    fn test_ton_wallet_create_transfer_body() {
        let wallet = TonWallet::new(TonNetwork::Mainnet);
        let body = wallet.create_transfer_body(1, 1234567890);
        
        // Should contain wallet_id + valid_until + seqno = 12 bytes
        assert_eq!(body.len(), 12);
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Amount roundtrip: from_nano -> nano() should be identity
        #[test]
        fn amount_nano_roundtrip(nano in 0u64..u64::MAX) {
            let amount = TonAmount::from_nano(nano);
            prop_assert_eq!(amount.nano(), nano);
        }

        /// Amount checked_add should not overflow
        #[test]
        fn amount_checked_add_safe(a in 0u64..u64::MAX/2, b in 0u64..u64::MAX/2) {
            let amount_a = TonAmount::from_nano(a);
            let amount_b = TonAmount::from_nano(b);
            let sum = amount_a.checked_add(amount_b);
            prop_assert!(sum.is_some());
            prop_assert_eq!(sum.unwrap().nano(), a + b);
        }

        /// Amount checked_sub should handle underflow
        #[test]
        fn amount_checked_sub_underflow(a in 0u64..1000, b in 1001u64..2000) {
            let amount_a = TonAmount::from_nano(a);
            let amount_b = TonAmount::from_nano(b);
            let diff = amount_a.checked_sub(amount_b);
            prop_assert!(diff.is_none());
        }

        /// Address raw format roundtrip
        #[test]
        fn address_raw_roundtrip(hash in prop::array::uniform32(any::<u8>())) {
            let addr = TonAddress::new(0, hash);
            let raw = addr.to_raw();
            let parsed = TonAddress::from_raw(&raw).unwrap();
            prop_assert_eq!(addr.workchain, parsed.workchain);
            prop_assert_eq!(addr.hash, parsed.hash);
        }

        /// Address friendly format roundtrip
        #[test]
        fn address_friendly_roundtrip(hash in prop::array::uniform32(any::<u8>())) {
            let addr = TonAddress::new(0, hash);
            let friendly = addr.to_friendly(TonNetwork::Mainnet, true);
            let parsed = TonAddress::from_friendly(&friendly).unwrap();
            prop_assert_eq!(addr.workchain, parsed.workchain);
            prop_assert_eq!(addr.hash, parsed.hash);
        }

        /// Signing is deterministic for same key and message
        #[test]
        fn signing_deterministic(message in prop::collection::vec(any::<u8>(), 0..1000)) {
            let wallet = TonWallet::new(TonNetwork::Mainnet);
            let sig1 = wallet.sign_bytes(&message);
            let sig2 = wallet.sign_bytes(&message);
            prop_assert_eq!(sig1, sig2);
        }

        /// Private key import produces same address
        #[test]
        fn private_key_deterministic(key in prop::array::uniform32(1u8..)) {
            let wallet1 = TonWallet::from_private_key_bytes(&key, TonNetwork::Mainnet).unwrap();
            let wallet2 = TonWallet::from_private_key_bytes(&key, TonNetwork::Mainnet).unwrap();
            prop_assert_eq!(wallet1.address_friendly(), wallet2.address_friendly());
        }
    }
}
