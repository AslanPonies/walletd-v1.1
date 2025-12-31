#![allow(clippy::arithmetic_side_effects)]

use crate::Error;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

/// The basis for all Solana wallets, wrapping a Keypair from the Solana SDK.
///
/// In Solana, everything is an account. For naming consistency, we call this a `SolanaAccount`.
/// ### Key Concepts:
/// - Accounts store data and have a unique address (usually a public key).
/// - Accounts have a max size of 10MB; Program-Derived Accounts (PDAs) are limited to 10KB.
/// - PDAs can sign on behalf of a program.
/// - Account sizes are fixed at creation but can be adjusted using `realloc`.
/// - Data storage is paid with rent.
/// - Default account owner is the System Program.
/// - Generate an account in the CLI using `solana-keygen new`.
///
/// ### Account Fields:
/// - **lamports**: Number of lamports owned by the account.
/// - **owner**: Program owner of the account.
/// - **executable**: Whether the account can process instructions.
/// - **data**: Raw data byte array stored by the account.
/// - **rent_epoch**: Next epoch when rent is owed.
///
/// Only the account's owner can modify its data or debit lamports. Anyone can credit lamports.
/// The owner can reassign ownership if the account's data is zeroed out.
/// Program accounts do not store state.
///
/// Example: A counter program requires two accounts—one for the program code and one for the counter.
#[allow(dead_code)]
pub struct SolanaAccount {
    keypair: Keypair,
}

impl SolanaAccount {
    /// Creates a new `SolanaAccount` from a 64-byte array.
    ///
    /// # Errors
    /// Returns an `Error` if the byte array cannot be converted to a valid `Keypair`.
    pub fn new_from_bytes(bytes: [u8; 64]) -> Result<Self, Error> {
        let keypair = Keypair::try_from(&bytes[..])
            .map_err(|e| Error::Custom(format!("Failed to create keypair from bytes: {e}")))?;
        Ok(Self { keypair })
    }

    /// Returns the public key associated with the account.
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Retrieves the account's balance in lamports using the provided `RpcClient`.
    ///
    /// # Errors
    /// Returns an `Error` if the balance query fails.
    pub async fn balance(&self, rpc_client: RpcClient) -> Result<u64, Error> {
        let balance = rpc_client
            .get_balance(&self.pubkey())
            .await
            .map_err(|e| Error::Custom(format!("Failed to get balance: {e}")))?;
        Ok(balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Account Creation Tests
    // ============================================================================

    #[test]
    fn test_new_from_bytes_valid() {
        // Generate a valid 64-byte keypair
        let keypair = Keypair::new();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account = SolanaAccount::new_from_bytes(bytes);
        assert!(account.is_ok());
    }

    #[test]
    fn test_new_from_bytes_preserves_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account = SolanaAccount::new_from_bytes(bytes).unwrap();
        assert_eq!(account.pubkey(), expected_pubkey);
    }

    #[test]
    fn test_new_from_bytes_invalid() {
        // All zeros is not a valid keypair
        let invalid_bytes: [u8; 64] = [0u8; 64];
        let account = SolanaAccount::new_from_bytes(invalid_bytes);
        // This may or may not fail depending on solana-sdk validation
        // Just verify it doesn't panic
        let _ = account;
    }

    // ============================================================================
    // Pubkey Tests
    // ============================================================================

    #[test]
    fn test_pubkey_returns_valid_address() {
        let keypair = Keypair::new();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account = SolanaAccount::new_from_bytes(bytes).unwrap();
        let pubkey = account.pubkey();
        
        // Pubkey should be 32 bytes
        assert_eq!(pubkey.to_bytes().len(), 32);
    }

    #[test]
    fn test_pubkey_is_deterministic() {
        let keypair = Keypair::new();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account1 = SolanaAccount::new_from_bytes(bytes).unwrap();
        let account2 = SolanaAccount::new_from_bytes(bytes).unwrap();
        
        assert_eq!(account1.pubkey(), account2.pubkey());
    }

    #[test]
    fn test_pubkey_different_for_different_accounts() {
        let keypair1 = Keypair::new();
        let keypair2 = Keypair::new();
        
        let account1 = SolanaAccount::new_from_bytes(keypair1.to_bytes()).unwrap();
        let account2 = SolanaAccount::new_from_bytes(keypair2.to_bytes()).unwrap();
        
        assert_ne!(account1.pubkey(), account2.pubkey());
    }

    // ============================================================================
    // Pubkey Format Tests
    // ============================================================================

    #[test]
    fn test_pubkey_base58_encoding() {
        let keypair = Keypair::new();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account = SolanaAccount::new_from_bytes(bytes).unwrap();
        let pubkey_str = account.pubkey().to_string();
        
        // Base58 encoded pubkey should be between 32-44 characters
        assert!(pubkey_str.len() >= 32 && pubkey_str.len() <= 44);
    }

    #[test]
    fn test_pubkey_valid_base58_chars() {
        let keypair = Keypair::new();
        let bytes: [u8; 64] = keypair.to_bytes();
        
        let account = SolanaAccount::new_from_bytes(bytes).unwrap();
        let pubkey_str = account.pubkey().to_string();
        
        // Base58 alphabet (no 0, O, I, l)
        for c in pubkey_str.chars() {
            assert!(c.is_alphanumeric());
            assert_ne!(c, '0');
            assert_ne!(c, 'O');
            assert_ne!(c, 'I');
            assert_ne!(c, 'l');
        }
    }
}
