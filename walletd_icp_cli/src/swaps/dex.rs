//! DEX integration module using alloy
//! 
//! Note: This module provides DEX aggregation functionality.
//! UniswapV3 and 1inch integrations are provided as reference implementations.

use anyhow::Result;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolCall;
use serde::Deserialize;
use std::str::FromStr;

// Uniswap V3 Quoter interface
sol! {
    function quoteExactInputSingle(
        address tokenIn,
        address tokenOut,
        uint24 fee,
        uint256 amountIn,
        uint160 sqrtPriceLimitX96
    ) external returns (uint256 amountOut);
}

pub struct UniswapV3 {
    router_address: Address,
    quoter_address: Address,
    rpc_url: String,
}

impl UniswapV3 {
    pub fn new(provider_url: &str) -> Result<Self> {
        let router_address = Address::from_str("0xE592427A0AEce92De3Edee1F18E0157C05861564")?; // Uniswap V3 Router
        let quoter_address = Address::from_str("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6")?; // Quoter
        
        Ok(Self {
            router_address,
            quoter_address,
            rpc_url: provider_url.to_string(),
        })
    }
    
    pub async fn get_quote(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        fee: u32,
    ) -> Result<U256> {
        let provider = ProviderBuilder::new()
            .connect_http(self.rpc_url.parse()?);
        
        // Encode quoter call
        let call = quoteExactInputSingleCall {
            tokenIn: token_in,
            tokenOut: token_out,
            fee: fee.try_into().unwrap_or(3000),
            amountIn: amount_in,
            sqrtPriceLimitX96: U256::ZERO.try_into().unwrap_or_default(),
        };
        
        let call_data = call.abi_encode();
        
        let tx = TransactionRequest::default()
            .to(self.quoter_address)
            .input(call_data.into());
            
        let result = provider.call(tx).await?;
        let amount_out = quoteExactInputSingleCall::abi_decode_returns(&result)?;
        
        Ok(amount_out)
    }
}

// 1inch DEX Aggregator
pub struct OneInch {
    api_key: String,
    base_url: String,
    chain_id: u32,
}

impl OneInch {
    pub fn new(api_key: String, chain_id: u32) -> Self {
        Self {
            api_key,
            base_url: "https://api.1inch.io/v5.0".to_string(),
            chain_id,
        }
    }
    
    pub async fn get_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
    ) -> Result<OneInchQuote> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/{}/quote?fromTokenAddress={}&toTokenAddress={}&amount={}",
            self.base_url, self.chain_id, from_token, to_token, amount
        );
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
            
        let quote: OneInchQuote = response.json().await?;
        Ok(quote)
    }
    
    pub async fn build_swap_tx(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        from_address: &str,
        slippage: f64,
    ) -> Result<OneInchSwapTx> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/{}/swap?fromTokenAddress={}&toTokenAddress={}&amount={}&fromAddress={}&slippage={}",
            self.base_url, self.chain_id, from_token, to_token, amount, from_address, slippage
        );
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
            
        let swap_tx: OneInchSwapTx = response.json().await?;
        Ok(swap_tx)
    }
}

#[derive(Debug, Deserialize)]
pub struct OneInchQuote {
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub to_token_amount: String,
    pub from_token_amount: String,
    pub protocols: Vec<Vec<ProtocolInfo>>,
    pub estimated_gas: u64,
}

#[derive(Debug, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub address: String,
    pub logo_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct ProtocolInfo {
    pub name: String,
    pub part: f64,
    pub from_token_address: String,
    pub to_token_address: String,
}

#[derive(Debug, Deserialize)]
pub struct OneInchSwapTx {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas: u64,
    pub gas_price: String,
}
