use bip39::Mnemonic;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use std::str::FromStr;

use crate::types::{Error, Result};

#[derive(Debug, Clone)]
pub struct PrasagaAvioKeypair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    derivation_path: Option<String>,
}

impl PrasagaAvioKeypair {
    /// Create keypair from seed phrase
    pub fn from_mnemonic(mnemonic: &str, passphrase: &str, path: &str) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)
            .map_err(|e| Error::Crypto(format!("Invalid mnemonic: {e}")))?;

        // Convert mnemonic to seed (64 bytes)
        let seed = mnemonic.to_seed(passphrase);
        Self::from_seed(&seed, path)
    }

    /// Create keypair from seed bytes
    pub fn from_seed(seed: &[u8], path: &str) -> Result<Self> {
        // For now, we'll use a simple derivation
        // In production, this should use proper BIP32 derivation
        let key_material = if seed.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(seed);
            bytes
        } else if seed.len() >= 32 {
            // Take first 32 bytes for ed25519
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&seed[..32]);
            bytes
        } else {
            // Hash the seed to get 32 bytes
            let hash = blake3::hash(seed);
            *hash.as_bytes()
        };

        let signing_key = SigningKey::from_bytes(&key_material);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
            derivation_path: Some(path.to_string()),
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    /// Export private key (be careful!)
    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    /// Get the derivation path
    pub fn derivation_path(&self) -> Option<&str> {
        self.derivation_path.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Keypair Generation Tests
    // ============================================================================

    #[test]
    fn test_keypair_generation() {
        let seed = b"test seed for prasaga avio chain integration!!!";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_keypair_from_32_byte_seed() {
        let seed = [0u8; 32];
        let keypair = PrasagaAvioKeypair::from_seed(&seed, "m/44'/9000'/0'/0/0").unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
        assert_eq!(keypair.private_key_bytes().len(), 32);
    }

    #[test]
    fn test_keypair_from_64_byte_seed() {
        let seed = [1u8; 64];
        let keypair = PrasagaAvioKeypair::from_seed(&seed, "m/44'/9000'/0'/0/0").unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_keypair_from_short_seed() {
        let seed = b"short"; // Less than 32 bytes
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_deterministic_keys() {
        let seed = b"deterministic test seed";
        let keypair1 = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        let keypair2 = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
        assert_eq!(keypair1.private_key_bytes(), keypair2.private_key_bytes());
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let keypair1 = PrasagaAvioKeypair::from_seed(b"seed one", "m/44'/9000'/0'/0/0").unwrap();
        let keypair2 = PrasagaAvioKeypair::from_seed(b"seed two", "m/44'/9000'/0'/0/0").unwrap();
        
        assert_ne!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }

    // ============================================================================
    // Signature Tests
    // ============================================================================

    #[test]
    fn test_signature() {
        let seed = b"test seed for prasaga avio chain integration!!!";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        let message = b"Hello PraSaga!";
        let signature = keypair.sign(message);
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_signature_deterministic() {
        let seed = b"signature test seed";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        let message = b"Test message";
        
        let sig1 = keypair.sign(message);
        let sig2 = keypair.sign(message);
        
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_different_messages_different_signatures() {
        let seed = b"signature test seed";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        
        let sig1 = keypair.sign(b"Message 1");
        let sig2 = keypair.sign(b"Message 2");
        
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_sign_empty_message() {
        let seed = b"empty message test";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        let signature = keypair.sign(b"");
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_sign_large_message() {
        let seed = b"large message test";
        let keypair = PrasagaAvioKeypair::from_seed(seed, "m/44'/9000'/0'/0/0").unwrap();
        let large_message = vec![0u8; 10000];
        let signature = keypair.sign(&large_message);
        assert_eq!(signature.len(), 64);
    }

    // ============================================================================
    // Mnemonic Tests
    // ============================================================================

    #[test]
    fn test_from_mnemonic() {
        let mnemonic = "test test test test test test test test test test test junk";
        let keypair =
            PrasagaAvioKeypair::from_mnemonic(mnemonic, "", "m/44'/9000'/0'/0/0").unwrap();
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_from_mnemonic_with_passphrase() {
        let mnemonic = "test test test test test test test test test test test junk";
        let keypair1 = PrasagaAvioKeypair::from_mnemonic(mnemonic, "", "m/44'/9000'/0'/0/0").unwrap();
        let keypair2 = PrasagaAvioKeypair::from_mnemonic(mnemonic, "password", "m/44'/9000'/0'/0/0").unwrap();
        
        // Different passphrase should produce different keys
        assert_ne!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let keypair1 = PrasagaAvioKeypair::from_mnemonic(mnemonic, "", "m/44'/9000'/0'/0/0").unwrap();
        let keypair2 = PrasagaAvioKeypair::from_mnemonic(mnemonic, "", "m/44'/9000'/0'/0/0").unwrap();
        
        assert_eq!(keypair1.public_key_bytes(), keypair2.public_key_bytes());
    }

    #[test]
    fn test_invalid_mnemonic() {
        let result = PrasagaAvioKeypair::from_mnemonic("invalid mnemonic", "", "m/44'/9000'/0'/0/0");
        assert!(result.is_err());
    }

    // ============================================================================
    // Derivation Path Tests
    // ============================================================================

    #[test]
    fn test_derivation_path_stored() {
        let path = "m/44'/9000'/0'/0/0";
        let keypair = PrasagaAvioKeypair::from_seed(b"test", path).unwrap();
        assert_eq!(keypair.derivation_path(), Some(path));
    }

    #[test]
    fn test_different_path_format() {
        let path1 = "m/44'/9000'/0'/0/0";
        let path2 = "m/44'/9000'/0'/0/1";
        
        let keypair1 = PrasagaAvioKeypair::from_seed(b"test", path1).unwrap();
        let keypair2 = PrasagaAvioKeypair::from_seed(b"test", path2).unwrap();
        
        assert_eq!(keypair1.derivation_path(), Some(path1));
        assert_eq!(keypair2.derivation_path(), Some(path2));
    }

    // ============================================================================
    // Key Export Tests
    // ============================================================================

    #[test]
    fn test_public_key_export() {
        let keypair = PrasagaAvioKeypair::from_seed(b"export test", "m/44'/9000'/0'/0/0").unwrap();
        let pub_key = keypair.public_key_bytes();
        
        assert_eq!(pub_key.len(), 32);
        // Ed25519 public keys should have some non-zero bytes
        assert!(pub_key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_private_key_export() {
        let keypair = PrasagaAvioKeypair::from_seed(b"export test", "m/44'/9000'/0'/0/0").unwrap();
        let priv_key = keypair.private_key_bytes();
        
        assert_eq!(priv_key.len(), 32);
    }

    #[test]
    fn test_public_private_key_different() {
        let keypair = PrasagaAvioKeypair::from_seed(b"key test", "m/44'/9000'/0'/0/0").unwrap();
        
        assert_ne!(keypair.public_key_bytes(), keypair.private_key_bytes());
    }
}
