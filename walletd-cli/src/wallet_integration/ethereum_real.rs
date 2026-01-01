//! Real Ethereum Wallet Integration
//!
//! Provides actual Ethereum wallet operations using ethers-rs.

use anyhow::Result;
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
};
use std::sync::Arc;

/// Real Ethereum wallet with blockchain integration
pub struct RealEthereumWallet {
    pub wallet: LocalWallet,
    pub address: Address,
    pub chain_id: u64,
    provider: Option<Arc<Provider<Http>>>,
}

impl RealEthereumWallet {
    /// Create a new Ethereum wallet with random key
    pub fn new(chain_id: u64) -> Result<Self> {
        let wallet = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(chain_id);
        let address = wallet.address();

        Ok(Self {
            wallet,
            address,
            chain_id,
            provider: None,
        })
    }

    /// Import wallet from private key hex
    pub fn from_private_key(key_hex: &str, chain_id: u64) -> Result<Self> {
        let wallet: LocalWallet = key_hex.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let address = wallet.address();

        Ok(Self {
            wallet,
            address,
            chain_id,
            provider: None,
        })
    }

    /// Connect to an RPC provider
    pub async fn connect(&mut self, rpc_url: &str) -> Result<()> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        self.provider = Some(Arc::new(provider));
        Ok(())
    }

    /// Get address as string
    pub fn address_string(&self) -> String {
        format!("{:?}", self.address)
    }

    /// Get balance in wei
    pub async fn get_balance(&self) -> Result<u64> {
        if let Some(provider) = &self.provider {
            let balance = provider.get_balance(self.address, None).await?;
            // Convert to u64 (will overflow for very large balances)
            Ok(balance.as_u64())
        } else {
            Ok(0)
        }
    }

    /// Get balance formatted as ETH
    pub async fn get_balance_eth(&self) -> Result<f64> {
        let wei = self.get_balance().await?;
        Ok(wei as f64 / 1e18)
    }

    /// Send ETH transaction
    pub async fn send_transaction(&self, to_address: &str, amount_eth: f64) -> Result<String> {
        let provider = self.provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to provider"))?;

        let to: Address = to_address.parse()?;
        let amount_wei = U256::from((amount_eth * 1e18) as u64);

        let tx = TransactionRequest::new()
            .to(to)
            .value(amount_wei)
            .from(self.address);

        let client = SignerMiddleware::new(provider.clone(), self.wallet.clone());
        let pending_tx = client.send_transaction(tx, None).await?;
        
        let tx_hash = pending_tx.tx_hash();
        Ok(format!("{:?}", tx_hash))
    }

    /// Get private key as hex
    pub fn get_private_key(&self) -> String {
        // This is a simplified version - in production use proper serialization
        format!("0x{}", hex::encode(self.wallet.signer().to_bytes()))
    }

    /// Get explorer URL
    pub fn explorer_url(&self) -> String {
        match self.chain_id {
            1 => format!("https://etherscan.io/address/{:?}", self.address),
            11155111 => format!("https://sepolia.etherscan.io/address/{:?}", self.address),
            137 => format!("https://polygonscan.com/address/{:?}", self.address),
            43114 => format!("https://snowtrace.io/address/{:?}", self.address),
            8453 => format!("https://basescan.org/address/{:?}", self.address),
            42161 => format!("https://arbiscan.io/address/{:?}", self.address),
            _ => String::new(),
        }
    }

    /// Get tx explorer URL
    pub fn tx_explorer_url(&self, tx_hash: &str) -> String {
        match self.chain_id {
            1 => format!("https://etherscan.io/tx/{}", tx_hash),
            11155111 => format!("https://sepolia.etherscan.io/tx/{}", tx_hash),
            137 => format!("https://polygonscan.com/tx/{}", tx_hash),
            43114 => format!("https://snowtrace.io/tx/{}", tx_hash),
            8453 => format!("https://basescan.org/tx/{}", tx_hash),
            42161 => format!("https://arbiscan.io/tx/{}", tx_hash),
            _ => String::new(),
        }
    }
}
