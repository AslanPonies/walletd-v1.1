//! WalletD Staking - Multi-chain Staking Support
//!
//! Unified staking interface for ETH 2.0, Solana, Polkadot, Cosmos, Cardano

pub mod ethereum;
pub mod solana;
pub mod polkadot;
pub mod cosmos;
pub mod cardano;
pub mod error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
pub use error::{StakingError, StakingResult};

/// Unified staking trait
#[async_trait]
pub trait Staking: Send + Sync {
    /// Get staking info for an address
    async fn get_stake_info(&self, address: &str) -> StakingResult<StakeInfo>;
    
    /// Get available validators
    async fn get_validators(&self, limit: usize) -> StakingResult<Vec<ValidatorInfo>>;
    
    /// Create stake transaction
    async fn stake(&self, amount: u128, validator: &str) -> StakingResult<Vec<u8>>;
    
    /// Create unstake transaction
    async fn unstake(&self, amount: u128, validator: &str) -> StakingResult<Vec<u8>>;
    
    /// Claim rewards
    async fn claim_rewards(&self) -> StakingResult<Vec<u8>>;
    
    /// Get estimated APY
    async fn get_apy(&self) -> StakingResult<f64>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub total_staked: u128,
    pub available_balance: u64,
    pub pending_rewards: u64,
    pub delegations: Vec<Delegation>,
    pub unbonding: Vec<UnbondingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub validator: String,
    pub amount: u64,
    pub rewards: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingEntry {
    pub validator: String,
    pub amount: u64,
    pub completion_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub name: String,
    pub commission: f64,
    pub total_stake: u128,
    pub delegators: u64,
    pub uptime: f64,
    pub apy: f64,
    pub is_active: bool,
}

/// Multi-chain staking manager
pub struct StakingManager {
    ethereum: ethereum::EthereumStaking,
    solana: solana::SolanaStaking,
    polkadot: polkadot::PolkadotStaking,
    cosmos: cosmos::CosmosStaking,
    cardano: cardano::CardanoStaking,
}

impl StakingManager {
    pub fn new() -> Self {
        Self {
            ethereum: ethereum::EthereumStaking::new(),
            solana: solana::SolanaStaking::new(),
            polkadot: polkadot::PolkadotStaking::new(),
            cosmos: cosmos::CosmosStaking::new(),
            cardano: cardano::CardanoStaking::new(),
        }
    }

    pub fn for_chain(&self, chain: StakingChain) -> &dyn Staking {
        match chain {
            StakingChain::Ethereum => &self.ethereum,
            StakingChain::Solana => &self.solana,
            StakingChain::Polkadot => &self.polkadot,
            StakingChain::Cosmos => &self.cosmos,
            StakingChain::Cardano => &self.cardano,
        }
    }
}

impl Default for StakingManager {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy)]
pub enum StakingChain { Ethereum, Solana, Polkadot, Cosmos, Cardano }
