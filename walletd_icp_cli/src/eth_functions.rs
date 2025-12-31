use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::network::TransactionBuilder;
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolCall;
use anyhow::Result;
use std::str::FromStr;

// ERC20 function selectors
sol! {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

pub struct EthereumTransactionManager {
    rpc_url: String,
}

impl EthereumTransactionManager {
    pub fn new(_private_key: &str, provider_url: &str) -> Result<Self> {
        Ok(Self { 
            rpc_url: provider_url.to_string(),
        })
    }
    
    pub async fn get_balance(&self, address: &str) -> Result<U256> {
        let addr = Address::from_str(address)?;
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let balance = provider.get_balance(addr).await?;
        Ok(balance)
    }
    
    pub async fn get_token_balance(
        &self,
        token_address: &str,
        wallet_address: &str,
    ) -> Result<U256> {
        let token = Address::from_str(token_address)?;
        let wallet = Address::from_str(wallet_address)?;
        
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        
        // Encode balanceOf call
        let call = balanceOfCall { account: wallet };
        let call_data = call.abi_encode();
        
        let tx = TransactionRequest::default()
            .to(token)
            .input(call_data.into());
            
        let result = provider.call(tx).await?;
        let balance = balanceOfCall::abi_decode_returns(&result)?;
        
        Ok(balance)
    }
    
    pub async fn get_gas_price(&self) -> Result<U256> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        let gas_price = provider.get_gas_price().await?;
        Ok(U256::from(gas_price))
    }
}

// Token lists
pub fn get_common_tokens(chain_id: u32) -> Vec<TokenInfo> {
    match chain_id {
        1 => vec![
            TokenInfo {
                address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
            },
            TokenInfo {
                address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                symbol: "USDT".to_string(),
                decimals: 6,
            },
            TokenInfo {
                address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
                symbol: "DAI".to_string(),
                decimals: 18,
            },
        ],
        _ => vec![],
    }
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
}
