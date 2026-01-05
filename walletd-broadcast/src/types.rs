//! Broadcast types and configurations

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Broadcast configuration
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Network mode
    pub network: NetworkMode,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Enable automatic fee estimation
    pub auto_fee: bool,
    /// Custom RPC endpoints per chain
    pub custom_endpoints: std::collections::HashMap<String, Vec<String>>,
    /// API keys for providers
    pub api_keys: ApiKeys,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            network: NetworkMode::Mainnet,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            auto_fee: true,
            custom_endpoints: std::collections::HashMap::new(),
            api_keys: ApiKeys::default(),
        }
    }
}

impl BroadcastConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            network: NetworkMode::Mainnet,
            ..Default::default()
        }
    }

    /// Create testnet configuration
    pub fn testnet() -> Self {
        Self {
            network: NetworkMode::Testnet,
            ..Default::default()
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Add custom endpoint for a chain
    pub fn with_endpoint(mut self, chain: &str, endpoint: &str) -> Self {
        self.custom_endpoints
            .entry(chain.to_string())
            .or_default()
            .push(endpoint.to_string());
        self
    }
}

/// Network mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    Mainnet,
    Testnet,
}

/// API keys for various providers
#[derive(Debug, Clone, Default)]
pub struct ApiKeys {
    pub infura: Option<String>,
    pub alchemy: Option<String>,
    pub etherscan: Option<String>,
    pub blockstream: Option<String>,
    pub hedera: Option<String>,
    pub solana_rpc: Option<String>,
}

/// Broadcast response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResponse {
    /// Transaction hash
    pub tx_hash: String,
    /// Chain identifier
    pub chain: String,
    /// Timestamp of broadcast
    pub timestamp: u64,
    /// Provider used
    pub provider: String,
    /// Estimated confirmation time in seconds
    pub estimated_confirmation: Option<u64>,
    /// Fee paid
    pub fee: Option<u64>,
}

impl BroadcastResponse {
    /// Create a new broadcast response
    pub fn new(tx_hash: String, chain: &str, provider: &str) -> Self {
        Self {
            tx_hash,
            chain: chain.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            provider: provider.to_string(),
            estimated_confirmation: None,
            fee: None,
        }
    }

    /// Set estimated confirmation time
    pub fn with_confirmation_time(mut self, seconds: u64) -> Self {
        self.estimated_confirmation = Some(seconds);
        self
    }

    /// Set fee
    pub fn with_fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    /// Transaction hash
    pub tx_hash: String,
    /// Current status
    pub status: TxStatus,
    /// Number of confirmations
    pub confirmations: u64,
    /// Block number (if confirmed)
    pub block_number: Option<u64>,
    /// Block hash (if confirmed)
    pub block_hash: Option<String>,
    /// Timestamp
    pub timestamp: Option<u64>,
    /// Fee paid
    pub fee: Option<u64>,
    /// Gas used (for EVM chains)
    pub gas_used: Option<u64>,
}

/// Transaction status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    /// Transaction is pending in mempool
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction was dropped from mempool
    Dropped,
    /// Transaction replaced (RBF)
    Replaced,
    /// Status unknown
    Unknown,
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStatus::Pending => write!(f, "Pending"),
            TxStatus::Confirmed => write!(f, "Confirmed"),
            TxStatus::Failed => write!(f, "Failed"),
            TxStatus::Dropped => write!(f, "Dropped"),
            TxStatus::Replaced => write!(f, "Replaced"),
            TxStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    /// Slow (economy) fee
    pub slow: u64,
    /// Medium (standard) fee  
    pub medium: u64,
    /// Fast (priority) fee
    pub fast: u64,
    /// Unit (satoshi/byte, gwei, lamports, etc)
    pub unit: String,
}

impl FeeEstimate {
    /// Get fee for priority level
    pub fn for_priority(&self, priority: FeePriority) -> u64 {
        match priority {
            FeePriority::Slow => self.slow,
            FeePriority::Medium => self.medium,
            FeePriority::Fast => self.fast,
        }
    }
}

/// Fee priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeePriority {
    Slow,
    Medium,
    Fast,
}
