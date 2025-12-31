use anyhow::Result;
use bip39::Mnemonic;
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::RngCore;
use std::str::FromStr;

use crate::address::CardanoAddress;
use crate::config::{NetworkConfig, MAINNET_NETWORK_ID, TESTNET_NETWORK_ID, LOVELACE_PER_ADA};

/// Cardano wallet for managing ADA
pub struct CardanoWallet {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    network_id: u8,
    config: NetworkConfig,
    address: CardanoAddress,
    api_key: Option<String>,
}

impl CardanoWallet {
    /// Create a new random wallet
    pub fn new(network_id: u8) -> Result<Self> {
        let mut csprng = rand::rngs::OsRng;
        let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
        csprng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        
        let config = if network_id == MAINNET_NETWORK_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::preview()
        };

        let address = CardanoAddress::enterprise(
            verifying_key.as_bytes(),
            network_id,
        )?;

        Ok(Self {
            signing_key,
            verifying_key,
            network_id,
            config,
            address,
            api_key: None,
        })
    }

    /// Create wallet on Cardano Mainnet
    pub fn mainnet() -> Result<Self> {
        Self::new(MAINNET_NETWORK_ID)
    }

    /// Create wallet on Cardano Testnet (Preview)
    pub fn testnet() -> Result<Self> {
        Self::new(TESTNET_NETWORK_ID)
    }

    /// Create wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, network_id: u8) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");

        // Cardano uses a specific key derivation (CIP-1852)
        // For simplicity, we'll use the first 32 bytes of seed for signing key
        // In production, use proper Cardano HD derivation
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[..32]);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        let config = if network_id == MAINNET_NETWORK_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::preview()
        };

        let address = CardanoAddress::enterprise(
            verifying_key.as_bytes(),
            network_id,
        )?;

        Ok(Self {
            signing_key,
            verifying_key,
            network_id,
            config,
            address,
            api_key: None,
        })
    }

    /// Create wallet from private key bytes
    pub fn from_private_key(private_key: &[u8], network_id: u8) -> Result<Self> {
        if private_key.len() != 32 {
            return Err(anyhow::anyhow!("Private key must be 32 bytes"));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(private_key);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        let config = if network_id == MAINNET_NETWORK_ID {
            NetworkConfig::mainnet()
        } else {
            NetworkConfig::preview()
        };

        let address = CardanoAddress::enterprise(
            verifying_key.as_bytes(),
            network_id,
        )?;

        Ok(Self {
            signing_key,
            verifying_key,
            network_id,
            config,
            address,
            api_key: None,
        })
    }

    /// Create wallet from hex-encoded private key
    pub fn from_private_key_hex(private_key: &str, network_id: u8) -> Result<Self> {
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);
        let bytes = hex::decode(key)?;
        Self::from_private_key(&bytes, network_id)
    }

    /// Set API key for Blockfrost or other providers
    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_string());
    }

    /// Get wallet address (bech32 encoded)
    pub fn address(&self) -> &str {
        self.address.to_bech32()
    }

    /// Get the CardanoAddress struct
    pub fn address_info(&self) -> &CardanoAddress {
        &self.address
    }

    /// Get public key as hex
    pub fn public_key(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    /// Get private key as hex (with 0x prefix)
    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signing_key.as_bytes()))
    }

    /// Get network ID
    pub fn network_id(&self) -> u8 {
        self.network_id
    }

    /// Get network config
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Check if mainnet
    pub fn is_mainnet(&self) -> bool {
        self.network_id == MAINNET_NETWORK_ID
    }

    /// Check if API key is set
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Verify a signature
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

    /// Get balance (placeholder - requires API)
    pub async fn get_balance(&self) -> Result<u64> {
        if self.api_key.is_none() {
            return Ok(0);
        }
        // In production, query Blockfrost or Koios API
        Ok(0)
    }

    /// Get balance as ADA
    pub async fn get_balance_ada(&self) -> Result<f64> {
        let lovelace = self.get_balance().await?;
        Ok(lovelace as f64 / LOVELACE_PER_ADA as f64)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // ========================================================================
    // Wallet Creation Tests
    // ========================================================================

    #[test]
    fn test_new_wallet_mainnet() {
        let wallet = CardanoWallet::mainnet().expect("Failed to create wallet");
        assert_eq!(wallet.network_id(), MAINNET_NETWORK_ID);
        assert!(wallet.is_mainnet());
    }

    #[test]
    fn test_new_wallet_testnet() {
        let wallet = CardanoWallet::testnet().expect("Failed to create wallet");
        assert_eq!(wallet.network_id(), TESTNET_NETWORK_ID);
        assert!(!wallet.is_mainnet());
    }

    #[test]
    fn test_wallet_address_format_mainnet() {
        let wallet = CardanoWallet::mainnet().unwrap();
        assert!(wallet.address().starts_with("addr1"));
    }

    #[test]
    fn test_wallet_address_format_testnet() {
        let wallet = CardanoWallet::testnet().unwrap();
        assert!(wallet.address().starts_with("addr_test1"));
    }

    #[test]
    fn test_random_wallets_different() {
        let wallet1 = CardanoWallet::mainnet().unwrap();
        let wallet2 = CardanoWallet::mainnet().unwrap();
        assert_ne!(wallet1.address(), wallet2.address());
    }

    // ========================================================================
    // Mnemonic Import Tests
    // ========================================================================

    #[test]
    fn test_from_mnemonic_mainnet() {
        let wallet = CardanoWallet::from_mnemonic(TEST_MNEMONIC, MAINNET_NETWORK_ID)
            .expect("Failed to create wallet from mnemonic");
        assert!(wallet.address().starts_with("addr1"));
    }

    #[test]
    fn test_from_mnemonic_testnet() {
        let wallet = CardanoWallet::from_mnemonic(TEST_MNEMONIC, TESTNET_NETWORK_ID)
            .expect("Failed to create wallet from mnemonic");
        assert!(wallet.address().starts_with("addr_test1"));
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let wallet1 = CardanoWallet::from_mnemonic(TEST_MNEMONIC, MAINNET_NETWORK_ID).unwrap();
        let wallet2 = CardanoWallet::from_mnemonic(TEST_MNEMONIC, MAINNET_NETWORK_ID).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
        assert_eq!(wallet1.public_key(), wallet2.public_key());
    }

    #[test]
    fn test_from_mnemonic_invalid() {
        let result = CardanoWallet::from_mnemonic("invalid mnemonic phrase", MAINNET_NETWORK_ID);
        assert!(result.is_err());
    }

    // ========================================================================
    // Private Key Import Tests
    // ========================================================================

    #[test]
    fn test_from_private_key() {
        let key_bytes = [1u8; 32];
        let wallet = CardanoWallet::from_private_key(&key_bytes, MAINNET_NETWORK_ID)
            .expect("Failed to create wallet");
        assert!(wallet.address().starts_with("addr1"));
    }

    #[test]
    fn test_from_private_key_hex() {
        let key_hex = "0101010101010101010101010101010101010101010101010101010101010101";
        let wallet = CardanoWallet::from_private_key_hex(key_hex, MAINNET_NETWORK_ID)
            .expect("Failed to create wallet");
        assert!(wallet.address().starts_with("addr1"));
    }

    #[test]
    fn test_from_private_key_hex_with_prefix() {
        let key_hex = "0x0101010101010101010101010101010101010101010101010101010101010101";
        let wallet = CardanoWallet::from_private_key_hex(key_hex, MAINNET_NETWORK_ID)
            .expect("Failed to create wallet");
        assert!(wallet.address().starts_with("addr1"));
    }

    #[test]
    fn test_from_private_key_deterministic() {
        let key_bytes = [42u8; 32];
        let wallet1 = CardanoWallet::from_private_key(&key_bytes, MAINNET_NETWORK_ID).unwrap();
        let wallet2 = CardanoWallet::from_private_key(&key_bytes, MAINNET_NETWORK_ID).unwrap();
        assert_eq!(wallet1.address(), wallet2.address());
    }

    #[test]
    fn test_from_private_key_invalid_length() {
        let key_bytes = [1u8; 16]; // Too short
        let result = CardanoWallet::from_private_key(&key_bytes, MAINNET_NETWORK_ID);
        assert!(result.is_err());
    }

    // ========================================================================
    // Key Access Tests
    // ========================================================================

    #[test]
    fn test_public_key_format() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let pubkey = wallet.public_key();
        assert_eq!(pubkey.len(), 64); // 32 bytes = 64 hex chars
        assert!(pubkey.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_private_key_format() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let privkey = wallet.private_key();
        assert!(privkey.starts_with("0x"));
        assert_eq!(privkey.len(), 66); // 0x + 64 hex chars
    }

    // ========================================================================
    // Signing Tests
    // ========================================================================

    #[test]
    fn test_sign_message() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let message = b"Hello, Cardano!";
        let signature = wallet.sign(message);
        assert_eq!(signature.len(), 64); // Ed25519 signature is 64 bytes
    }

    #[test]
    fn test_verify_signature() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let message = b"Hello, Cardano!";
        let signature = wallet.sign(message);
        assert!(wallet.verify(message, &signature));
    }

    #[test]
    fn test_verify_wrong_message() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let message = b"Hello, Cardano!";
        let wrong_message = b"Wrong message";
        let signature = wallet.sign(message);
        assert!(!wallet.verify(wrong_message, &signature));
    }

    #[test]
    fn test_verify_invalid_signature() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let message = b"Hello, Cardano!";
        let invalid_sig = vec![0u8; 64];
        assert!(!wallet.verify(message, &invalid_sig));
    }

    #[test]
    fn test_verify_wrong_length_signature() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let message = b"Hello, Cardano!";
        let wrong_sig = vec![0u8; 32]; // Too short
        assert!(!wallet.verify(message, &wrong_sig));
    }

    // ========================================================================
    // Config Tests
    // ========================================================================

    #[test]
    fn test_mainnet_config() {
        let wallet = CardanoWallet::mainnet().unwrap();
        assert_eq!(wallet.config().currency_symbol, "ADA");
        assert_eq!(wallet.config().decimals, 6);
    }

    #[test]
    fn test_testnet_config() {
        let wallet = CardanoWallet::testnet().unwrap();
        assert_eq!(wallet.config().currency_symbol, "tADA");
    }

    // ========================================================================
    // API Key Tests
    // ========================================================================

    #[test]
    fn test_no_api_key_initially() {
        let wallet = CardanoWallet::mainnet().unwrap();
        assert!(!wallet.has_api_key());
    }

    #[test]
    fn test_set_api_key() {
        let mut wallet = CardanoWallet::mainnet().unwrap();
        wallet.set_api_key("test_api_key");
        assert!(wallet.has_api_key());
    }

    // ========================================================================
    // Balance Tests (without API)
    // ========================================================================

    #[tokio::test]
    async fn test_get_balance_no_api() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0);
    }

    #[tokio::test]
    async fn test_get_balance_ada_no_api() {
        let wallet = CardanoWallet::mainnet().unwrap();
        let balance = wallet.get_balance_ada().await.unwrap();
        assert_eq!(balance, 0.0);
    }
}
