//! Ethereum Wallet - Real Implementation

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
};
use std::sync::Arc;

pub struct EthereumWallet {
    wallet: LocalWallet,
    pub address: Address,
    pub chain_id: u64,
    provider: Option<Arc<Provider<Http>>>,
}

impl EthereumWallet {
    pub fn new(chain_id: u64) -> Result<Self> {
        let wallet = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(chain_id);
        let address = wallet.address();
        Ok(Self { wallet, address, chain_id, provider: None })
    }

    pub fn from_mnemonic(mnemonic: &str, chain_id: u64) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::ETHEREUM)?;
        let wallet = LocalWallet::from_bytes(&key_bytes)?.with_chain_id(chain_id);
        let address = wallet.address();
        Ok(Self { wallet, address, chain_id, provider: None })
    }

    pub fn from_private_key(key_hex: &str, chain_id: u64) -> Result<Self> {
        let wallet: LocalWallet = key_hex.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let address = wallet.address();
        Ok(Self { wallet, address, chain_id, provider: None })
    }

    pub async fn connect(&mut self, rpc_url: &str) -> Result<()> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        self.provider = Some(Arc::new(provider));
        Ok(())
    }

    pub fn address_string(&self) -> String {
        format!("{:?}", self.address)
    }

    pub async fn get_balance(&self) -> Result<u64> {
        if let Some(provider) = &self.provider {
            let balance = provider.get_balance(self.address, None).await?;
            Ok(balance.as_u64())
        } else {
            Ok(0)
        }
    }

    pub async fn send(&self, to: &str, amount_eth: f64) -> Result<String> {
        let provider = self.provider.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected"))?;
        let to_addr: Address = to.parse()?;
        let amount = U256::from((amount_eth * 1e18) as u64);

        let tx = TransactionRequest::new()
            .to(to_addr)
            .value(amount)
            .from(self.address);

        let client = SignerMiddleware::new(provider.clone(), self.wallet.clone());
        let pending = client.send_transaction(tx, None).await?;
        Ok(format!("{:?}", pending.tx_hash()))
    }

    pub fn get_private_key(&self) -> String {
        hex::encode(self.wallet.signer().to_bytes())
    }

    pub fn rpc_url(&self) -> &str {
        match self.chain_id {
            1 => "https://eth.llamarpc.com",
            11155111 => "https://rpc.sepolia.org",
            _ => "https://rpc.sepolia.org",
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.chain_id {
            1 => format!("https://etherscan.io/address/{:?}", self.address),
            11155111 => format!("https://sepolia.etherscan.io/address/{:?}", self.address),
            _ => format!("https://etherscan.io/address/{:?}", self.address),
        }
    }
}
