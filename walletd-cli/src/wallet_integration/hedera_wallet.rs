//! Hedera Wallet

use anyhow::Result;
use ed25519_dalek::SigningKey;

pub struct HederaWallet {
    signing_key: SigningKey,
    pub account_id: Option<String>,
    pub network: String,
}

impl HederaWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        Ok(Self {
            signing_key,
            account_id: None,
            network: network.to_string(),
        })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().as_bytes())
    }

    pub fn set_account_id(&mut self, id: &str) {
        self.account_id = Some(id.to_string());
    }

    pub async fn get_balance(&self) -> Result<u64> {
        Ok(0) // Requires Hedera SDK
    }

    pub fn explorer_url(&self) -> String {
        if let Some(id) = &self.account_id {
            match self.network.as_str() {
                "mainnet" => format!("https://hashscan.io/mainnet/account/{}", id),
                _ => format!("https://hashscan.io/testnet/account/{}", id),
            }
        } else {
            String::new()
        }
    }
}
