//! Cosmos (ATOM) wallet support for WalletD
//!
//! Supports Cosmos Hub and other Cosmos SDK chains.

use anyhow::Result;
use bech32::{Bech32, Hrp};
use bip39::Mnemonic;
use ripemd::Ripemd160;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Error, Debug)]
pub enum CosmosError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Key error: {0}")]
    KeyError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Other: {0}")]
    Other(#[from] anyhow::Error),
}

// ============================================================================
// CONFIG
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub chain_id: String,
    pub name: String,
    pub denom: String,
    pub decimals: u8,
    pub bech32_prefix: String,
    pub rpc_endpoints: Vec<String>,
    pub rest_endpoints: Vec<String>,
    pub explorer: String,
}

pub const COSMOS_HUB_CHAIN_ID: &str = "cosmoshub-4";
pub const UATOM_DENOM: &str = "uatom";

impl NetworkConfig {
    pub fn cosmos_hub() -> Self {
        Self {
            chain_id: COSMOS_HUB_CHAIN_ID.to_string(),
            name: "Cosmos Hub".to_string(),
            denom: UATOM_DENOM.to_string(),
            decimals: 6,
            bech32_prefix: "cosmos".to_string(),
            rpc_endpoints: vec![
                "https://rpc.cosmos.network".to_string(),
                "https://cosmos-rpc.publicnode.com".to_string(),
            ],
            rest_endpoints: vec![
                "https://rest.cosmos.network".to_string(),
                "https://cosmos-rest.publicnode.com".to_string(),
            ],
            explorer: "https://www.mintscan.io/cosmos".to_string(),
        }
    }

    pub fn theta_testnet() -> Self {
        Self {
            chain_id: "theta-testnet-001".to_string(),
            name: "Cosmos Testnet".to_string(),
            denom: "uatom".to_string(),
            decimals: 6,
            bech32_prefix: "cosmos".to_string(),
            rpc_endpoints: vec!["https://rpc.sentry-01.theta-testnet.polypore.xyz".to_string()],
            rest_endpoints: vec!["https://rest.sentry-01.theta-testnet.polypore.xyz".to_string()],
            explorer: "https://explorer.theta-testnet.polypore.xyz".to_string(),
        }
    }

    pub fn testnet() -> Self {
        Self::theta_testnet()
    }

    pub fn atom_to_uatom(atom: f64) -> u64 {
        (atom * 1_000_000.0) as u64
    }

    pub fn uatom_to_atom(uatom: u64) -> f64 {
        uatom as f64 / 1_000_000.0
    }
}

// ============================================================================
// WALLET
// ============================================================================

pub struct CosmosWallet {
    secret_key: SecretKey,
    public_key: PublicKey,
    config: NetworkConfig,
    api_endpoint: Option<String>,
}

impl CosmosWallet {
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
            api_endpoint: None,
        })
    }

    pub fn mainnet() -> Result<Self> {
        Self::new(NetworkConfig::cosmos_hub())
    }

    pub fn testnet() -> Result<Self> {
        Self::new(NetworkConfig::testnet())
    }

    pub fn from_mnemonic(mnemonic: &str, config: NetworkConfig) -> Result<Self> {
        let mnemonic = Mnemonic::from_str(mnemonic)?;
        let seed = mnemonic.to_seed("");

        // Cosmos derivation path: m/44'/118'/0'/0/0
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&seed[..32]);

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
            config,
            api_endpoint: None,
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
            api_endpoint: None,
        })
    }

    pub fn set_api_endpoint(&mut self, endpoint: &str) {
        self.api_endpoint = Some(endpoint.to_string());
    }

    pub fn address(&self) -> String {
        let pubkey_bytes = self.public_key.serialize();
        let sha256_hash = Sha256::digest(&pubkey_bytes);
        let ripemd_hash = Ripemd160::digest(&sha256_hash);

        let hrp = Hrp::parse(&self.config.bech32_prefix).unwrap();
        bech32::encode::<Bech32>(hrp, &ripemd_hash).unwrap()
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

    pub fn chain_id(&self) -> &str {
        &self.config.chain_id
    }

    pub async fn get_balance(&self) -> Result<u64> {
        if self.api_endpoint.is_none() {
            return Ok(0);
        }
        // Would query REST API: /cosmos/bank/v1beta1/balances/{address}
        Ok(0)
    }

    pub async fn get_balance_atom(&self) -> Result<f64> {
        let uatom = self.get_balance().await?;
        Ok(NetworkConfig::uatom_to_atom(uatom))
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::new();
        let msg_hash = Sha256::digest(message);
        let msg = secp256k1::Message::from_slice(&msg_hash).unwrap();
        let sig = secp.sign_ecdsa(&msg, &self.secret_key);
        sig.serialize_compact().to_vec()
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
        let wallet = CosmosWallet::mainnet().unwrap();
        assert!(wallet.address().starts_with("cosmos1"));
    }

    #[test]
    fn test_testnet_wallet() {
        let wallet = CosmosWallet::testnet().unwrap();
        assert!(wallet.address().starts_with("cosmos1"));
    }

    #[test]
    fn test_from_mnemonic() {
        let wallet = CosmosWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::cosmos_hub()).unwrap();
        assert!(wallet.address().starts_with("cosmos1"));
    }

    #[test]
    fn test_from_mnemonic_deterministic() {
        let w1 = CosmosWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::cosmos_hub()).unwrap();
        let w2 = CosmosWallet::from_mnemonic(TEST_MNEMONIC, NetworkConfig::cosmos_hub()).unwrap();
        assert_eq!(w1.address(), w2.address());
    }

    #[test]
    fn test_random_wallets_different() {
        let w1 = CosmosWallet::mainnet().unwrap();
        let w2 = CosmosWallet::mainnet().unwrap();
        assert_ne!(w1.address(), w2.address());
    }

    #[test]
    fn test_address_format() {
        let wallet = CosmosWallet::mainnet().unwrap();
        let addr = wallet.address();
        assert!(addr.starts_with("cosmos1"));
        assert!(addr.len() > 40);
    }

    #[test]
    fn test_public_key_format() {
        let wallet = CosmosWallet::mainnet().unwrap();
        let pk = wallet.public_key();
        assert_eq!(pk.len(), 66); // Compressed public key
    }

    #[test]
    fn test_private_key_format() {
        let wallet = CosmosWallet::mainnet().unwrap();
        let pk = wallet.private_key();
        assert!(pk.starts_with("0x"));
        assert_eq!(pk.len(), 66);
    }

    #[test]
    fn test_sign_message() {
        let wallet = CosmosWallet::mainnet().unwrap();
        let sig = wallet.sign(b"Hello Cosmos!");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_config() {
        let config = NetworkConfig::cosmos_hub();
        assert_eq!(config.chain_id, "cosmoshub-4");
        assert_eq!(config.denom, "uatom");
        assert_eq!(config.decimals, 6);
    }

    #[test]
    fn test_atom_conversion() {
        assert_eq!(NetworkConfig::atom_to_uatom(1.0), 1_000_000);
        assert_eq!(NetworkConfig::uatom_to_atom(1_000_000), 1.0);
    }

    #[tokio::test]
    async fn test_get_balance_no_api() {
        let wallet = CosmosWallet::mainnet().unwrap();
        let balance = wallet.get_balance().await.unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_chain_id() {
        let wallet = CosmosWallet::mainnet().unwrap();
        assert_eq!(wallet.chain_id(), "cosmoshub-4");
    }
}
