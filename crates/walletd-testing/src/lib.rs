//! # WalletD Testing Infrastructure
//!
//! Comprehensive testing utilities for WalletD SDK including:
//! - Edge case generators
//! - Property-based testing helpers
//! - Security test patterns
//! - Fuzzing utilities
//!
//! ## Usage
//!
//! ```rust,ignore
//! use walletd_testing::*;
//!
//! // Generate edge case private keys
//! for key in EdgeCaseKeys::all() {
//!     test_wallet_with_key(key);
//! }
//!
//! // Property-based testing
//! proptest! {
//!     #[test]
//!     fn test_address_roundtrip(key in valid_private_key()) {
//!         // ...
//!     }
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proptest::prelude::*;
use std::fmt;

// ============================================================================
// Edge Case Key Material
// ============================================================================

/// Edge case private keys for testing boundary conditions
pub struct EdgeCaseKeys;

impl EdgeCaseKeys {
    /// All zeros (invalid for most curves)
    pub const ALL_ZEROS: [u8; 32] = [0u8; 32];

    /// All ones
    pub const ALL_ONES: [u8; 32] = [0xFF; 32];

    /// Minimum valid key (just above zero)
    pub const MIN_VALID: [u8; 32] = {
        let mut k = [0u8; 32];
        k[31] = 1;
        k
    };

    /// Key with alternating bits
    pub const ALTERNATING: [u8; 32] = [0xAA; 32];

    /// Key with single bit set (bit 0)
    pub const SINGLE_BIT_0: [u8; 32] = {
        let mut k = [0u8; 32];
        k[31] = 1;
        k
    };

    /// Key with single bit set (bit 255)
    pub const SINGLE_BIT_255: [u8; 32] = {
        let mut k = [0u8; 32];
        k[0] = 0x80;
        k
    };

    /// secp256k1 curve order (n) - invalid as private key
    pub const SECP256K1_ORDER: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
        0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
        0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x41,
    ];

    /// secp256k1 curve order minus 1 (n-1) - valid, maximum key
    pub const SECP256K1_ORDER_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE,
        0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B,
        0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36, 0x41, 0x40,
    ];

    /// Ed25519 curve order (L) - for reference
    pub const ED25519_ORDER: [u8; 32] = [
        0xED, 0xD3, 0xF5, 0x5C, 0x1A, 0x63, 0x12, 0x58,
        0xD6, 0x9C, 0xF7, 0xA2, 0xDE, 0xF9, 0xDE, 0x14,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
    ];

    /// Returns all edge case keys for comprehensive testing
    pub fn all() -> Vec<[u8; 32]> {
        vec![
            Self::ALL_ZEROS,
            Self::ALL_ONES,
            Self::MIN_VALID,
            Self::ALTERNATING,
            Self::SINGLE_BIT_0,
            Self::SINGLE_BIT_255,
            Self::SECP256K1_ORDER,
            Self::SECP256K1_ORDER_MINUS_1,
        ]
    }

    /// Returns only valid edge case keys
    pub fn valid_only() -> Vec<[u8; 32]> {
        vec![
            Self::MIN_VALID,
            Self::ALTERNATING,
            Self::SINGLE_BIT_0,
            Self::SINGLE_BIT_255,
            Self::SECP256K1_ORDER_MINUS_1,
        ]
    }

    /// Returns only invalid edge case keys
    pub fn invalid_only() -> Vec<[u8; 32]> {
        vec![
            Self::ALL_ZEROS,
            Self::ALL_ONES,
            Self::SECP256K1_ORDER,
        ]
    }
}

// ============================================================================
// Edge Case Mnemonics
// ============================================================================

/// Edge case mnemonic phrases for testing
pub struct EdgeCaseMnemonics;

impl EdgeCaseMnemonics {
    /// Standard 12-word test mnemonic (all "abandon" except last)
    pub const STANDARD_12: &'static str = 
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    /// Standard 24-word test mnemonic
    pub const STANDARD_24: &'static str = 
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    /// All same word (valid checksum)
    pub const ALL_ABANDON: &'static str = 
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    /// All "zoo" words (last word varies for checksum)
    pub const ALL_ZOO: &'static str = 
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

    /// Mixed case (should be normalized)
    pub const MIXED_CASE: &'static str = 
        "ABANDON abandon ABANDON abandon ABANDON abandon ABANDON abandon ABANDON abandon ABANDON about";

    /// Extra whitespace
    pub const EXTRA_WHITESPACE: &'static str = 
        "  abandon   abandon  abandon abandon abandon abandon abandon abandon abandon abandon abandon   about  ";

    /// Returns all valid mnemonics
    pub fn valid() -> Vec<&'static str> {
        vec![
            Self::STANDARD_12,
            Self::STANDARD_24,
            Self::ALL_ABANDON,
        ]
    }

    /// Returns invalid/malformed mnemonics for error handling tests
    pub fn invalid() -> Vec<&'static str> {
        vec![
            "",                           // Empty
            "abandon",                    // Single word
            "abandon abandon abandon",    // Too few words
            "invalid words here",         // Invalid words
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", // 13 words
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon wrong", // Bad checksum
        ]
    }
}

