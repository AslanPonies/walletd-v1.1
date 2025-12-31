//! Integration tests for walletd_ethereum

use walletd_ethereum::prelude::*;
use walletd_ethereum::{EthClient, EthereumAmount, EthereumWallet, ConnectedEthereumWallet};
use walletd_traits::prelude::*;
use std::str::FromStr;

/// Test mnemonic (DO NOT USE IN PRODUCTION)
const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

mod wallet_tests {
    use super::*;

    #[test]
    fn test_wallet_creation_from_mnemonic() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let address = wallet.public_address();
        assert!(address.starts_with("0x"), "Address should start with 0x");
        assert_eq!(address.len(), 42, "Ethereum address should be 42 characters");
    }

    #[test]
    fn test_wallet_deterministic_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        // Create wallet twice with same mnemonic
        let wallet1 = EthereumWallet::builder()
            .mnemonic(mnemonic.clone())
            .build()
            .unwrap();
        
        let wallet2 = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        assert_eq!(
            wallet1.public_address(),
            wallet2.public_address(),
            "Same mnemonic should produce same address"
        );
    }

    #[test]
    fn test_wallet_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        let mainnet_wallet = EthereumWallet::builder()
            .mnemonic(mnemonic.clone())
            .chain_id(1)
            .build()
            .unwrap();
        
        let sepolia_wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111)
            .build()
            .unwrap();

        assert_eq!(mainnet_wallet.chain_id(), 1);
        assert_eq!(sepolia_wallet.chain_id(), 11155111);
    }

    #[test]
    fn test_wallet_without_mnemonic_fails() {
        let result = EthereumWallet::builder().build();
        assert!(result.is_err(), "Building wallet without mnemonic should fail");
    }
}

mod amount_tests {
    use super::*;

    #[test]
    fn test_ethereum_amount_from_wei() {
        let amount = EthereumAmount::from_wei(alloy::primitives::U256::from(1_000_000_000_000_000_000u128));
        assert_eq!(amount.eth(), 1.0);
    }

    #[test]
    fn test_ethereum_amount_from_eth() {
        let amount = EthereumAmount::from_eth(1.5);
        assert_eq!(amount.eth(), 1.5);
    }

    #[test]
    fn test_ethereum_amount_zero() {
        let amount = EthereumAmount::zero();
        assert_eq!(amount.eth(), 0.0);
        assert_eq!(amount.wei(), alloy::primitives::U256::ZERO);
    }

    #[test]
    fn test_ethereum_amount_gwei() {
        let amount = EthereumAmount::from_gwei(1_000_000_000); // 1 ETH in gwei
        assert_eq!(amount.eth(), 1.0);
    }

    #[test]
    fn test_amount_arithmetic() {
        let a = EthereumAmount::from_eth(1.0);
        let b = EthereumAmount::from_eth(0.5);
        
        // Test that wei values are correct
        let a_wei = a.wei();
        let b_wei = b.wei();
        
        assert!(a_wei > b_wei, "1 ETH should be greater than 0.5 ETH");
    }
}

mod traits_tests {
    use super::*;

    #[test]
    fn test_connected_wallet_creation() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let connected = ConnectedEthereumWallet::new(wallet, "https://eth.llamarpc.com");
        
        assert_eq!(connected.currency_symbol(), "ETH");
        assert_eq!(connected.decimals(), 18);
    }

    #[test]
    fn test_wallet_trait_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let connected = ConnectedEthereumWallet::new(wallet.clone(), "https://eth.llamarpc.com");
        
        // Both should return the same address
        assert_eq!(
            connected.address(),
            wallet.public_address(),
            "Trait address should match wallet address"
        );
    }

    #[test]
    fn test_mainnet_network() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(1)
            .build()
            .unwrap();

        let connected = ConnectedEthereumWallet::new(wallet, "https://eth.llamarpc.com");
        let network = connected.network();
        
        assert!(!network.is_testnet);
        assert_eq!(network.chain_id, Some(1));
        assert_eq!(network.name, "Ethereum Mainnet");
    }

    #[test]
    fn test_sepolia_network() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111)
            .build()
            .unwrap();

        let connected = ConnectedEthereumWallet::new(wallet, "https://sepolia.infura.io");
        let network = connected.network();
        
        assert!(network.is_testnet);
        assert_eq!(network.chain_id, Some(11155111));
        assert_eq!(network.name, "Sepolia");
    }
}

mod format_tests {
    use super::*;
    use walletd_ethereum::EthereumFormat;

    #[test]
    fn test_checksummed_format() {
        let format = EthereumFormat::Checksummed;
        assert_eq!(format.to_string(), "Checksummed");
    }

    #[test]
    fn test_non_checksummed_format() {
        let format = EthereumFormat::NonChecksummed;
        assert_eq!(format.to_string(), "NonChecksummed");
    }

    #[test]
    fn test_default_format_is_checksummed() {
        let format = EthereumFormat::default();
        assert_eq!(format.to_string(), "Checksummed");
    }
}

// Integration tests that require network (marked as ignored by default)
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_balance_mainnet() {
        // Vitalik's address - should have some ETH
        let address: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse().unwrap();
        let rpc_url = "https://eth.llamarpc.com";
        
        let balance = EthClient::balance(rpc_url, address).await;
        assert!(balance.is_ok(), "Should be able to fetch balance");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_gas_price() {
        let rpc_url = "https://eth.llamarpc.com";
        
        let gas_price = EthClient::gas_price(rpc_url).await;
        assert!(gas_price.is_ok(), "Should be able to fetch gas price");
        
        let price = gas_price.unwrap();
        assert!(price.wei() > alloy::primitives::U256::ZERO, "Gas price should be non-zero");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_block_number() {
        let rpc_url = "https://eth.llamarpc.com";
        
        let block_number = EthClient::current_block_number(rpc_url).await;
        assert!(block_number.is_ok(), "Should be able to fetch block number");
        assert!(block_number.unwrap() > 0, "Block number should be positive");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_chain_id() {
        let rpc_url = "https://eth.llamarpc.com";
        
        let chain_id = EthClient::chain_id(rpc_url).await;
        assert!(chain_id.is_ok(), "Should be able to fetch chain ID");
        assert_eq!(chain_id.unwrap(), 1, "Mainnet chain ID should be 1");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_wallet_balance_via_trait() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let connected = ConnectedEthereumWallet::new(wallet, "https://eth.llamarpc.com");
        
        let balance = connected.balance().await;
        assert!(balance.is_ok(), "Should be able to fetch balance via trait");
    }
}
