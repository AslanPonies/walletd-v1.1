#![allow(clippy::arithmetic_side_effects)]

use crate::Error;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;

/// A client for interacting with the Solana blockchain via an RPC endpoint.
#[allow(dead_code)]
pub struct SolanaClient {
    rpc_client: RpcClient,
    endpoint: String,
    commitment_level: CommitmentConfig,
}

impl SolanaClient {
    /// Creates a new `SolanaClient` with the default commitment level (`confirmed`).
    ///
    /// # Errors
    /// Returns an `Error` if the endpoint is invalid or the transport fails to connect.
    pub async fn new(endpoint: &str) -> Result<Self, Error> {
        let rpc_client = RpcClient::new(endpoint.to_string());
        Ok(Self {
            rpc_client,
            endpoint: endpoint.to_string(),
            commitment_level: CommitmentConfig::confirmed(),
        })
    }

    /// Creates a new `SolanaClient` with a specified commitment level.
    ///
    /// Valid commitment levels are:
    /// - `CommitmentLevel::Processed`
    /// - `CommitmentLevel::Finalized`
    /// - `CommitmentLevel::Confirmed`
    ///
    /// # Errors
    /// Returns an `Error` if the endpoint or commitment configuration is invalid.
    pub async fn new_with_commitment(
        endpoint: &str,
        commitment: CommitmentConfig,
    ) -> Result<Self, Error> {
        let rpc_client = RpcClient::new_with_commitment(endpoint.to_string(), commitment);
        Ok(Self {
            rpc_client,
            endpoint: endpoint.to_string(),
            commitment_level: commitment,
        })
    }

    /// Returns the underlying `RpcClient`.
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Returns the current endpoint URL.
    #[allow(dead_code)]
    fn url(&self) -> &str {
        &self.endpoint
    }

    /// Returns the current commitment level.
    pub fn commitment_level(&self) -> &CommitmentConfig {
        &self.commitment_level
    }

    /// Gets the SOL balance for a specific pubkey address in lamports.
    ///
    /// # Errors
    /// Returns an `Error` if the balance query fails.
    pub async fn get_balance(&self, address: &Pubkey) -> Result<u64, Error> {
        let balance = self
            .rpc_client
            .get_balance(address)
            .await
            .map_err(|e| Error::Custom(format!("Failed to get balance: {e}")))?;
        Ok(balance)
    }

    /// Requests an airdrop of 1 SOL to a given address (devnet only).
    ///
    /// # Errors
    /// Returns an `Error` if the airdrop request or confirmation fails.
    pub async fn request_airdrop(&self, public_address: Pubkey) -> Result<String, Error> {
        let sig = self
            .rpc_client
            .request_airdrop(&public_address, 1_000_000_000)
            .await
            .map_err(|e| Error::Custom(format!("Failed to request airdrop: {e}")))?;

        let confirmed = self
            .rpc_client
            .confirm_transaction(&sig)
            .await
            .map_err(|e| Error::Custom(format!("Failed to confirm airdrop: {e}")))?;

        if confirmed {
            Ok(format!("Transaction: {sig} Status: {confirmed}"))
        } else {
            Err(Error::Custom(format!(
                "Airdrop transaction {sig} not confirmed"
            )))
        }
    }

    /// Retrieves account details for a given pubkey.
    ///
    /// # Errors
    /// Returns an `Error` if the account query fails.
    pub async fn get_account(&self, address: &Pubkey) -> Result<Account, Error> {
        let account = self
            .rpc_client
            .get_account(address)
            .await
            .map_err(|e| Error::Custom(format!("Failed to get account: {e}")))?;
        Ok(account)
    }

    /// Retrieves program accounts for a given pubkey.
    ///
    /// # Errors
    /// Returns an `Error` if the program accounts query fails.
    pub async fn get_program_accounts(
        &self,
        address: &Pubkey,
    ) -> Result<Vec<(Pubkey, Account)>, Error> {
        let accounts = self
            .rpc_client
            .get_program_accounts(address)
            .await
            .map_err(|e| Error::Custom(format!("Failed to get program accounts: {e}")))?;
        Ok(accounts)
    }