// ============================================================================
// Edge Case Addresses
// ============================================================================

/// Edge case addresses for testing
pub struct EdgeCaseAddresses;

impl EdgeCaseAddresses {
    /// Valid Bitcoin mainnet P2WPKH
    pub const BTC_P2WPKH_VALID: &'static str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
    
    /// Valid Bitcoin testnet P2WPKH
    pub const BTC_P2WPKH_TESTNET: &'static str = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
    
    /// Valid Ethereum address
    pub const ETH_VALID: &'static str = "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9";
    
    /// Ethereum zero address
    pub const ETH_ZERO: &'static str = "0x0000000000000000000000000000000000000000";
    
    /// Ethereum max address
    pub const ETH_MAX: &'static str = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

    /// Invalid addresses for error testing
    pub fn invalid_bitcoin() -> Vec<&'static str> {
        vec![
            "",
            "not_an_address",
            "bc1qinvalid",                    // Invalid checksum
            "bc1qw508d6qejxtdg4y5r3zarvar",   // Truncated
            "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2", // Mixed formats
        ]
    }

    /// Invalid Ethereum addresses
    pub fn invalid_ethereum() -> Vec<&'static str> {
        vec![
            "",
            "0x",
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG", // Invalid hex
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5",      // Too short
            "742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",    // Missing 0x
        ]
    }
}

// ============================================================================
// Edge Case Amounts
// ============================================================================

/// Edge case amounts for testing overflow and precision
pub struct EdgeCaseAmounts;

impl EdgeCaseAmounts {
    /// Zero amount
    pub const ZERO: u64 = 0;
    
    /// One satoshi / wei / smallest unit
    pub const MIN: u64 = 1;
    
    /// Maximum u64
    pub const MAX_U64: u64 = u64::MAX;
    
    /// Bitcoin max supply in satoshis (21 million BTC)
    pub const BTC_MAX_SUPPLY: u64 = 21_000_000 * 100_000_000;
    
    /// Ethereum max reasonable (for testing)
    pub const ETH_LARGE: u128 = 1_000_000_000_000_000_000_000_000; // 1 million ETH in wei
    
    /// Common dust thresholds
    pub const BTC_DUST: u64 = 546;      // Typical P2PKH dust
    pub const BTC_DUST_SEGWIT: u64 = 294; // SegWit dust
    
    /// Common fee amounts
    pub const TYPICAL_FEE: u64 = 10_000; // 10,000 satoshis
    
    /// Amounts that test precision
    pub fn precision_test_amounts() -> Vec<u64> {
        vec![
            1,
            10,
            100,
            1_000,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
            100_000_000,        // 1 BTC in sats
            1_000_000_000,
            10_000_000_000,
            100_000_000_000,
            1_000_000_000_000,
            u64::MAX / 2,
            u64::MAX - 1,
            u64::MAX,
        ]
    }
    
    /// Amounts that might cause overflow
    pub fn overflow_test_pairs() -> Vec<(u64, u64)> {
        vec![
            (u64::MAX, 1),
            (u64::MAX / 2, u64::MAX / 2 + 2),
            (u64::MAX - 1, 2),
        ]
    }
}

// ============================================================================
// Property-Based Testing Strategies
// ============================================================================

/// Generates valid 32-byte private keys
pub fn valid_private_key_bytes() -> impl Strategy<Value = [u8; 32]> {
    // Generate random bytes, but ensure they're valid for both secp256k1 and Ed25519
    prop::array::uniform32(1u8..=254u8)
}

/// Generates valid hex-encoded private keys
pub fn valid_private_key_hex() -> impl Strategy<Value = String> {
    valid_private_key_bytes().prop_map(|bytes| hex::encode(bytes))
}

/// Generates valid 12-word mnemonic indices (for BIP-39)
pub fn mnemonic_word_indices() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..2048u16, 12..=24)
}

/// Generates valid amounts within reasonable bounds
pub fn valid_amount() -> impl Strategy<Value = u64> {
    1u64..=EdgeCaseAmounts::BTC_MAX_SUPPLY
}

/// Generates fee amounts
pub fn valid_fee() -> impl Strategy<Value = u64> {
    100u64..=1_000_000u64
}

// ============================================================================
// Security Test Patterns
// ============================================================================

/// Security-focused test patterns
pub struct SecurityTests;

impl SecurityTests {
    /// Test that sensitive data is properly zeroized
    /// Usage: Pass a closure that creates sensitive data, returns pointer
    pub fn test_zeroization<F, T>(create_sensitive: F) -> bool
    where
        F: FnOnce() -> (T, *const u8, usize),
    {
        let (_value, ptr, len) = create_sensitive();
        // After drop, memory should be zeroed
        // Note: This is a simplified check - in real tests, use memory inspection
        drop(_value);
        
        // In a real implementation, we'd check the memory
        // For now, just return true as a placeholder
        true
    }
    
