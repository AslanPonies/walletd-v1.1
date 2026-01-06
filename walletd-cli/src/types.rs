//! Shared types for WalletD CLI

use serde::{Deserialize, Serialize};

/// Wallet operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletMode {
    Testnet,
    Mainnet,
    Demo,
}

impl WalletMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletMode::Testnet => "Testnet",
            WalletMode::Mainnet => "Mainnet",
            WalletMode::Demo => "Demo",
        }
    }
    
    pub fn is_testnet(&self) -> bool {
        matches!(self, WalletMode::Testnet)
    }
    
    pub fn is_mainnet(&self) -> bool {
        matches!(self, WalletMode::Mainnet)
    }
    
    pub fn is_demo(&self) -> bool {
        matches!(self, WalletMode::Demo)
    }
}

impl Default for WalletMode {
    fn default() -> Self {
        WalletMode::Testnet
    }
}

/// Supported blockchain chains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chain {
    // Original chains (1-9)
    Bitcoin,
    Ethereum,
    Solana,
    Hedera,
    Monero,
    Icp,
    Erc20,
    Base,
    Prasaga,
    // Extended chains (10-20)
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

impl Chain {
    pub fn menu_number(&self) -> u8 {
        match self {
            Chain::Bitcoin => 1,
            Chain::Ethereum => 2,
            Chain::Solana => 3,
            Chain::Hedera => 4,
            Chain::Monero => 5,
            Chain::Icp => 6,
            Chain::Erc20 => 7,
            Chain::Base => 8,
            Chain::Prasaga => 9,
            Chain::Polygon => 10,
            Chain::Avalanche => 11,
            Chain::Arbitrum => 12,
            Chain::Cardano => 13,
            Chain::Cosmos => 14,
            Chain::Polkadot => 15,
            Chain::Near => 16,
            Chain::Tron => 17,
            Chain::Sui => 18,
            Chain::Aptos => 19,
            Chain::Ton => 20,
        }
    }
    
    pub fn from_menu_number(n: u8) -> Option<Chain> {
        match n {
            1 => Some(Chain::Bitcoin),
            2 => Some(Chain::Ethereum),
            3 => Some(Chain::Solana),
            4 => Some(Chain::Hedera),
            5 => Some(Chain::Monero),
            6 => Some(Chain::Icp),
            7 => Some(Chain::Erc20),
            8 => Some(Chain::Base),
            9 => Some(Chain::Prasaga),
            10 => Some(Chain::Polygon),
            11 => Some(Chain::Avalanche),
            12 => Some(Chain::Arbitrum),
            13 => Some(Chain::Cardano),
            14 => Some(Chain::Cosmos),
            15 => Some(Chain::Polkadot),
            16 => Some(Chain::Near),
            17 => Some(Chain::Tron),
            18 => Some(Chain::Sui),
            19 => Some(Chain::Aptos),
            20 => Some(Chain::Ton),
            _ => None,
        }
    }
    
    pub fn symbol(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "BTC",
            Chain::Ethereum => "ETH",
            Chain::Solana => "SOL",
            Chain::Hedera => "HBAR",
            Chain::Monero => "XMR",
            Chain::Icp => "ICP",
            Chain::Erc20 => "ERC20",
            Chain::Base => "BASE",
            Chain::Prasaga => "PRA",
            Chain::Polygon => "POL",
            Chain::Avalanche => "AVAX",
            Chain::Arbitrum => "ARB",
            Chain::Cardano => "ADA",
            Chain::Cosmos => "ATOM",
            Chain::Polkadot => "DOT",
            Chain::Near => "NEAR",
            Chain::Tron => "TRX",
            Chain::Sui => "SUI",
            Chain::Aptos => "APT",
            Chain::Ton => "TON",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "Bitcoin",
            Chain::Ethereum => "Ethereum",
            Chain::Solana => "Solana",
            Chain::Hedera => "Hedera",
            Chain::Monero => "Monero",
            Chain::Icp => "Internet Computer",
            Chain::Erc20 => "ERC-20 Tokens",
            Chain::Base => "Base L2",
            Chain::Prasaga => "Prasaga",
            Chain::Polygon => "Polygon",
            Chain::Avalanche => "Avalanche",
            Chain::Arbitrum => "Arbitrum",
            Chain::Cardano => "Cardano",
            Chain::Cosmos => "Cosmos",
            Chain::Polkadot => "Polkadot",
            Chain::Near => "NEAR Protocol",
            Chain::Tron => "Tron",
            Chain::Sui => "Sui",
            Chain::Aptos => "Aptos",
            Chain::Ton => "TON",
        }
    }
    
    pub fn all() -> &'static [Chain] {
        &[
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Solana,
            Chain::Hedera,
            Chain::Monero,
            Chain::Icp,
            Chain::Erc20,
            Chain::Base,
            Chain::Prasaga,
            Chain::Polygon,
            Chain::Avalanche,
            Chain::Arbitrum,
            Chain::Cardano,
            Chain::Cosmos,
            Chain::Polkadot,
            Chain::Near,
            Chain::Tron,
            Chain::Sui,
            Chain::Aptos,
            Chain::Ton,
        ]
    }
    
    pub fn original_chains() -> &'static [Chain] {
        &[
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Solana,
            Chain::Hedera,
            Chain::Monero,
            Chain::Icp,
            Chain::Erc20,
            Chain::Base,
            Chain::Prasaga,
        ]
    }
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub chain: String,
    pub address: String,
    pub balance: String,
    pub created_at: String,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Confirmed => "Confirmed",
            TransactionStatus::Failed => "Failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_mode_default() {
        assert_eq!(WalletMode::default(), WalletMode::Testnet);
    }
    
    #[test]
    fn test_chain_menu_numbers() {
        assert_eq!(Chain::Bitcoin.menu_number(), 1);
        assert_eq!(Chain::Ton.menu_number(), 20);
    }
    
    #[test]
    fn test_chain_from_menu_number() {
        assert_eq!(Chain::from_menu_number(1), Some(Chain::Bitcoin));
        assert_eq!(Chain::from_menu_number(20), Some(Chain::Ton));
        assert_eq!(Chain::from_menu_number(99), None);
    }
    
    #[test]
    fn test_chain_symbols() {
        assert_eq!(Chain::Bitcoin.symbol(), "BTC");
        assert_eq!(Chain::Ethereum.symbol(), "ETH");
    }
}
