//! ICP Wallet

use anyhow::Result;
use ed25519_dalek::SigningKey;

pub struct IcpWallet {
    signing_key: SigningKey,
    principal: String,
    pub network: String,
}

impl IcpWallet {
    pub fn new(network: &str) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let pk_bytes = signing_key.verifying_key().as_bytes();
        let principal = format!("{}-{}-cai", hex::encode(&pk_bytes[..5]), hex::encode(&pk_bytes[5..10]));
        Ok(Self { signing_key, principal, network: network.to_string() })
    }

    pub fn principal_id(&self) -> String { self.principal.clone() }
    pub fn public_key_hex(&self) -> String { hex::encode(self.signing_key.verifying_key().as_bytes()) }
    pub async fn get_balance(&self) -> Result<u64> { Ok(0) }
}
