//! WalletD Broadcast - Multi-chain transaction broadcasting
//!
//! This crate provides a unified interface for broadcasting transactions
//! across 17+ blockchain networks with automatic retry, fallback providers,
//! and comprehensive error handling.

pub mod chains;
pub mod providers;
pub mod error;
pub mod types;

use async_trait::async_trait;
use std::time::Duration;

pub use error::{BroadcastError, BroadcastResult};
pub use types::{BroadcastConfig, BroadcastResponse, TransactionStatus};

/// Unified transaction broadcaster trait
#[async_trait]
pub trait TransactionBroadcaster: Send + Sync {
    /// Broadcast a signed transaction
    async fn broadcast(&self, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse>;
    
    /// Check transaction status
    async fn get_status(&self, tx_hash: &str) -> BroadcastResult<TransactionStatus>;
    
    /// Estimate transaction fee
    async fn estimate_fee(&self, tx_size: usize) -> BroadcastResult<u64>;
    
    /// Get current network status
    async fn network_status(&self) -> BroadcastResult<NetworkStatus>;
}

/// Network status information
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub is_healthy: bool,
    pub block_height: u64,
    pub avg_block_time: Duration,
    pub mempool_size: Option<u64>,
    pub suggested_fee: u64,
}

/// Multi-chain broadcaster with automatic failover
pub struct MultiBroadcaster {
    config: BroadcastConfig,
    bitcoin: chains::BitcoinBroadcaster,
    ethereum: chains::EthereumBroadcaster,
    solana: chains::SolanaBroadcaster,
    hedera: chains::HederaBroadcaster,
    monero: chains::MoneroBroadcaster,
    icp: chains::IcpBroadcaster,
    base: chains::EvmBroadcaster,
    polygon: chains::EvmBroadcaster,
    avalanche: chains::EvmBroadcaster,
    arbitrum: chains::EvmBroadcaster,
    cardano: chains::CardanoBroadcaster,
    cosmos: chains::CosmosBroadcaster,
    polkadot: chains::SubstrateBroadcaster,
    near: chains::NearBroadcaster,
    tron: chains::TronBroadcaster,
    sui: chains::SuiBroadcaster,
    aptos: chains::AptosBroadcaster,
    ton: chains::TonBroadcaster,
}

impl MultiBroadcaster {
    /// Create a new multi-chain broadcaster
    pub fn new(config: BroadcastConfig) -> Self {
        Self {
            bitcoin: chains::BitcoinBroadcaster::new(&config),
            ethereum: chains::EthereumBroadcaster::new(&config),
            solana: chains::SolanaBroadcaster::new(&config),
            hedera: chains::HederaBroadcaster::new(&config),
            monero: chains::MoneroBroadcaster::new(&config),
            icp: chains::IcpBroadcaster::new(&config),
            base: chains::EvmBroadcaster::new_base(&config),
            polygon: chains::EvmBroadcaster::new_polygon(&config),
            avalanche: chains::EvmBroadcaster::new_avalanche(&config),
            arbitrum: chains::EvmBroadcaster::new_arbitrum(&config),
            cardano: chains::CardanoBroadcaster::new(&config),
            cosmos: chains::CosmosBroadcaster::new(&config),
            polkadot: chains::SubstrateBroadcaster::new_polkadot(&config),
            near: chains::NearBroadcaster::new(&config),
            tron: chains::TronBroadcaster::new(&config),
            sui: chains::SuiBroadcaster::new(&config),
            aptos: chains::AptosBroadcaster::new(&config),
            ton: chains::TonBroadcaster::new(&config),
            config,
        }
    }

