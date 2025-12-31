use crate::Error;
use crate::EthereumAmount;

use alloy::primitives::{Address, B256, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Block, BlockId, BlockNumberOrTag, Transaction};
use alloy::sol;

/// A blockchain connector for Ethereum using Alloy.
pub struct EthClient {}

// Creates Rust bindings for the ERC20 ABI using alloy's sol! macro
sol! {
    #[sol(rpc)]
    contract ERC20 {
        function name() public view returns (string memory);
        function symbol() public view returns (string memory);
        function decimals() public view returns (uint8);
        function totalSupply() public view returns (uint256);
        function balanceOf(address account) public view returns (uint256);
        function transfer(address to, uint256 amount) public returns (bool);
        function allowance(address owner, address spender) public view returns (uint256);
        function approve(address spender, uint256 amount) public returns (bool);
        function transferFrom(address from, address to, uint256 amount) public returns (bool);
    }
}

#[allow(unused)]
impl EthClient {
    /// Returns the chain id of the current network.
    pub async fn chain_id(rpc_url: &str) -> Result<u64, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get chain ID: {e}")))?;
        Ok(chain_id)
    }

    /// Returns the balance of an address as an [EthereumAmount].
    pub async fn balance(rpc_url: &str, address: Address) -> Result<EthereumAmount, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let balance = provider
            .get_balance(address)
            .await
            .map_err(|e| Error::Custom(format!("Failed to get balance: {e}")))?;
        Ok(EthereumAmount { wei: balance })
    }

    /// Gets a transaction given a specific tx hash.
    ///
    /// Returns an error[Error] if the transaction is not found.
    pub async fn get_transaction_data_from_tx_hash(
        rpc_url: &str,
        tx_hash: B256,
    ) -> Result<Transaction, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => {
                if tx.block_hash.is_none() {
                    Err(Error::TxResponse(format!(
                        "Transaction with tx_hash {tx_hash} not found"
                    )))
                } else {
                    Ok(tx)
                }
            }
            Ok(None) => Err(Error::TxResponse(format!(
                "Transaction with tx_hash {tx_hash} not found"
            ))),
            Err(error) => Err(Error::TxResponse(error.to_string())),
        }
    }

    /// Get the current price of gas as an [EthereumAmount].
    pub async fn gas_price(rpc_url: &str) -> Result<EthereumAmount, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let gas_price = provider
            .get_gas_price()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get gas price: {e}")))?;
        Ok(EthereumAmount { wei: U256::from(gas_price) })
    }

    /// Get the latest block number for the current network chain.
    pub async fn current_block_number(rpc_url: &str) -> Result<u64, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let block_number = provider
            .get_block_number()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get block number: {e}")))?;
        Ok(block_number)
    }

    /// Gets the latest block's data.
    pub async fn latest_block(rpc_url: &str) -> Result<Block, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let block_data = provider
            .get_block(BlockId::Number(BlockNumberOrTag::Latest))
            .full()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get block: {e}")))?
            .ok_or_else(|| Error::Custom("Block not found".to_string()))?;

        Ok(block_data)
    }

    /// Gets current chain's block using a specified block number.
    pub async fn block_data_from_u64(rpc_url: &str, block_num: u64) -> Result<Block, Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);
        let block_data = provider
            .get_block(BlockId::Number(BlockNumberOrTag::Number(block_num)))
            .full()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get block: {e}")))?
            .ok_or_else(|| Error::Custom("Block not found".to_string()))?;
        Ok(block_data)
    }
}

#[cfg(test)]
fn anvil_available() -> bool {
    std::process::Command::new("anvil")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::node_bindings::Anvil;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_ethereum_amount_creation() {
        let amount = EthereumAmount::from_eth(1.5);
        assert_eq!(amount.eth(), 1.5);
        
        let wei_amount = EthereumAmount { wei: U256::from(1_000_000_000_000_000_000u128) };
        assert_eq!(wei_amount.eth(), 1.0);
    }

    #[tokio::test]
    async fn test_ethereum_amount_wei() {
        let amount = EthereumAmount::from_eth(1.0);
        assert_eq!(amount.wei(), U256::from(1_000_000_000_000_000_000u128));
    }

    #[tokio::test]
    async fn test_ethereum_amount_gwei() {
        let amount = EthereumAmount { wei: U256::from(1_000_000_000u128) };
        assert_eq!(amount.gwei(), 1.0);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_balance_with_anvil() {
        if !anvil_available() {
            println!("Skipping test - anvil not installed");
            return;
        }

        let anvil = Anvil::new()
            .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
            .spawn();

        let address = Address::from_str("0x3cDB3d9e1B74692Bb1E3bb5fc81938151cA64b02").unwrap();
        let balance: EthereumAmount = EthClient::balance(&anvil.endpoint(), address).await.unwrap();
        // Anvil's default accounts have 10000 ETH
        assert_eq!(balance.wei, U256::from(10000000000000000000000u128));
        drop(anvil);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_chain_id_with_anvil() {
        if !anvil_available() {
            println!("Skipping test - anvil not installed");
            return;
        }

        let anvil = Anvil::new().spawn();
        let chain_id = EthClient::chain_id(&anvil.endpoint()).await.unwrap();
        assert_eq!(chain_id, 31337); // Anvil's default chain ID
        drop(anvil);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_block_number_with_anvil() {
        if !anvil_available() {
            println!("Skipping test - anvil not installed");
            return;
        }

        let anvil = Anvil::new().spawn();
        let block_num = EthClient::current_block_number(&anvil.endpoint()).await.unwrap();
        assert_eq!(block_num, 0); // Fresh Anvil starts at block 0
        drop(anvil);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_gas_price_with_anvil() {
        if !anvil_available() {
            println!("Skipping test - anvil not installed");
            return;
        }

        let anvil = Anvil::new().spawn();
        let gas_price = EthClient::gas_price(&anvil.endpoint()).await.unwrap();
        assert!(gas_price.wei > U256::ZERO);
        drop(anvil);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_latest_block_with_anvil() {
        if !anvil_available() {
            println!("Skipping test - anvil not installed");
            return;
        }

        let anvil = Anvil::new().spawn();
        let block = EthClient::latest_block(&anvil.endpoint()).await.unwrap();
        assert_eq!(block.header.number, 0);
        drop(anvil);
    }
}
