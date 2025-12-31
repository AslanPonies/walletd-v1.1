//! Comprehensive edge case and security tests for Bitcoin wallet

use walletd_bitcoin::{BitcoinWallet, BdkAddressType};
use bdk::bitcoin::Network;
use bdk::keys::bip39::Mnemonic;

const TEST_MNEMONIC_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const TEST_MNEMONIC_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// ============================================================================
// Mnemonic Validation Tests
// ============================================================================

#[test]
fn test_12_word_mnemonic() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build();
    assert!(wallet.is_ok());
}

#[test]
fn test_24_word_mnemonic() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_24).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build();
    assert!(wallet.is_ok());
}

#[test]
fn test_invalid_mnemonic() {
    let result = Mnemonic::parse("invalid mnemonic words");
    assert!(result.is_err());
}

#[test]
fn test_empty_mnemonic() {
    let result = Mnemonic::parse("");
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
fn test_builder_without_mnemonic() {
    let result = BitcoinWallet::builder().build();
    assert!(result.is_err());
}

// ============================================================================
// Network Type Tests
// ============================================================================

#[test]
fn test_mainnet_default() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build().unwrap();
    assert_eq!(wallet.network().unwrap(), Network::Bitcoin);
}

#[test]
fn test_testnet() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()
        .unwrap();
    assert_eq!(wallet.network().unwrap(), Network::Testnet);
}

#[test]
fn test_regtest() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Regtest)
        .build()
        .unwrap();
    assert_eq!(wallet.network().unwrap(), Network::Regtest);
}

#[test]
fn test_mainnet_address_prefix() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Bitcoin)
        .build()
        .unwrap();
    let addr = wallet.receive_address().unwrap();
    assert!(addr.starts_with("bc1"), "Expected bc1 prefix, got: {}", addr);
}

#[test]
fn test_testnet_address_prefix() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()
        .unwrap();
    let addr = wallet.receive_address().unwrap();
    assert!(addr.starts_with("tb1"), "Expected tb1 prefix, got: {}", addr);
}

#[test]
fn test_regtest_address_prefix() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Regtest)
        .build()
        .unwrap();
    let addr = wallet.receive_address().unwrap();
    assert!(addr.starts_with("bcrt1"), "Expected bcrt1 prefix, got: {}", addr);
}

// ============================================================================
// Address Format Tests
// ============================================================================

#[test]
fn test_default_address_format_p2wpkh() {
    let wallet = BitcoinWallet::default();
    assert_eq!(wallet.address_format(), BdkAddressType::P2wpkh);
}

#[test]
fn test_p2wpkh_format() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .address_format(BdkAddressType::P2wpkh)
        .build()
        .unwrap();
    assert_eq!(wallet.address_format(), BdkAddressType::P2wpkh);
    let addr = wallet.receive_address().unwrap();
    assert!(addr.starts_with("bc1q"), "P2WPKH should start with bc1q, got: {}", addr);
}

#[test]
fn test_p2pkh_format() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .address_format(BdkAddressType::P2pkh)
        .build()
        .unwrap();
    assert_eq!(wallet.address_format(), BdkAddressType::P2pkh);
}

#[test]
fn test_p2sh_format() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .address_format(BdkAddressType::P2sh)
        .build()
        .unwrap();
    assert_eq!(wallet.address_format(), BdkAddressType::P2sh);
}

// ============================================================================
// HD Purpose Tests
// ============================================================================

#[test]
fn test_p2wpkh_uses_bip84() {
    use walletd_hd_key::HDPurpose;
    let wallet = BitcoinWallet::default();
    assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP84);
}

#[test]
fn test_p2pkh_uses_bip44() {
    use walletd_hd_key::HDPurpose;
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .address_format(BdkAddressType::P2pkh)
        .build()
        .unwrap();
    assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP44);
}

#[test]
fn test_p2sh_uses_bip49() {
    use walletd_hd_key::HDPurpose;
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .address_format(BdkAddressType::P2sh)
        .build()
        .unwrap();
    assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP49);
}

// ============================================================================
// Coin Type Tests
// ============================================================================

#[test]
fn test_mainnet_coin_type() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Bitcoin)
        .build()
        .unwrap();
    assert_eq!(wallet.coin_type_id().unwrap(), 0);
}

