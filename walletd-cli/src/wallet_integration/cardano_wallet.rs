//! Cardano Wallet

use super::hd_derivation::{self, paths};
use anyhow::Result;

pub struct CardanoWallet {
    pub address: String,
    pub network: String,
    key_bytes: [u8; 32],
}

impl CardanoWallet {
    pub fn new(network: &str) -> Result<Self> {
        let mut key_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key_bytes);
        let prefix = if network == "mainnet" { "addr1" } else { "addr_test1" };
        let address = format!("{}{}", prefix, hex::encode(&key_bytes[..28]));
        Ok(Self { address, network: network.to_string(), key_bytes })
    }

    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::CARDANO)?;
        let prefix = if network == "mainnet" { "addr1" } else { "addr_test1" };
        let address = format!("{}{}", prefix, hex::encode(&key_bytes[..28]));
        Ok(Self { address, network: network.to_string(), key_bytes })
    }

    pub async fn get_balance(&self) -> Result<u64> { Ok(0) }
    
    pub fn explorer_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => format!("https://cardanoscan.io/address/{}", self.address),
            _ => format!("https://preprod.cardanoscan.io/address/{}", self.address),
        }
    }
}
