//! # WalletD - Unified Multi-Chain Cryptocurrency Wallet SDK
//!
//! WalletD provides a consistent API for interacting with multiple blockchain networks.
//! Use feature flags to include only the chains you need, keeping binary size minimal.
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `default` | Core traits only |
//! | `bitcoin` | Bitcoin wallet support |
//! | `ethereum` | Ethereum wallet support |
//! | `solana` | Solana wallet support |
//! | `base` | Base L2 wallet support |
//! | `arbitrum` | Arbitrum L2 wallet support |
//! | `erc20` | ERC-20 token support (requires ethereum) |
//! | `icp` | Internet Computer support |
//! | `hedera` | Hedera Hashgraph support |
//! | `monero` | Monero privacy coin support |
//! | `sui` | SUI blockchain support |
//! | `aptos` | Aptos blockchain support |
//! | `ton` | TON (Telegram) blockchain support |
//! | `prasaga` | Prasaga Avio support |
//! | `evm` | All EVM chains (ethereum + base + arbitrum + erc20) |
//! | `move-chains` | Move VM chains (sui + aptos) |
//! | `all-chains` | All supported chains |
//! | `full` | All chains + async runtime + serde |
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! # Just Bitcoin and Ethereum
//! walletd = { version = "0.3", features = ["bitcoin", "ethereum"] }
//!
//! # All EVM chains
//! walletd = { version = "0.3", features = ["evm"] }
//!
//! # Everything
//! walletd = { version = "0.3", features = ["full"] }
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use walletd::prelude::*;
//!
//! #[cfg(feature = "ethereum")]
//! use walletd::ethereum::EthereumWallet;
//!
//! #[cfg(feature = "bitcoin")]
//! use walletd::bitcoin::BitcoinWallet;
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ============================================================================
// Core re-exports (always available with any chain)
// ============================================================================

#[cfg(feature = "core")]
#[cfg_attr(docsrs, doc(cfg(feature = "core")))]
pub use walletd_traits as traits;

#[cfg(feature = "core")]
#[cfg_attr(docsrs, doc(cfg(feature = "core")))]
pub use walletd_core as core;

// ============================================================================
// Chain-specific re-exports
// ============================================================================

/// Bitcoin wallet functionality
#[cfg(feature = "bitcoin")]
#[cfg_attr(docsrs, doc(cfg(feature = "bitcoin")))]
pub mod bitcoin {
    pub use walletd_bitcoin::*;
}

/// Ethereum wallet functionality
#[cfg(feature = "ethereum")]
#[cfg_attr(docsrs, doc(cfg(feature = "ethereum")))]
pub mod ethereum {
    pub use walletd_ethereum::*;
}

/// Solana wallet functionality
#[cfg(feature = "solana")]
#[cfg_attr(docsrs, doc(cfg(feature = "solana")))]
pub mod solana {
    pub use walletd_solana::*;
}

/// Base L2 wallet functionality
#[cfg(feature = "base")]
#[cfg_attr(docsrs, doc(cfg(feature = "base")))]
pub mod base {
    pub use walletd_base::*;
}

/// Arbitrum L2 wallet functionality
#[cfg(feature = "arbitrum")]
#[cfg_attr(docsrs, doc(cfg(feature = "arbitrum")))]
pub mod arbitrum {
    pub use walletd_arbitrum::*;
}

/// ERC-20 token functionality
#[cfg(feature = "erc20")]
#[cfg_attr(docsrs, doc(cfg(feature = "erc20")))]
pub mod erc20 {
    pub use walletd_erc20::*;
}

/// ICP (Internet Computer) wallet functionality
#[cfg(feature = "icp")]
#[cfg_attr(docsrs, doc(cfg(feature = "icp")))]
pub mod icp {
    pub use walletd_icp::*;
}

/// Hedera Hashgraph wallet functionality
#[cfg(feature = "hedera")]
#[cfg_attr(docsrs, doc(cfg(feature = "hedera")))]
pub mod hedera {
    pub use walletd_hedera::*;
}

/// Monero privacy coin functionality
#[cfg(feature = "monero")]
#[cfg_attr(docsrs, doc(cfg(feature = "monero")))]
pub mod monero {
    pub use walletd_monero::*;
}

/// SUI blockchain functionality
#[cfg(feature = "sui")]
#[cfg_attr(docsrs, doc(cfg(feature = "sui")))]
pub mod sui {
    pub use walletd_sui::*;
}

/// Aptos blockchain functionality
#[cfg(feature = "aptos")]
#[cfg_attr(docsrs, doc(cfg(feature = "aptos")))]
pub mod aptos {
    pub use walletd_aptos::*;
}

/// TON (The Open Network) blockchain functionality
#[cfg(feature = "ton")]
#[cfg_attr(docsrs, doc(cfg(feature = "ton")))]
pub mod ton {
    pub use walletd_ton::*;
}

/// Prasaga Avio blockchain functionality
#[cfg(feature = "prasaga")]
#[cfg_attr(docsrs, doc(cfg(feature = "prasaga")))]
pub mod prasaga {
    pub use walletd_prasaga_avio::*;
}

// ============================================================================
// Prelude - commonly used types
// ============================================================================

/// Prelude module for convenient imports
///
/// ```ignore
/// use walletd::prelude::*;
/// ```
pub mod prelude {
    #[cfg(feature = "core")]
    pub use walletd_traits::prelude::*;

    #[cfg(feature = "core")]
    pub use walletd_core::{ct_eq, Zeroize, ZeroizeOnDrop};
}

// ============================================================================
// Version information
// ============================================================================

/// Returns the WalletD SDK version
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns enabled chain features as a slice
pub fn enabled_chains() -> Vec<&'static str> {
    #[allow(unused_mut)]
    let mut chains = Vec::new();
    
    #[cfg(feature = "bitcoin")]
    chains.push("bitcoin");
    
    #[cfg(feature = "ethereum")]
    chains.push("ethereum");
    
    #[cfg(feature = "solana")]
    chains.push("solana");
    
    #[cfg(feature = "base")]
    chains.push("base");
    
    #[cfg(feature = "erc20")]
    chains.push("erc20");
    
    #[cfg(feature = "icp")]
    chains.push("icp");
    
    #[cfg(feature = "hedera")]
    chains.push("hedera");
    
    #[cfg(feature = "monero")]
    chains.push("monero");
    
    #[cfg(feature = "prasaga")]
    chains.push("prasaga");
    
    chains
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
        assert!(v.contains('.'));
    }

    #[test]
    fn test_enabled_chains() {
        let chains = enabled_chains();
        // At minimum, should return empty vec if no chains enabled
        // This test passes regardless of features
        println!("Enabled chains: {:?}", chains);
    }

    #[cfg(feature = "core")]
    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;
        
        // Test that core types are accessible
        let _zero = Amount::zero(18);
    }

    #[cfg(feature = "bitcoin")]
    #[test]
    fn test_bitcoin_import() {
        use crate::bitcoin::BitcoinWallet;
        let _wallet = BitcoinWallet::default();
    }

    #[cfg(feature = "ethereum")]
    #[test]
    fn test_ethereum_import() {
        use crate::ethereum::EthereumWallet;
        // Just verify the type is accessible
        let _ = std::any::type_name::<EthereumWallet>();
    }
}