#[test]
fn test_testnet_coin_type() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()
        .unwrap();
    assert_eq!(wallet.coin_type_id().unwrap(), 1);
}

#[test]
fn test_regtest_coin_type() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Regtest)
        .build()
        .unwrap();
    assert_eq!(wallet.coin_type_id().unwrap(), 1);
}

// ============================================================================
// Deterministic Derivation Tests
// ============================================================================

#[test]
fn test_same_mnemonic_same_address() {
    let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    
    let wallet1 = BitcoinWallet::builder().mnemonic(mnemonic1).build().unwrap();
    let wallet2 = BitcoinWallet::builder().mnemonic(mnemonic2).build().unwrap();
    
    assert_eq!(
        wallet1.receive_address().unwrap(),
        wallet2.receive_address().unwrap()
    );
}

#[test]
fn test_different_mnemonic_different_address() {
    let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let mnemonic2 = Mnemonic::parse("zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong").unwrap();
    
    let wallet1 = BitcoinWallet::builder().mnemonic(mnemonic1).build().unwrap();
    let wallet2 = BitcoinWallet::builder().mnemonic(mnemonic2).build().unwrap();
    
    assert_ne!(
        wallet1.receive_address().unwrap(),
        wallet2.receive_address().unwrap()
    );
}

#[test]
fn test_next_address_increments() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build().unwrap();
    
    let addr1 = wallet.next_address().unwrap();
    let addr2 = wallet.next_address().unwrap();
    let addr3 = wallet.next_address().unwrap();
    
    assert_eq!(addr1.index, 0);
    assert_eq!(addr2.index, 1);
    assert_eq!(addr3.index, 2);
    assert_ne!(addr1.address, addr2.address);
    assert_ne!(addr2.address, addr3.address);
}

#[test]
fn test_known_address_derivation() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Bitcoin)
        .address_format(BdkAddressType::P2wpkh)
        .build()
        .unwrap();
    
    let addr = wallet.receive_address().unwrap();
    assert!(addr.starts_with("bc1q"));
    assert!(addr.len() >= 42 && addr.len() <= 44);
}

// ============================================================================
// Balance Tests (Offline)
// ============================================================================

#[tokio::test]
async fn test_new_wallet_zero_balance() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build().unwrap();
    
    let balance = wallet.balance().await.unwrap();
    assert_eq!(balance.confirmed, 0);
    assert_eq!(balance.immature, 0);
    assert_eq!(balance.trusted_pending, 0);
    assert_eq!(balance.untrusted_pending, 0);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_network_error_without_wallet() {
    let wallet = BitcoinWallet::default();
    let result = wallet.network();
    assert!(result.is_err());
}

#[test]
fn test_missing_mnemonic_error() {
    let result = BitcoinWallet::builder().build();
    assert!(result.is_err());
}

// ============================================================================
// Address Validation Tests
// ============================================================================

#[test]
fn test_mainnet_p2wpkh_address_length() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Bitcoin)
        .address_format(BdkAddressType::P2wpkh)
        .build()
        .unwrap();
    
    let addr = wallet.receive_address().unwrap();
    assert!(addr.len() >= 42 && addr.len() <= 44, "Invalid length: {}", addr.len());
}

#[test]
fn test_address_contains_only_valid_chars() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build().unwrap();
    let addr = wallet.receive_address().unwrap();
    
    let valid_chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let addr_lower = addr.to_lowercase();
    
    for c in addr_lower[3..].chars() {
        assert!(valid_chars.contains(c), "Invalid char '{}' in address", c);
    }
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_builder_chain_methods() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .address_format(BdkAddressType::P2wpkh)
        .build()
        .unwrap();
    
    assert_eq!(wallet.network().unwrap(), Network::Testnet);
    assert_eq!(wallet.address_format(), BdkAddressType::P2wpkh);
}

#[test]
fn test_builder_defaults() {
    let mnemonic = Mnemonic::parse(TEST_MNEMONIC_12).unwrap();
    let wallet = BitcoinWallet::builder().mnemonic(mnemonic).build().unwrap();
    
    assert_eq!(wallet.network().unwrap(), Network::Bitcoin);
    assert_eq!(wallet.address_format(), BdkAddressType::P2wpkh);
}