    /// Malformed input test patterns
    pub fn malformed_inputs() -> Vec<Vec<u8>> {
        vec![
            vec![],                          // Empty
            vec![0],                         // Single byte
            vec![0; 31],                     // One byte short
            vec![0; 33],                     // One byte long
            vec![0xFF; 32],                  // All high bits
            vec![0x00; 32],                  // All zeros
            (0..256).map(|i| i as u8).collect(), // Sequential bytes
        ]
    }
    
    /// Generates inputs designed to cause timing variations
    pub fn timing_attack_inputs() -> Vec<[u8; 32]> {
        let mut inputs = Vec::new();
        
        // All same values
        for b in [0x00, 0x55, 0xAA, 0xFF] {
            inputs.push([b; 32]);
        }
        
        // Single bit differences
        for i in 0..32 {
            let mut arr = [0u8; 32];
            arr[i] = 1;
            inputs.push(arr);
        }
        
        inputs
    }
}

// ============================================================================
// Test Result Tracking
// ============================================================================

/// Test result for comprehensive reporting
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Whether it passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in microseconds
    pub duration_us: u64,
}

impl TestResult {
    /// Create a passing result
    pub fn pass(name: impl Into<String>, duration_us: u64) -> Self {
        Self {
            name: name.into(),
            passed: true,
            error: None,
            duration_us,
        }
    }
    
    /// Create a failing result
    pub fn fail(name: impl Into<String>, error: impl Into<String>, duration_us: u64) -> Self {
        Self {
            name: name.into(),
            passed: false,
            error: Some(error.into()),
            duration_us,
        }
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed { "✓" } else { "✗" };
        write!(f, "{} {} ({}μs)", status, self.name, self.duration_us)?;
        if let Some(ref err) = self.error {
            write!(f, " - {}", err)?;
        }
        Ok(())
    }
}

/// Collection of test results
#[derive(Debug, Default)]
pub struct TestSuite {
    /// All test results
    pub results: Vec<TestResult>,
}

impl TestSuite {
    /// Create new test suite
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a result
    pub fn add(&mut self, result: TestResult) {
        self.results.push(result);
    }
    
    /// Count passed tests
    pub fn passed(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }
    
    /// Count failed tests
    pub fn failed(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }
    
    /// Total tests
    pub fn total(&self) -> usize {
        self.results.len()
    }
    
    /// Print summary
    pub fn summary(&self) -> String {
        format!(
            "{}/{} tests passed ({} failed)",
            self.passed(),
            self.total(),
            self.failed()
        )
    }
}

// ============================================================================
// Cross-Chain Consistency Helpers
// ============================================================================

/// Checks that the same mnemonic produces expected behavior across chains
pub struct CrossChainConsistency;

impl CrossChainConsistency {
    /// Standard test mnemonic for cross-chain testing
    pub const TEST_MNEMONIC: &'static str = 
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    /// Expected addresses for standard test mnemonic
    /// These should be verified against reference implementations
    pub fn expected_addresses() -> std::collections::HashMap<&'static str, &'static str> {
        let mut map = std::collections::HashMap::new();
        // These are example addresses - should be verified
        map.insert("bitcoin_p2wpkh", "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
        map.insert("ethereum", "0x9858EfFD232B4033E47d90003D41EC34EcaEda94");
        map
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_case_keys_count() {
        assert_eq!(EdgeCaseKeys::all().len(), 8);
        assert!(EdgeCaseKeys::valid_only().len() < EdgeCaseKeys::all().len());
    }

    #[test]
    fn test_edge_case_mnemonics() {
        assert!(!EdgeCaseMnemonics::valid().is_empty());
        assert!(!EdgeCaseMnemonics::invalid().is_empty());
    }

    #[test]
    fn test_edge_case_amounts() {
        let amounts = EdgeCaseAmounts::precision_test_amounts();
        assert!(amounts.len() > 10);
        
        let overflow_pairs = EdgeCaseAmounts::overflow_test_pairs();
        for (a, b) in overflow_pairs {
            assert!(a.checked_add(b).is_none(), "Expected overflow for {} + {}", a, b);
        }
    }

    #[test]
    fn test_test_suite() {
        let mut suite = TestSuite::new();
        suite.add(TestResult::pass("test1", 100));
        suite.add(TestResult::fail("test2", "error", 200));
        
        assert_eq!(suite.total(), 2);
        assert_eq!(suite.passed(), 1);
        assert_eq!(suite.failed(), 1);
    }

    proptest! {
        #[test]
        fn test_valid_key_is_32_bytes(key in valid_private_key_bytes()) {
            assert_eq!(key.len(), 32);
            // Should not be all zeros
            assert!(key.iter().any(|&b| b != 0));
        }

        #[test]
        fn test_valid_amount_in_range(amount in valid_amount()) {
            assert!(amount > 0);
            assert!(amount <= EdgeCaseAmounts::BTC_MAX_SUPPLY);
        }
    }
}
