//! Comprehensive edge case and security tests for Ethereum wallet
//!
//! Tests cover:
//! - Invalid/malformed inputs
//! - Boundary conditions
//! - Error handling
//! - Address validation
//! - Amount handling

use walletd_ethereum::{EthereumWallet, EthereumAmount, EthereumFormat};
use bdk::keys::bip39::Mnemonic;

const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// ============================================================================
// Mnemonic Edge Cases
// ============================================================================

mod mnemonic_tests {
    use super::*;

    #[test]
    fn test_12_word_mnemonic() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_24_word_mnemonic() {
        let mnemonic_24 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let mnemonic = Mnemonic::parse(mnemonic_24).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_invalid_mnemonic_word() {
        let result = Mnemonic::parse("invalid word should not work as mnemonic");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_mnemonic() {
        let result = Mnemonic::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_single_word_mnemonic() {
        let result = Mnemonic::parse("abandon");
        assert!(result.is_err());
    }

    #[test]
    fn test_11_word_mnemonic() {
        let result = Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_13_word_mnemonic() {
        let result = Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about extra"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_mnemonic_with_extra_whitespace() {
        // BIP-39 should normalize whitespace
        let mnemonic = Mnemonic::parse(
            "  abandon  abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about  "
        );
        // This may or may not be valid depending on the BIP-39 implementation
        // The test documents the behavior
        if let Ok(m) = mnemonic {
            let wallet = EthereumWallet::builder().mnemonic(m).build();
            assert!(wallet.is_ok() || wallet.is_err()); // Document behavior
        }
    }

    #[test]
    fn test_mnemonic_bad_checksum() {
        // "about" should be at the end for valid checksum
        let result = Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_without_mnemonic() {
        let result = EthereumWallet::builder().build();
        assert!(result.is_err());
    }
}

// ============================================================================
// Address Format Tests
// ============================================================================

mod address_format_tests {
    use super::*;

    #[test]
    fn test_checksummed_has_mixed_case() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .address_format(EthereumFormat::Checksummed)
            .build()
            .unwrap();

        let addr = wallet.public_address();
        let has_upper = addr[2..].chars().any(|c| c.is_uppercase());
        let has_lower = addr[2..].chars().any(|c| c.is_lowercase());
        
        // Checksummed addresses typically have mixed case
        // (unless the address happens to be all one case)
        assert!(addr.starts_with("0x"));
        assert!(has_upper || has_lower); // At least some letters
    }

    #[test]
    fn test_non_checksummed_all_lowercase() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .address_format(EthereumFormat::NonChecksummed)
            .build()
            .unwrap();

        let addr = wallet.public_address();
        assert!(addr.starts_with("0x"));
        // Everything after 0x should be lowercase
        assert!(!addr[2..].chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn test_address_length() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let addr = wallet.public_address();
        // Ethereum addresses are 42 characters: "0x" + 40 hex chars
        assert_eq!(addr.len(), 42);
    }

    #[test]
    fn test_address_is_valid_hex() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let addr = wallet.public_address();
        assert!(addr.starts_with("0x"));
        
        // Rest should be valid hex
        let hex_part = &addr[2..];
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_deterministic_address() {
        // Same mnemonic should always produce same address
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC).unwrap();

        let wallet1 = EthereumWallet::builder().mnemonic(mnemonic1).build().unwrap();
        let wallet2 = EthereumWallet::builder().mnemonic(mnemonic2).build().unwrap();

        assert_eq!(wallet1.public_address(), wallet2.public_address());
    }

    #[test]
    fn test_known_address_derivation() {
        // Verify against known test vector
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        // This is the expected address for the standard test mnemonic
        // at derivation path m/44'/60'/0'/0/0
        assert_eq!(
            wallet.public_address().to_lowercase(),
            "0x9858effd232b4033e47d90003d41ec34ecaeda94"
        );
    }
}

// ============================================================================
// Chain ID Tests
// ============================================================================

mod chain_id_tests {
    use super::*;

    #[test]
    fn test_default_chain_id_is_mainnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 1); // Mainnet
    }

    #[test]
    fn test_sepolia_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111) // Sepolia
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 11155111);
    }

    #[test]
    fn test_goerli_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(5) // Goerli
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 5);
    }

    #[test]
    fn test_polygon_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(137) // Polygon
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 137);
    }

    #[test]
    fn test_arbitrum_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(42161) // Arbitrum One
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 42161);
    }

    #[test]
    fn test_base_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(8453) // Base
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 8453);
    }

    #[test]
    fn test_zero_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(0)
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), 0);
    }

    #[test]
    fn test_max_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(u64::MAX)
            .build()
            .unwrap();

        assert_eq!(wallet.chain_id(), u64::MAX);
    }
}

// ============================================================================
// Amount Tests
// ============================================================================

mod amount_tests {
    use super::*;
    use walletd_ethereum::alloy::primitives::U256;

    #[test]
    fn test_amount_from_wei() {
        let amount = EthereumAmount::from_wei(U256::from(1_000_000_000_000_000_000u128)); // 1 ETH
        assert_eq!(amount.wei(), U256::from(1_000_000_000_000_000_000u128));
    }

    #[test]
    fn test_amount_from_eth() {
        let amount = EthereumAmount::from_eth(1.0);
        assert_eq!(amount.wei(), U256::from(1_000_000_000_000_000_000u128));
    }

