//! Monero Wallet

use anyhow::Result;

pub struct MoneroWallet {
    pub address: String,
    pub network: String,
    spend_key: [u8; 32],
    view_key: [u8; 32],
}

impl MoneroWallet {
    pub fn new(network: &str) -> Result<Self> {
        let mut spend_key = [0u8; 32];
        let mut view_key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut spend_key);
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut view_key);
        
        let prefix = match network {
            "mainnet" => "4",
            _ => "5",
        };
        
        let address = format!("{}{}", prefix, hex::encode(&spend_key[..43]));

        Ok(Self {
            address,
            network: network.to_string(),
            spend_key,
            view_key,
        })
    }

    pub fn get_view_key(&self) -> String { hex::encode(&self.view_key) }
    pub fn get_spend_key(&self) -> String { hex::encode(&self.spend_key) }
}
