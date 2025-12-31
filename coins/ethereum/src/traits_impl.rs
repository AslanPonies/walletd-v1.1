//! Implementation of walletd-traits for EthereumWallet

use async_trait::async_trait;
use walletd_traits::{Amount, Network, Transferable, TxHash, Wallet, WalletError, WalletResult};

use crate::{EthereumWallet, EthClient};

impl EthereumWallet {
    /// Creates a Network struct for this wallet
    fn get_network(&self) -> Network {
        let (name, is_testnet) = match self.chain_id() {
            1 => ("Ethereum Mainnet", false),
            11155111 => ("Sepolia", true),
            5 => ("Goerli", true),
            _ => ("Unknown", false),
        };
        Network {
            name: name.to_string(),
            chain_id: Some(self.chain_id()),
            is_testnet,
        }
    }
}

/// Wrapper that holds an EthereumWallet with an RPC URL for trait implementations
pub struct ConnectedEthereumWallet {
    /// The underlying wallet
    pub wallet: EthereumWallet,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Cached network info
    network: Network,
}

impl ConnectedEthereumWallet {
    /// Creates a new connected wallet
    pub fn new(wallet: EthereumWallet, rpc_url: impl Into<String>) -> Self {
        let network = wallet.get_network();
        Self {
            wallet,
            rpc_url: rpc_url.into(),
            network,
        }
    }
}

#[async_trait]
impl Wallet for ConnectedEthereumWallet {
    fn address(&self) -> String {
        self.wallet.public_address()
    }

    async fn balance(&self) -> WalletResult<Amount> {
        let balance = self.wallet.balance(&self.rpc_url).await
            .map_err(|e| WalletError::NetworkError(e.to_string()))?;
        
        // Convert EthereumAmount to Amount
        // EthereumAmount.wei is alloy U256, we need u128
        let wei_bytes = balance.wei().to_le_bytes::<32>();
        let wei_u128 = u128::from_le_bytes(wei_bytes[0..16].try_into().unwrap());
        
        Ok(Amount::from_smallest_unit(wei_u128, 18))
    }

    fn network(&self) -> &Network {
        &self.network
    }

    fn currency_symbol(&self) -> &str {
        "ETH"
    }

    fn decimals(&self) -> u8 {
        18
    }
}

#[async_trait]
impl Transferable for ConnectedEthereumWallet {
    async fn transfer(&self, to: &str, amount: Amount) -> WalletResult<TxHash> {
        // Convert Amount to EthereumAmount
        let eth_amount = crate::EthereumAmount::from_wei(
            alloy::primitives::U256::from(amount.smallest_unit())
        );
        
        let tx_hash = self.wallet.transfer(&self.rpc_url, eth_amount, to).await
            .map_err(|e| WalletError::TransactionFailed(e.to_string()))?;
        
        Ok(TxHash::new(tx_hash))
    }

    async fn estimate_fee(&self, _to: &str, _amount: Amount) -> WalletResult<Amount> {
        // Get current gas price and estimate gas (21000 for simple transfer)
        let gas_price = EthClient::gas_price(&self.rpc_url).await
            .map_err(|e| WalletError::NetworkError(e.to_string()))?;
        
        // Simple ETH transfer uses 21000 gas
        let gas_limit = 21000u128;
        let gas_price_wei = gas_price.wei();
        let gas_price_bytes = gas_price_wei.to_le_bytes::<32>();
        let gas_price_u128 = u128::from_le_bytes(gas_price_bytes[0..16].try_into().unwrap());
        
        let fee = gas_limit.saturating_mul(gas_price_u128);
        Ok(Amount::from_smallest_unit(fee, 18))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdk::keys::bip39::Mnemonic;
    use std::str::FromStr;

    #[test]
    fn test_ethereum_wallet_address() {
        let mnemonic = Mnemonic::from_str(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();
        
        let connected = ConnectedEthereumWallet::new(wallet, "https://eth.llamarpc.com");
        
        // Should return a valid Ethereum address
        let address = connected.address();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_network_info() {
        let mnemonic = Mnemonic::from_str(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(1)
            .build()
            .unwrap();
        
        let connected = ConnectedEthereumWallet::new(wallet, "https://eth.llamarpc.com");
        
        assert_eq!(connected.currency_symbol(), "ETH");
        assert_eq!(connected.decimals(), 18);
        assert_eq!(connected.network().chain_id, Some(1));
        assert!(!connected.network().is_testnet);
    }

    #[test]
    fn test_sepolia_network() {
        let mnemonic = Mnemonic::from_str(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap();
        
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111)
            .build()
            .unwrap();
        
        let connected = ConnectedEthereumWallet::new(wallet, "https://sepolia.infura.io");
        
        assert!(connected.network().is_testnet);
        assert_eq!(connected.network().name, "Sepolia");
    }
}