    /// Transfers SOL to a specified pubkey.
    ///
    /// # Errors
    /// Returns an `Error` if the transfer or confirmation fails.
    pub async fn transfer(
        &self,
        from_keypair: Keypair,
        to_pubkey: Pubkey,
        lamports: u64,
    ) -> Result<bool, Error> {
        let from_pubkey = from_keypair.pubkey();
        let ix = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

        let recent_blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .await
            .map_err(|e| Error::Custom(format!("Failed to get latest blockhash: {e}")))?;

        let txn = Transaction::new_signed_with_payer(
            &[ix],
            Some(&from_pubkey),
            &[&from_keypair],
            recent_blockhash,
        );

        let sig = self
            .rpc_client
            .send_and_confirm_transaction(&txn)
            .await
            .map_err(|e| Error::Custom(format!("Failed to send transaction: {e}")))?;

        let confirmed = self
            .rpc_client
            .confirm_transaction(&sig)
            .await
            .map_err(|e| Error::Custom(format!("Failed to confirm transaction: {e}")))?;

        if confirmed {
            println!("Transaction: {sig} Status: {confirmed}");
            Ok(true)
        } else {
            println!("Transaction {sig} not confirmed");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ============================================================================
    // SolanaClient Construction Tests
    // ============================================================================

    #[tokio::test]
    async fn test_new_client_devnet() {
        let client = SolanaClient::new("https://api.devnet.solana.com").await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_new_client_mainnet() {
        let client = SolanaClient::new("https://api.mainnet-beta.solana.com").await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_new_client_with_commitment_finalized() {
        let client = SolanaClient::new_with_commitment(
            "https://api.devnet.solana.com",
            CommitmentConfig::finalized()
        ).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_new_client_with_commitment_processed() {
        let client = SolanaClient::new_with_commitment(
            "https://api.devnet.solana.com",
            CommitmentConfig::processed()
        ).await;
        assert!(client.is_ok());
    }

    // ============================================================================
    // Commitment Level Tests
    // ============================================================================

    #[tokio::test]
    async fn test_commitment_level_default() {
        let client = SolanaClient::new("https://api.devnet.solana.com").await.unwrap();
        assert_eq!(*client.commitment_level(), CommitmentConfig::confirmed());
    }

    #[tokio::test]
    async fn test_commitment_level_finalized() {
        let client = SolanaClient::new_with_commitment(
            "https://api.devnet.solana.com",
            CommitmentConfig::finalized()
        ).await.unwrap();
        assert_eq!(*client.commitment_level(), CommitmentConfig::finalized());
    }

    // ============================================================================
    // Pubkey Tests
    // ============================================================================

    #[test]
    fn test_pubkey_from_string() {
        let pubkey_str = "11111111111111111111111111111111";
        let pubkey = Pubkey::from_str(pubkey_str);
        assert!(pubkey.is_ok());
    }

    #[test]
    fn test_system_program_pubkey() {
        // System program address
        let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        assert_eq!(system_program.to_string(), "11111111111111111111111111111111");
    }

    #[test]
    fn test_invalid_pubkey() {
        let result = Pubkey::from_str("invalid-pubkey");
        assert!(result.is_err());
    }

    // ============================================================================
    // Keypair Tests
    // ============================================================================

    #[test]
    fn test_keypair_generation() {
        let keypair = Keypair::new();
        // Keypair should have a valid pubkey
        let pubkey = keypair.pubkey();
        assert_eq!(pubkey.to_string().len(), 44); // Base58 encoded pubkey is 44 chars
    }

    #[test]
    fn test_keypair_signer() {
        let keypair = Keypair::new();
        // Test that keypair implements Signer trait
        let _pubkey = Signer::pubkey(&keypair);
    }

    // ============================================================================
    // RPC Client Reference Tests
    // ============================================================================

    #[tokio::test]
    async fn test_rpc_client_reference() {
        let client = SolanaClient::new("https://api.devnet.solana.com").await.unwrap();
        let rpc = client.rpc_client();
        // Just verify we can get a reference to the RPC client
        assert!(!rpc.url().is_empty());
    }
}

// ============================================================================
// Integration Tests (require network access)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::str::FromStr;

    const DEVNET_URL: &str = "https://api.devnet.solana.com";

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_balance_devnet() {
        let client = SolanaClient::new(DEVNET_URL).await.unwrap();
        
        // System program always exists
        let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let balance = client.get_balance(&system_program).await;
        
        // System program should have some balance
        assert!(balance.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_get_account_devnet() {
        let client = SolanaClient::new(DEVNET_URL).await.unwrap();
        
        // Token program account
        let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        let account = client.get_account(&token_program).await;
        
        assert!(account.is_ok());
        let acc = account.unwrap();
        assert!(acc.executable); // Token program is executable
    }

    #[tokio::test]
    #[ignore = "Requires network access and funded wallet"]
    async fn test_request_airdrop_devnet() {
        let client = SolanaClient::new(DEVNET_URL).await.unwrap();
        let keypair = Keypair::new();
        
        let result = client.request_airdrop(keypair.pubkey()).await;
        // This may fail due to rate limiting, so just check it doesn't panic
        println!("Airdrop result: {:?}", result);
    }
}