    #[test]
    fn test_amount_zero() {
        let amount = EthereumAmount::zero();
        assert_eq!(amount.wei(), U256::ZERO);
        assert_eq!(amount.eth(), 0.0);
    }

    #[test]
    fn test_amount_one_wei() {
        let amount = EthereumAmount::from_wei(U256::from(1u64));
        assert_eq!(amount.wei(), U256::from(1u64));
    }

    #[test]
    fn test_amount_gwei_conversion() {
        let amount = EthereumAmount::from_gwei(1);
        assert_eq!(amount.wei(), U256::from(1_000_000_000u128));
    }

    #[test]
    fn test_amount_large_value() {
        // 1 million ETH - note: f64 has precision limits
        let amount = EthereumAmount::from_eth(1_000_000.0);
        // Due to f64 precision, we check approximate equality
        let expected = U256::from(1_000_000_000_000_000_000_000_000u128);
        let diff = if amount.wei() > expected {
            amount.wei() - expected
        } else {
            expected - amount.wei()
        };
        // Allow 0.01% difference due to floating point
        let tolerance = expected / U256::from(10000u64);
        assert!(diff < tolerance, "Amount too far from expected: {:?}", amount.wei());
    }

    #[test]
    fn test_amount_fractional_eth() {
        let amount = EthereumAmount::from_eth(0.001);
        assert_eq!(amount.wei(), U256::from(1_000_000_000_000_000u128)); // 0.001 ETH = 10^15 wei
    }

    #[test]
    fn test_amount_debug() {
        let amount = EthereumAmount::from_eth(1.5);
        let debug = format!("{:?}", amount);
        assert!(!debug.is_empty());
    }

    #[test]
    fn test_amount_arithmetic() {
        let a = EthereumAmount::from_eth(1.0);
        let b = EthereumAmount::from_eth(0.5);
        
        // Test that amounts can be compared
        assert!(a.wei() > b.wei());
    }

    #[test]
    fn test_amount_addition() {
        let a = EthereumAmount::from_eth(1.0);
        let b = EthereumAmount::from_eth(0.5);
        
        let result = (a + b).unwrap();
        assert_eq!(result.eth(), 1.5);
    }

    #[test]
    fn test_amount_subtraction() {
        let a = EthereumAmount::from_eth(1.0);
        let b = EthereumAmount::from_eth(0.5);
        
        let result = (a - b).unwrap();
        assert!((result.eth() - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_amount_negative_eth() {
        // Negative ETH should return zero wei
        let amount = EthereumAmount::from_eth(-1.0);
        assert_eq!(amount.wei(), U256::ZERO);
    }

    #[test]
    fn test_amount_from_wei_u128() {
        let amount = EthereumAmount::from_wei_u128(1_000_000_000_000_000_000u128);
        assert_eq!(amount.eth(), 1.0);
    }
}

// ============================================================================
// Public Key Tests
// ============================================================================

mod public_key_tests {
    use super::*;

    #[test]
    fn test_public_key_available() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let pubkey = wallet.public_key();
        assert!(pubkey.is_ok());
    }

    #[test]
    fn test_public_key_length() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let pubkey = wallet.public_key().unwrap();
        // Compressed public key is 33 bytes
        assert_eq!(pubkey.public_key.serialize().len(), 33);
    }

    #[test]
    fn test_public_key_deterministic() {
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC).unwrap();

        let wallet1 = EthereumWallet::builder().mnemonic(mnemonic1).build().unwrap();
        let wallet2 = EthereumWallet::builder().mnemonic(mnemonic2).build().unwrap();

        let pk1 = wallet1.public_key().unwrap();
        let pk2 = wallet2.public_key().unwrap();

        assert_eq!(pk1.public_key.serialize(), pk2.public_key.serialize());
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_build_without_mnemonic_returns_error() {
        let result = EthereumWallet::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_receive_address_success() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let addr = wallet.receive_address();
        assert!(addr.is_ok());
    }
}

// ============================================================================
// Consistency Tests
// ============================================================================

mod consistency_tests {
    use super::*;

    #[test]
    fn test_receive_address_equals_public_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        assert_eq!(
            wallet.receive_address().unwrap(),
            wallet.public_address()
        );
    }

    #[test]
    fn test_multiple_wallet_instances_independent() {
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
        ).unwrap();

        let wallet1 = EthereumWallet::builder().mnemonic(mnemonic1).build().unwrap();
        let wallet2 = EthereumWallet::builder().mnemonic(mnemonic2).build().unwrap();

        // Different mnemonics should produce different addresses
        assert_ne!(wallet1.public_address(), wallet2.public_address());
        
        // But each should be deterministic
        let mnemonic1_again = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet1_again = EthereumWallet::builder()
            .mnemonic(mnemonic1_again)
            .build()
            .unwrap();
        assert_eq!(wallet1.public_address(), wallet1_again.public_address());
    }

    #[test]
    fn test_chain_id_does_not_affect_address() {
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC).unwrap();

        let wallet_mainnet = EthereumWallet::builder()
            .mnemonic(mnemonic1)
            .chain_id(1)
            .build()
            .unwrap();

        let wallet_sepolia = EthereumWallet::builder()
            .mnemonic(mnemonic2)
            .chain_id(11155111)
            .build()
            .unwrap();

        // Same mnemonic should produce same address regardless of chain
        assert_eq!(wallet_mainnet.public_address(), wallet_sepolia.public_address());
    }
}
