//! Multi-chain ERC-20 token registry
//!
//! Provides a unified interface for ERC-20 tokens across all EVM chains.

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// EVM Chain identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvmChain {
    Ethereum = 1,
    Polygon = 137,
    Avalanche = 43114,
    Base = 8453,
    Arbitrum = 42161,
    Optimism = 10,
}

impl EvmChain {
    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        *self as u64
    }

    /// Get chain name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Polygon => "Polygon",
            Self::Avalanche => "Avalanche",
            Self::Base => "Base",
            Self::Arbitrum => "Arbitrum",
            Self::Optimism => "Optimism",
        }
    }

    /// Get native token symbol
    pub fn native_symbol(&self) -> &'static str {
        match self {
            Self::Ethereum => "ETH",
            Self::Polygon => "POL",
            Self::Avalanche => "AVAX",
            Self::Base => "ETH",
            Self::Arbitrum => "ETH",
            Self::Optimism => "ETH",
        }
    }

    /// Get default RPC endpoint
    pub fn default_rpc(&self) -> &'static str {
        match self {
            Self::Ethereum => "https://eth.llamarpc.com",
            Self::Polygon => "https://polygon-rpc.com",
            Self::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
            Self::Base => "https://mainnet.base.org",
            Self::Arbitrum => "https://arb1.arbitrum.io/rpc",
            Self::Optimism => "https://mainnet.optimism.io",
        }
    }
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token symbol (e.g., "USDC")
    pub symbol: String,
    /// Token name (e.g., "USD Coin")
    pub name: String,
    /// Decimal places
    pub decimals: u8,
    /// Contract address
    pub address: String,
    /// Logo URL (optional)
    pub logo_url: Option<String>,
    /// Coingecko ID for price data
    pub coingecko_id: Option<String>,
}

impl TokenInfo {
    /// Create a new token info
    pub fn new(symbol: &str, name: &str, decimals: u8, address: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            name: name.to_string(),
            decimals,
            address: address.to_string(),
            logo_url: None,
            coingecko_id: None,
        }
    }

    /// Get contract address as Address type
    pub fn contract_address(&self) -> Option<Address> {
        Address::from_str(&self.address).ok()
    }
}

/// Multi-chain token registry
#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    tokens: HashMap<EvmChain, HashMap<String, TokenInfo>>,
}

