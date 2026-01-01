//! Polkadot Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;

pub struct PolkadotWallet {
    pub address: String,
    pub network: String,
    key_bytes: [u8; 32],
}

impl PolkadotWallet {
    pub fn new(network: &str) -> Result<Self> {
        let mut key_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key_bytes);
        let address = Self::encode_address(&key_bytes, network);
        Ok(Self { address, network: network.to_string(), key_bytes })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::POLKADOT)?;
        let address = Self::encode_address(&key_bytes, network);
        Ok(Self { address, network: network.to_string(), key_bytes })
    }

    fn encode_address(key_bytes: &[u8; 32], network: &str) -> String {
        // SS58 encoding - simplified
        let prefix = match network {
            "polkadot" => 0u8,
            "kusama" => 2u8,
            "westend" => 42u8,
            _ => 42u8,
        };
        let mut data = vec![prefix];
        data.extend_from_slice(key_bytes);
        bs58::encode(&data).into_string()
    }

    pub async fn get_balance(&self) -> Result<u64> { Ok(0) }

    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "polkadot" => format!("https://polkadot.subscan.io/account/{}", self.address),
            "kusama" => format!("https://kusama.subscan.io/account/{}", self.address),
            _ => format!("https://westend.subscan.io/account/{}", self.address),
        }
    }
}
