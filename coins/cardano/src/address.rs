use anyhow::Result;
use blake2::{Blake2b, Digest};
use blake2::digest::consts::U28;
use bech32::{Bech32, Hrp};

use crate::config::{AddressType, MAINNET_NETWORK_ID, TESTNET_NETWORK_ID};

/// Cardano address
#[derive(Debug, Clone)]
pub struct CardanoAddress {
    pub address_type: AddressType,
    pub network_id: u8,
    pub payment_key_hash: [u8; 28],
    pub staking_key_hash: Option<[u8; 28]>,
    pub bech32: String,
}

impl CardanoAddress {
    /// Create a new enterprise address (no staking)
    pub fn enterprise(payment_pubkey: &[u8], network_id: u8) -> Result<Self> {
        let payment_key_hash = Self::hash_key(payment_pubkey)?;
        let bech32 = Self::encode_enterprise(&payment_key_hash, network_id)?;
        
        Ok(Self {
            address_type: AddressType::Enterprise,
            network_id,
            payment_key_hash,
            staking_key_hash: None,
            bech32,
        })
    }

    /// Create a new base address (payment + staking)
    pub fn base(
        payment_pubkey: &[u8],
        staking_pubkey: &[u8],
        network_id: u8,
    ) -> Result<Self> {
        let payment_key_hash = Self::hash_key(payment_pubkey)?;
        let staking_key_hash = Self::hash_key(staking_pubkey)?;
        let bech32 = Self::encode_base(&payment_key_hash, &staking_key_hash, network_id)?;
        
        Ok(Self {
            address_type: AddressType::Base,
            network_id,
            payment_key_hash,
            staking_key_hash: Some(staking_key_hash),
            bech32,
        })
    }

    /// Hash a public key using Blake2b-224
    pub fn hash_key(pubkey: &[u8]) -> Result<[u8; 28]> {
        let mut hasher = Blake2b::<U28>::new();
        hasher.update(pubkey);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 28];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Encode an enterprise address to bech32
    fn encode_enterprise(payment_hash: &[u8; 28], network_id: u8) -> Result<String> {
        // Enterprise address header: 0110 | network_id (4 bits)
        // 0110 = 6 for enterprise with key hash
        let header = 0x60 | (network_id & 0x0F);
        
        let mut data = Vec::with_capacity(29);
        data.push(header);
        data.extend_from_slice(payment_hash);
        
        let hrp = if network_id == MAINNET_NETWORK_ID {
            Hrp::parse("addr")?
        } else {
            Hrp::parse("addr_test")?
        };
        
        let encoded = bech32::encode::<Bech32>(hrp, &data)?;
        Ok(encoded)
    }

    /// Encode a base address to bech32
    fn encode_base(
        payment_hash: &[u8; 28],
        staking_hash: &[u8; 28],
        network_id: u8,
    ) -> Result<String> {
        // Base address header: 0000 | network_id (4 bits)
        // 0000 = 0 for base address with key hash for both
        let header = 0x00 | (network_id & 0x0F);
        
        let mut data = Vec::with_capacity(57);
        data.push(header);
        data.extend_from_slice(payment_hash);
        data.extend_from_slice(staking_hash);
        
        let hrp = if network_id == MAINNET_NETWORK_ID {
            Hrp::parse("addr")?
        } else {
            Hrp::parse("addr_test")?
        };
        
        let encoded = bech32::encode::<Bech32>(hrp, &data)?;
        Ok(encoded)
    }

    /// Get the bech32 encoded address
    pub fn to_bech32(&self) -> &str {
        &self.bech32
    }

    /// Check if this is a mainnet address
    pub fn is_mainnet(&self) -> bool {
        self.network_id == MAINNET_NETWORK_ID
    }

    /// Validate a Cardano address string
    pub fn validate(address: &str) -> bool {
        // Check prefix
        if !address.starts_with("addr") {
            return false;
        }
        
        // Try to decode
        if let Ok((hrp, _data)) = bech32::decode(address) {
            let hrp_str = hrp.as_str();
            hrp_str == "addr" || hrp_str == "addr_test"
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test key (32 bytes for Ed25519 public key)
    fn test_pubkey() -> [u8; 32] {
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = i as u8;
        }
        key
    }

    #[test]
    fn test_hash_key() {
        let pubkey = test_pubkey();
        let hash = CardanoAddress::hash_key(&pubkey).unwrap();
        assert_eq!(hash.len(), 28);
    }

    #[test]
    fn test_enterprise_address_mainnet() {
        let pubkey = test_pubkey();
        let addr = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID).unwrap();
        
        assert!(addr.bech32.starts_with("addr1"));
        assert_eq!(addr.address_type, AddressType::Enterprise);
        assert!(addr.is_mainnet());
        assert!(addr.staking_key_hash.is_none());
    }

    #[test]
    fn test_enterprise_address_testnet() {
        let pubkey = test_pubkey();
        let addr = CardanoAddress::enterprise(&pubkey, TESTNET_NETWORK_ID).unwrap();
        
        assert!(addr.bech32.starts_with("addr_test1"));
        assert!(!addr.is_mainnet());
    }

    #[test]
    fn test_base_address_mainnet() {
        let payment_key = test_pubkey();
        let mut staking_key = test_pubkey();
        staking_key[0] = 0xFF; // Make different
        
        let addr = CardanoAddress::base(&payment_key, &staking_key, MAINNET_NETWORK_ID).unwrap();
        
        assert!(addr.bech32.starts_with("addr1"));
        assert_eq!(addr.address_type, AddressType::Base);
        assert!(addr.staking_key_hash.is_some());
    }

    #[test]
    fn test_base_address_testnet() {
        let payment_key = test_pubkey();
        let staking_key = test_pubkey();
        
        let addr = CardanoAddress::base(&payment_key, &staking_key, TESTNET_NETWORK_ID).unwrap();
        
        assert!(addr.bech32.starts_with("addr_test1"));
    }

    #[test]
    fn test_address_deterministic() {
        let pubkey = test_pubkey();
        let addr1 = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID).unwrap();
        let addr2 = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID).unwrap();
        
        assert_eq!(addr1.bech32, addr2.bech32);
    }

    #[test]
    fn test_validate_mainnet_address() {
        let pubkey = test_pubkey();
        let addr = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID).unwrap();
        assert!(CardanoAddress::validate(&addr.bech32));
    }

    #[test]
    fn test_validate_testnet_address() {
        let pubkey = test_pubkey();
        let addr = CardanoAddress::enterprise(&pubkey, TESTNET_NETWORK_ID).unwrap();
        assert!(CardanoAddress::validate(&addr.bech32));
    }

    #[test]
    fn test_validate_invalid_address() {
        assert!(!CardanoAddress::validate("invalid"));
        assert!(!CardanoAddress::validate("btc1qxyz"));
        assert!(!CardanoAddress::validate("0x1234567890"));
    }

    #[test]
    fn test_to_bech32() {
        let pubkey = test_pubkey();
        let addr = CardanoAddress::enterprise(&pubkey, MAINNET_NETWORK_ID).unwrap();
        assert_eq!(addr.to_bech32(), &addr.bech32);
    }
}
