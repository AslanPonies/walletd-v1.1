//! WalletD Multi-Signature - Enterprise Multi-sig Support
//!
//! Supports M-of-N multi-signature wallets for Bitcoin, Ethereum, and other chains.

pub mod bitcoin_multisig;
pub mod ethereum_multisig;
pub mod policy;
pub mod error;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use error::{MultisigError, MultisigResult};

/// Multi-signature wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigConfig {
    /// Required signatures (M)
    pub threshold: u8,
    /// Total signers (N)
    pub total_signers: u8,
    /// Public keys of all signers
    pub signers: Vec<SignerInfo>,
    /// Chain type
    pub chain: MultisigChain,
    /// Optional time-lock
    pub timelock: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    pub id: String,
    pub public_key: Vec<u8>,
    pub weight: u8,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultisigChain {
    Bitcoin,
    Ethereum,
    Solana,
    Cosmos,
}

/// Multi-signature wallet
pub struct MultisigWallet {
    config: MultisigConfig,
    pending_txs: HashMap<String, PendingTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub id: String,
    pub tx_data: Vec<u8>,
    pub signatures: Vec<PartialSignature>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSignature {
    pub signer_id: String,
    pub signature: Vec<u8>,
    pub signed_at: u64,
}

impl MultisigWallet {
    /// Create new multisig wallet
    pub fn new(config: MultisigConfig) -> MultisigResult<Self> {
        if config.threshold > config.total_signers {
            return Err(MultisigError::InvalidThreshold);
        }
        if config.signers.len() != config.total_signers as usize {
            return Err(MultisigError::InvalidSignerCount);
        }
        Ok(Self { config, pending_txs: HashMap::new() })
    }

    /// Get wallet address
    pub fn address(&self) -> MultisigResult<String> {
        match self.config.chain {
            MultisigChain::Bitcoin => bitcoin_multisig::derive_address(&self.config),
            MultisigChain::Ethereum => ethereum_multisig::derive_address(&self.config),
            _ => Err(MultisigError::UnsupportedChain),
        }
    }

    /// Create unsigned transaction
    pub fn create_transaction(&mut self, tx_data: Vec<u8>) -> MultisigResult<String> {
        let id = hex::encode(sha2::Sha256::digest(&tx_data));
        let pending = PendingTransaction {
            id: id.clone(),
            tx_data,
            signatures: Vec::new(),
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            expires_at: None,
        };
        self.pending_txs.insert(id.clone(), pending);
        Ok(id)
    }

    /// Add signature to pending transaction
    pub fn add_signature(&mut self, tx_id: &str, signer_id: &str, signature: Vec<u8>) -> MultisigResult<()> {
        let pending = self.pending_txs.get_mut(tx_id).ok_or(MultisigError::TransactionNotFound)?;
        
        // Verify signer is authorized
        if !self.config.signers.iter().any(|s| s.id == signer_id) {
            return Err(MultisigError::UnauthorizedSigner);
        }
        
        // Check for duplicate
        if pending.signatures.iter().any(|s| s.signer_id == signer_id) {
            return Err(MultisigError::DuplicateSignature);
        }

        pending.signatures.push(PartialSignature {
            signer_id: signer_id.to_string(),
            signature,
            signed_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        });
        Ok(())
    }

    /// Check if transaction has enough signatures
    pub fn is_ready(&self, tx_id: &str) -> bool {
        self.pending_txs.get(tx_id)
            .map(|tx| tx.signatures.len() >= self.config.threshold as usize)
            .unwrap_or(false)
    }

    /// Finalize transaction with all signatures
    pub fn finalize(&self, tx_id: &str) -> MultisigResult<Vec<u8>> {
        let pending = self.pending_txs.get(tx_id).ok_or(MultisigError::TransactionNotFound)?;
        
        if pending.signatures.len() < self.config.threshold as usize {
            return Err(MultisigError::InsufficientSignatures);
        }

        match self.config.chain {
            MultisigChain::Bitcoin => bitcoin_multisig::finalize(pending, &self.config),
            MultisigChain::Ethereum => ethereum_multisig::finalize(pending, &self.config),
            _ => Err(MultisigError::UnsupportedChain),
        }
    }
}

use sha2::Digest;
