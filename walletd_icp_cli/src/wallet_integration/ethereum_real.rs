use anyhow::Result;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::network::TransactionBuilder;
use alloy::rpc::types::TransactionRequest;
use std::str::FromStr;

pub struct RealEthereumWallet {
    pub signer: PrivateKeySigner,
    pub address: Address,
    pub chain_id: u64,
    rpc_url: Option<String>,
}

impl RealEthereumWallet {
    pub fn new(chain_id: u64) -> Result<Self> {
        let signer = PrivateKeySigner::random();
        let address = signer.address();

        Ok(Self {
            signer,
            address,
            chain_id,
            rpc_url: None,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        let rpc_url = match self.chain_id {
            11155111 => "https://rpc.sepolia.org", // Sepolia public RPC
            1 => "https://eth.llamarpc.com",       // Mainnet public RPC
            _ => return Err(anyhow::anyhow!("Unsupported chain")),
        };

        self.rpc_url = Some(rpc_url.to_string());
        Ok(())
    }

    pub async fn get_balance(&self) -> Result<U256> {
        if let Some(rpc_url) = &self.rpc_url {
            let provider = ProviderBuilder::new()
                .connect_http(rpc_url.parse()?);
            match provider.get_balance(self.address).await {
                Ok(balance) => Ok(balance),
                Err(_) => Ok(U256::ZERO), // Return 0 if error
            }
        } else {
            Ok(U256::ZERO)
        }
    }

    pub async fn send_transaction(&self, to: &str, amount_eth: f64) -> Result<String> {
        if let Some(rpc_url) = &self.rpc_url {
            let to_address = Address::from_str(to)?;
            
            // Convert ETH to wei (1 ETH = 10^18 wei)
            let amount_wei = U256::from((amount_eth * 1e18) as u128);

            // Create provider with signer
            let provider = ProviderBuilder::new()
                .wallet(alloy::network::EthereumWallet::from(self.signer.clone()))
                .connect_http(rpc_url.parse()?);

            // Create the transaction
            let tx = TransactionRequest::default()
                .with_to(to_address)
                .with_value(amount_wei)
                .with_chain_id(self.chain_id);

            // Send the transaction
            println!("📡 Signing transaction...");
            let pending_tx = provider.send_transaction(tx).await?;

            println!("📡 Broadcasting to network...");
            let receipt = pending_tx.get_receipt().await?;

            Ok(format!("{:#x}", receipt.transaction_hash))
        } else {
            Err(anyhow::anyhow!("Not connected to network"))
        }
    }

    pub fn get_private_key(&self) -> String {
        format!("0x{}", hex::encode(self.signer.to_bytes()))
    }
}
