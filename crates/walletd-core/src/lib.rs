use std::future::Future;
use subtle::ConstantTimeEq;

// ============================================================================
// SECURITY MODULE v1.1.0
// Provides constant-time operations to prevent timing attacks
// ============================================================================

/// Constant-time equality comparison for sensitive data.
/// 
/// SECURITY: This function prevents timing attacks by ensuring the comparison
/// takes the same amount of time regardless of where differences occur.
/// 
/// # Example
/// ```
/// use walletd_core::ct_eq;
/// 
/// let secret1 = b"my_secret_key_123";
/// let secret2 = b"my_secret_key_123";
/// let secret3 = b"wrong_secret_key!";
/// 
/// assert!(ct_eq(secret1, secret2));
/// assert!(!ct_eq(secret1, secret3));
/// ```
#[inline]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Constant-time comparison for fixed-size arrays.
/// 
/// SECURITY: Use this for comparing cryptographic keys, MACs, and hashes.
#[inline]
pub fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    a.ct_eq(b).into()
}

/// Constant-time comparison for 64-byte values (e.g., Ed25519 signatures).
#[inline]
pub fn ct_eq_64(a: &[u8; 64], b: &[u8; 64]) -> bool {
    a.ct_eq(b).into()
}

// Re-export zeroize for secure memory cleanup
pub use zeroize::{Zeroize, ZeroizeOnDrop};

// ============================================================================
// CORE TYPES
// ============================================================================

#[derive(Debug)]
pub enum WalletError {
    Custom(String),
    InsufficientFunds,
    WalletNotFound,
}

pub trait Transaction {
    fn get_address(&self) -> String;
    fn to_address(&self) -> String;
    fn amount(&self) -> u64;
}

pub trait BlockchainWallet {
    fn new_wallet(&mut self) -> Result<(), WalletError>;
    fn sync_balance(&mut self) -> impl Future<Output = Result<(), WalletError>>;
}

pub trait CryptoWallet {
    fn generate_address(&mut self) -> Result<String, WalletError>;
    fn balance(&self, address: &str) -> Result<u64, WalletError>;
    fn transfer(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> impl Future<Output = Result<(), WalletError>>;
    fn transaction_history(&self, address: &str) -> Result<Vec<Box<dyn Transaction>>, WalletError>;
}
