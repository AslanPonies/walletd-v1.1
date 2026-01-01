//! EVM Wallet - Generic implementation for EVM-compatible chains
//!
//! Used for: Polygon, Avalanche, Base, Arbitrum

use super::hd_derivation::{self, paths};
use anyhow::Result;
use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
};
use std::sync::Arc;

pub struct EvmWallet {
    wallet: LocalWallet,
    pub address: Address,
    pub chain_id: u64,
    pub chain_name: String,
    provider: Option<Arc<Provider<Http>>>,
}

impl EvmWallet {
    pub fn new(chain_id: u64, chain_name: &str) -> Result<Self> {
        let wallet = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(chain_id);
        let address = wallet.address();
        Ok(Self { 
            wallet, 
            address, 
            chain_id, 
            chain_name: chain_name.to_string(),
            provider: None 
        })
    }

    pub fn from_mnemonic(mnemonic: &str, chain_id: u64, chain_name: &str) -> Result<Self> {
        let key_bytes = hd_derivation::derive_key_bytes(mnemonic, paths::ETHEREUM)?;
        let wallet = LocalWallet::from_bytes(&key_bytes)?.with_chain_id(chain_id);
        let address = wallet.address();
        Ok(Self { 
            wallet, 
            address, 
            chain_id, 
            chain_name: chain_name.to_string(),
            provider: None 
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        let rpc_url = self.default_rpc();
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
            // Try to connect on-demand
            let rpc_url = self.default_rpc();
            if let Ok(provider) = Provider::<Http>::try_from(rpc_url) {
                let balance = provider.get_balance(self.address, None).await?;
                return Ok(balance.as_u64());
            }
            Ok(0)
        }
    }

    pub async fn send(&self, to: &str, amount: f64) -> Result<String> {
        let provider = self.provider.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected"))?;
        let to_addr: Address = to.parse()?;
        let amount_wei = U256::from((amount * 1e18) as u64);

        let tx = TransactionRequest::new()
            .to(to_addr)
            .value(amount_wei)
            .from(self.address);

        let client = SignerMiddleware::new(provider.clone(), self.wallet.clone());
        let pending = client.send_transaction(tx, None).await?;
        Ok(format!("{:?}", pending.tx_hash()))
    }

    pub fn default_rpc(&self) -> &str {
        match self.chain_id {
            // Polygon
            137 => "https://polygon-rpc.com",
            80002 => "https://rpc-amoy.polygon.technology",
            // Avalanche
            43114 => "https://api.avax.network/ext/bc/C/rpc",
            43113 => "https://api.avax-test.network/ext/bc/C/rpc",
            // Base
            8453 => "https://mainnet.base.org",
            84532 => "https://sepolia.base.org",
            // Arbitrum
            42161 => "https://arb1.arbitrum.io/rpc",
            421614 => "https://sepolia-rollup.arbitrum.io/rpc",
            _ => "https://rpc.sepolia.org",
        }
    }

    pub fn explorer_url(&self) -> String {
        match self.chain_id {
            137 => format!("https://polygonscan.com/address/{:?}", self.address),
            80002 => format!("https://amoy.polygonscan.com/address/{:?}", self.address),
            43114 => format!("https://snowtrace.io/address/{:?}", self.address),
            43113 => format!("https://testnet.snowtrace.io/address/{:?}", self.address),
            8453 => format!("https://basescan.org/address/{:?}", self.address),
            84532 => format!("https://sepolia.basescan.org/address/{:?}", self.address),
            42161 => format!("https://arbiscan.io/address/{:?}", self.address),
            421614 => format!("https://sepolia.arbiscan.io/address/{:?}", self.address),
            _ => format!("https://etherscan.io/address/{:?}", self.address),
        }
    }

    pub fn native_symbol(&self) -> &str {
        match self.chain_id {
            137 | 80002 => "POL",
            43114 | 43113 => "AVAX",
            8453 | 84532 => "ETH",
            42161 | 421614 => "ETH",
            _ => "ETH",
        }
    }

    pub fn faucet_url(&self) -> Option<&str> {
        match self.chain_id {
            80002 => Some("https://faucet.polygon.technology"),
            43113 => Some("https://faucet.avax.network"),
            84532 => Some("https://www.coinbase.com/faucets/base-ethereum-goerli-faucet"),
            421614 => Some("https://faucet.arbitrum.io"),
            _ => None,
        }
    }
}
