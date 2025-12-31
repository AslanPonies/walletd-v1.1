//! Integration tests for the Ethereum module
//!
//! These tests require Anvil (local Ethereum node) for execution.
//! Install with: `cargo install --git https://github.com/foundry-rs/foundry anvil`
//!
//! Run with: `cargo test -p walletd_ethereum --test integration`

use walletd_ethereum::prelude::*;
use std::str::FromStr;

/// Test mnemonic for deterministic wallet generation
const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Check if Anvil is available
fn anvil_available() -> bool {
    std::process::Command::new("anvil")
        .arg("--version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod wallet_tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .expect("Failed to create wallet");

        let address = wallet.public_address();
        assert!(address.starts_with("0x"), "Address should start with 0x");
        assert_eq!(address.len(), 42, "Address should be 42 characters");
    }

    #[test]
    fn test_wallet_deterministic_address() {
        // Same mnemonic should produce same address
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet1 = EthereumWallet::builder()
            .mnemonic(mnemonic1)
            .build()
            .unwrap();

        let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet2 = EthereumWallet::builder()
            .mnemonic(mnemonic2)
            .build()
            .unwrap();

        assert_eq!(
            wallet1.public_address(),
            wallet2.public_address(),
            "Same mnemonic should produce same address"
        );
    }

    #[test]
    fn test_wallet_formats() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        // Test checksummed format (default)
        let checksummed = EthereumWallet::builder()
            .mnemonic(mnemonic.clone())
            .address_format(EthereumFormat::Checksummed)
            .build()
            .unwrap();

        // Test non-checksummed format
        let non_checksummed = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .address_format(EthereumFormat::NonChecksummed)
            .build()
            .unwrap();

        // Addresses should be equal when compared case-insensitively
        assert_eq!(
            checksummed.public_address().to_lowercase(),
            non_checksummed.public_address().to_lowercase()
        );
    }

    #[test]
    fn test_wallet_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        let mainnet = EthereumWallet::builder()
            .mnemonic(mnemonic.clone())
            .chain_id(1)
            .build()
            .unwrap();

        let sepolia = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111)
            .build()
            .unwrap();

        assert_eq!(mainnet.chain_id(), 1);
        assert_eq!(sepolia.chain_id(), 11155111);
        
        // Same address regardless of chain
        assert_eq!(mainnet.public_address(), sepolia.public_address());
    }

    #[test]
    fn test_receive_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let receive = wallet.receive_address().unwrap();
        assert_eq!(receive, wallet.public_address());
    }
}

#[cfg(test)]
mod amount_tests {
    use super::*;

    #[test]
    fn test_amount_from_wei() {
        let amount = EthereumAmount::from_wei(U256::from(1_000_000_000_000_000_000u128));
        assert_eq!(amount.eth(), 1.0);
    }

    #[test]
    fn test_amount_from_eth() {
        let amount = EthereumAmount::from_eth(1.5);
        assert_eq!(amount.wei(), U256::from(1_500_000_000_000_000_000u128));
    }

    #[test]
    fn test_amount_zero() {
        let zero = EthereumAmount::zero();
        assert_eq!(zero.wei(), U256::ZERO);
        assert_eq!(zero.eth(), 0.0);
    }

    #[test]
    fn test_amount_gwei_conversion() {
        // 1 ETH = 1e9 gwei
        let one_eth = EthereumAmount::from_eth(1.0);
        let gwei = one_eth.wei() / U256::from(1_000_000_000u64);
        assert_eq!(gwei, U256::from(1_000_000_000u64));
    }

    #[test]
    fn test_small_amounts() {
        // Test 1 wei
        let one_wei = EthereumAmount::from_wei(U256::from(1u64));
        assert_eq!(one_wei.wei(), U256::from(1u64));
        
        // ETH value should be very small but non-zero
        // 1 wei = 1e-18 ETH
        let eth = one_wei.eth();
        assert!(eth > 0.0);
        assert!(eth <= 1e-17); // 1 wei is 1e-18 ETH
    }

    #[test]
    fn test_large_amounts() {
        // Test 1 million ETH
        let million = EthereumAmount::from_eth(1_000_000.0);
        assert!(million.wei() > U256::ZERO);
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

    // Note: These tests require a running Anvil instance
    // They are marked as ignored by default

    #[ignore]
    #[tokio::test]
    async fn test_get_chain_id() {
        if !anvil_available() {
            println!("Skipping - Anvil not available");
            return;
        }

        use alloy::node_bindings::Anvil;
        let anvil = Anvil::new().spawn();
        
        let chain_id = EthClient::chain_id(&anvil.endpoint()).await.unwrap();
        assert_eq!(chain_id, 31337); // Anvil default chain ID
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_balance() {
        if !anvil_available() {
            println!("Skipping - Anvil not available");
            return;
        }

        use alloy::node_bindings::Anvil;
        let anvil = Anvil::new().spawn();
        
        // Anvil's first default account
        let address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap();
        let balance = EthClient::balance(&anvil.endpoint(), address).await.unwrap();
        
        // Anvil default accounts have 10000 ETH
        assert!(balance.eth() > 9999.0);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_gas_price() {
        if !anvil_available() {
            println!("Skipping - Anvil not available");
            return;
        }

        use alloy::node_bindings::Anvil;
        let anvil = Anvil::new().spawn();
        
        let gas_price = EthClient::gas_price(&anvil.endpoint()).await.unwrap();
        assert!(gas_price.wei() > U256::ZERO);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_block_number() {
        if !anvil_available() {
            println!("Skipping - Anvil not available");
            return;
        }

        use alloy::node_bindings::Anvil;
        let anvil = Anvil::new().spawn();
        
        let block_num = EthClient::current_block_number(&anvil.endpoint()).await.unwrap();
        assert!(block_num >= 0);
    }

    #[ignore]
    #[tokio::test]
    async fn test_get_latest_block() {
        if !anvil_available() {
            println!("Skipping - Anvil not available");
            return;
        }

        use alloy::node_bindings::Anvil;
        let anvil = Anvil::new().spawn();
        
        let block = EthClient::latest_block(&anvil.endpoint()).await.unwrap();
        assert!(block.header.number >= 0);
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", EthereumFormat::Checksummed), "Checksummed");
        assert_eq!(format!("{}", EthereumFormat::NonChecksummed), "NonChecksummed");
    }

    #[test]
    fn test_format_default() {
        let default: EthereumFormat = Default::default();
        assert!(matches!(default, EthereumFormat::Checksummed));
    }
}