impl TokenRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// Create registry with common tokens pre-loaded
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.load_defaults();
        registry
    }

    /// Load default token list
    pub fn load_defaults(&mut self) {
        // ========== USDC ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"
        ));
        self.add_token(EvmChain::Avalanche, TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E"
        ));
        self.add_token(EvmChain::Base, TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
        ));
        self.add_token(EvmChain::Arbitrum, TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
        ));

        // ========== USDT ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "USDT", "Tether USD", 6,
            "0xdAC17F958D2ee523a2206206994597C13D831ec7"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "USDT", "Tether USD", 6,
            "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"
        ));
        self.add_token(EvmChain::Avalanche, TokenInfo::new(
            "USDT", "Tether USD", 6,
            "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7"
        ));
        self.add_token(EvmChain::Arbitrum, TokenInfo::new(
            "USDT", "Tether USD", 6,
            "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"
        ));

        // ========== WETH ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "WETH", "Wrapped Ether", 18,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "WETH", "Wrapped Ether", 18,
            "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619"
        ));
        self.add_token(EvmChain::Avalanche, TokenInfo::new(
            "WETH", "Wrapped Ether", 18,
            "0x49D5c2BdFfac6CE2BFdB6640F4F80f226bc10bAB"
        ));
        self.add_token(EvmChain::Base, TokenInfo::new(
            "WETH", "Wrapped Ether", 18,
            "0x4200000000000000000000000000000000000006"
        ));
        self.add_token(EvmChain::Arbitrum, TokenInfo::new(
            "WETH", "Wrapped Ether", 18,
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
        ));

        // ========== DAI ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "DAI", "Dai Stablecoin", 18,
            "0x6B175474E89094C44Da98b954EescdeCB5BadAc0d"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "DAI", "Dai Stablecoin", 18,
            "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"
        ));

        // ========== WBTC ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "WBTC", "Wrapped Bitcoin", 8,
            "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "WBTC", "Wrapped Bitcoin", 8,
            "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6"
        ));
        self.add_token(EvmChain::Avalanche, TokenInfo::new(
            "WBTC", "Wrapped Bitcoin", 8,
            "0x50b7545627a5162F82A992c33b87aDc75187B218"
        ));
        self.add_token(EvmChain::Arbitrum, TokenInfo::new(
            "WBTC", "Wrapped Bitcoin", 8,
            "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f"
        ));

        // ========== LINK ==========
        self.add_token(EvmChain::Ethereum, TokenInfo::new(
            "LINK", "Chainlink", 18,
            "0x514910771AF9Ca656af840dff83E8264EcF986CA"
        ));
        self.add_token(EvmChain::Polygon, TokenInfo::new(
            "LINK", "Chainlink", 18,
            "0x53E0bca35eC356BD5ddDFebbD1Fc0fD03FaBad39"
        ));
        self.add_token(EvmChain::Avalanche, TokenInfo::new(
            "LINK", "Chainlink", 18,
            "0x5947BB275c521040051D82396192181b413227A3"
        ));
    }

    /// Add a token to the registry
    pub fn add_token(&mut self, chain: EvmChain, token: TokenInfo) {
        let chain_tokens = self.tokens.entry(chain).or_insert_with(HashMap::new);
        chain_tokens.insert(token.symbol.clone(), token);
    }

    /// Get a token by symbol and chain
    pub fn get(&self, chain: EvmChain, symbol: &str) -> Option<&TokenInfo> {
        self.tokens.get(&chain)?.get(symbol)
    }

    /// Get all tokens for a chain
    pub fn tokens_for_chain(&self, chain: EvmChain) -> Vec<&TokenInfo> {
        self.tokens
            .get(&chain)
            .map(|t| t.values().collect())
            .unwrap_or_default()
    }

    /// Get all chains that have a token
    pub fn chains_for_token(&self, symbol: &str) -> Vec<EvmChain> {
        self.tokens
            .iter()
            .filter_map(|(chain, tokens)| {
                if tokens.contains_key(symbol) {
                    Some(*chain)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all unique token symbols
    pub fn all_symbols(&self) -> Vec<String> {
        let mut symbols: Vec<String> = self.tokens
            .values()
            .flat_map(|t| t.keys().cloned())
            .collect();
        symbols.sort();
        symbols.dedup();
        symbols
    }

    /// Get token count for a chain
    pub fn token_count(&self, chain: EvmChain) -> usize {
        self.tokens.get(&chain).map(|t| t.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_chain_ids() {
        assert_eq!(EvmChain::Ethereum.chain_id(), 1);
        assert_eq!(EvmChain::Polygon.chain_id(), 137);
        assert_eq!(EvmChain::Avalanche.chain_id(), 43114);
        assert_eq!(EvmChain::Base.chain_id(), 8453);
        assert_eq!(EvmChain::Arbitrum.chain_id(), 42161);
    }

    #[test]
    fn test_evm_chain_names() {
        assert_eq!(EvmChain::Ethereum.name(), "Ethereum");
        assert_eq!(EvmChain::Polygon.name(), "Polygon");
    }

    #[test]
    fn test_evm_chain_native_symbols() {
        assert_eq!(EvmChain::Ethereum.native_symbol(), "ETH");
        assert_eq!(EvmChain::Polygon.native_symbol(), "POL");
        assert_eq!(EvmChain::Avalanche.native_symbol(), "AVAX");
    }

    #[test]
    fn test_token_info_creation() {
        let token = TokenInfo::new("USDC", "USD Coin", 6, "0x123");
        assert_eq!(token.symbol, "USDC");
        assert_eq!(token.name, "USD Coin");
        assert_eq!(token.decimals, 6);
    }

    #[test]
    fn test_registry_with_defaults() {
        let registry = TokenRegistry::with_defaults();
        
        // Check USDC exists on multiple chains
        assert!(registry.get(EvmChain::Ethereum, "USDC").is_some());
        assert!(registry.get(EvmChain::Polygon, "USDC").is_some());
        assert!(registry.get(EvmChain::Avalanche, "USDC").is_some());
    }

    #[test]
    fn test_registry_tokens_for_chain() {
        let registry = TokenRegistry::with_defaults();
        let eth_tokens = registry.tokens_for_chain(EvmChain::Ethereum);
        assert!(!eth_tokens.is_empty());
    }

    #[test]
    fn test_registry_chains_for_token() {
        let registry = TokenRegistry::with_defaults();
        let usdc_chains = registry.chains_for_token("USDC");
        assert!(usdc_chains.contains(&EvmChain::Ethereum));
        assert!(usdc_chains.contains(&EvmChain::Polygon));
    }

    #[test]
    fn test_registry_all_symbols() {
        let registry = TokenRegistry::with_defaults();
        let symbols = registry.all_symbols();
        assert!(symbols.contains(&"USDC".to_string()));
        assert!(symbols.contains(&"USDT".to_string()));
        assert!(symbols.contains(&"WETH".to_string()));
    }

    #[test]
    fn test_registry_add_custom_token() {
        let mut registry = TokenRegistry::new();
        registry.add_token(
            EvmChain::Ethereum,
            TokenInfo::new("CUSTOM", "Custom Token", 18, "0xabc"),
        );
        assert!(registry.get(EvmChain::Ethereum, "CUSTOM").is_some());
    }

    #[test]
    fn test_registry_token_count() {
        let registry = TokenRegistry::with_defaults();
        assert!(registry.token_count(EvmChain::Ethereum) > 0);
    }

    #[test]
    fn test_token_contract_address() {
        let token = TokenInfo::new(
            "USDC", "USD Coin", 6,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert!(token.contract_address().is_some());
    }

    #[test]
    fn test_default_rpc_endpoints() {
        assert!(!EvmChain::Ethereum.default_rpc().is_empty());
        assert!(!EvmChain::Polygon.default_rpc().is_empty());
    }
}
