//! HD Wallet Derivation
//!
//! BIP-39/44/84 compliant hierarchical deterministic key derivation.

use anyhow::Result;
use bip39::{Mnemonic, Language};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::secp256k1::Secp256k1;

/// Standard derivation paths for various chains
pub mod paths {
    /// BIP-84 Bitcoin Native SegWit: m/84'/0'/0'/0/0
    pub const BITCOIN: &str = "m/84'/0'/0'/0/0";
    /// BIP-44 Ethereum: m/44'/60'/0'/0/0
    pub const ETHEREUM: &str = "m/44'/60'/0'/0/0";
    /// BIP-44 Solana: m/44'/501'/0'/0'
    pub const SOLANA: &str = "m/44'/501'/0'/0'";
    /// BIP-44 Cosmos: m/44'/118'/0'/0/0
    pub const COSMOS: &str = "m/44'/118'/0'/0/0";
    /// BIP-44 Cardano (Shelley): m/1852'/1815'/0'/0/0
    pub const CARDANO: &str = "m/1852'/1815'/0'/0/0";
    /// BIP-44 Polkadot: m/44'/354'/0'/0'/0'
    pub const POLKADOT: &str = "m/44'/354'/0'/0'/0'";
    /// BIP-44 Near: m/44'/397'/0'
    pub const NEAR: &str = "m/44'/397'/0'";
    /// BIP-44 Tron: m/44'/195'/0'/0/0
    pub const TRON: &str = "m/44'/195'/0'/0/0";
    /// BIP-44 Sui: m/44'/784'/0'/0'/0'
    pub const SUI: &str = "m/44'/784'/0'/0'/0'";
    /// BIP-44 Aptos: m/44'/637'/0'/0'/0'
    pub const APTOS: &str = "m/44'/637'/0'/0'/0'";
    /// TON (non-standard): m/44'/607'/0'
    pub const TON: &str = "m/44'/607'/0'";
}

/// Generate a new BIP-39 mnemonic
pub fn generate_mnemonic(word_count: usize) -> Result<String> {
    let mnemonic = match word_count {
        12 => Mnemonic::generate_in(Language::English, 12)?,
        15 => Mnemonic::generate_in(Language::English, 15)?,
        18 => Mnemonic::generate_in(Language::English, 18)?,
        21 => Mnemonic::generate_in(Language::English, 21)?,
        24 => Mnemonic::generate_in(Language::English, 24)?,
        _ => return Err(anyhow::anyhow!("Invalid word count. Use 12, 15, 18, 21, or 24")),
    };
    Ok(mnemonic.to_string())
}

/// Validate a mnemonic phrase
pub fn validate_mnemonic(phrase: &str) -> Result<()> {
    Mnemonic::parse_in(Language::English, phrase)?;
    Ok(())
}

/// Convert mnemonic to seed bytes
pub fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> Result<[u8; 64]> {
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
    let seed = mnemonic.to_seed(passphrase);
    Ok(seed)
}

/// Derive Bitcoin extended private key from mnemonic
pub fn derive_bitcoin_xpriv(mnemonic: &str, network: bitcoin::Network) -> Result<Xpriv> {
    let seed = mnemonic_to_seed(mnemonic, "")?;
    let xpriv = Xpriv::new_master(network, &seed)?;
    Ok(xpriv)
}

/// Derive key at a specific path
pub fn derive_at_path(xpriv: &Xpriv, path: &str) -> Result<Xpriv> {
    let secp = Secp256k1::new();
    let path: DerivationPath = path.parse()?;
    let derived = xpriv.derive_priv(&secp, &path)?;
    Ok(derived)
}

/// Derive raw 32-byte key for non-Bitcoin chains
pub fn derive_key_bytes(mnemonic: &str, path: &str) -> Result<[u8; 32]> {
    let seed = mnemonic_to_seed(mnemonic, "")?;
    let xpriv = Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)?;
    let derived = derive_at_path(&xpriv, path)?;
    Ok(derived.private_key.secret_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_validate_mnemonic() {
        let valid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(validate_mnemonic(valid).is_ok());
        
        let invalid = "invalid mnemonic phrase";
        assert!(validate_mnemonic(invalid).is_err());
    }

    #[test]
    fn test_derive_key() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let key = derive_key_bytes(mnemonic, paths::ETHEREUM).unwrap();
        assert_eq!(key.len(), 32);
    }
}