    /// Broadcast to a specific chain
    pub async fn broadcast_to(&self, chain: Chain, signed_tx: &[u8]) -> BroadcastResult<BroadcastResponse> {
        match chain {
            Chain::Bitcoin => self.bitcoin.broadcast(signed_tx).await,
            Chain::Ethereum => self.ethereum.broadcast(signed_tx).await,
            Chain::Solana => self.solana.broadcast(signed_tx).await,
            Chain::Hedera => self.hedera.broadcast(signed_tx).await,
            Chain::Monero => self.monero.broadcast(signed_tx).await,
            Chain::Icp => self.icp.broadcast(signed_tx).await,
            Chain::Base => self.base.broadcast(signed_tx).await,
            Chain::Polygon => self.polygon.broadcast(signed_tx).await,
            Chain::Avalanche => self.avalanche.broadcast(signed_tx).await,
            Chain::Arbitrum => self.arbitrum.broadcast(signed_tx).await,
            Chain::Cardano => self.cardano.broadcast(signed_tx).await,
            Chain::Cosmos => self.cosmos.broadcast(signed_tx).await,
            Chain::Polkadot => self.polkadot.broadcast(signed_tx).await,
            Chain::Near => self.near.broadcast(signed_tx).await,
            Chain::Tron => self.tron.broadcast(signed_tx).await,
            Chain::Sui => self.sui.broadcast(signed_tx).await,
            Chain::Aptos => self.aptos.broadcast(signed_tx).await,
            Chain::Ton => self.ton.broadcast(signed_tx).await,
        }
    }

    /// Get transaction status on a specific chain
    pub async fn get_status(&self, chain: Chain, tx_hash: &str) -> BroadcastResult<TransactionStatus> {
        match chain {
            Chain::Bitcoin => self.bitcoin.get_status(tx_hash).await,
            Chain::Ethereum => self.ethereum.get_status(tx_hash).await,
            Chain::Solana => self.solana.get_status(tx_hash).await,
            Chain::Hedera => self.hedera.get_status(tx_hash).await,
            Chain::Monero => self.monero.get_status(tx_hash).await,
            Chain::Icp => self.icp.get_status(tx_hash).await,
            Chain::Base => self.base.get_status(tx_hash).await,
            Chain::Polygon => self.polygon.get_status(tx_hash).await,
            Chain::Avalanche => self.avalanche.get_status(tx_hash).await,
            Chain::Arbitrum => self.arbitrum.get_status(tx_hash).await,
            Chain::Cardano => self.cardano.get_status(tx_hash).await,
            Chain::Cosmos => self.cosmos.get_status(tx_hash).await,
            Chain::Polkadot => self.polkadot.get_status(tx_hash).await,
            Chain::Near => self.near.get_status(tx_hash).await,
            Chain::Tron => self.tron.get_status(tx_hash).await,
            Chain::Sui => self.sui.get_status(tx_hash).await,
            Chain::Aptos => self.aptos.get_status(tx_hash).await,
            Chain::Ton => self.ton.get_status(tx_hash).await,
        }
    }
}

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Solana,
    Hedera,
    Monero,
    Icp,
    Base,
    Polygon,
    Avalanche,
    Arbitrum,
    Cardano,
    Cosmos,
    Polkadot,
    Near,
    Tron,
    Sui,
    Aptos,
    Ton,
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Bitcoin => write!(f, "Bitcoin"),
            Chain::Ethereum => write!(f, "Ethereum"),
            Chain::Solana => write!(f, "Solana"),
            Chain::Hedera => write!(f, "Hedera"),
            Chain::Monero => write!(f, "Monero"),
            Chain::Icp => write!(f, "Internet Computer"),
            Chain::Base => write!(f, "Base"),
            Chain::Polygon => write!(f, "Polygon"),
            Chain::Avalanche => write!(f, "Avalanche"),
            Chain::Arbitrum => write!(f, "Arbitrum"),
            Chain::Cardano => write!(f, "Cardano"),
            Chain::Cosmos => write!(f, "Cosmos"),
            Chain::Polkadot => write!(f, "Polkadot"),
            Chain::Near => write!(f, "Near"),
            Chain::Tron => write!(f, "Tron"),
            Chain::Sui => write!(f, "Sui"),
            Chain::Aptos => write!(f, "Aptos"),
            Chain::Ton => write!(f, "TON"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_display() {
        assert_eq!(Chain::Bitcoin.to_string(), "Bitcoin");
        assert_eq!(Chain::Ethereum.to_string(), "Ethereum");
    }
}
