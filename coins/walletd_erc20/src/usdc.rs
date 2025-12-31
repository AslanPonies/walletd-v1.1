//! USDC token adapter

use crate::adapter::{Erc20Adapter, Erc20Error};
use alloy::primitives::{address, Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use alloy::sol_types::SolCall;
use async_trait::async_trait;

/// USDC contract address on Ethereum mainnet
pub const USDC_MAINNET: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

// Define ERC20 function selectors using sol!
sol! {
    function name() external view returns (string);
    function symbol() external view returns (string);
    function decimals() external view returns (uint8);
    function totalSupply() external view returns (uint256);
    function balanceOf(address account) external view returns (uint256);
    function allowance(address owner, address spender) external view returns (uint256);
}

/// USDC adapter for interacting with the USDC token contract
#[derive(Debug, Clone)]
pub struct UsdcAdapter {
    /// The contract address
    address: Address,
}

impl Default for UsdcAdapter {
    fn default() -> Self {
        Self {
            address: USDC_MAINNET,
        }
    }
}

impl UsdcAdapter {
    /// Create a new USDC adapter with a custom address
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    /// Create a USDC adapter for mainnet
    pub fn mainnet() -> Self {
        Self::default()
    }

    async fn call_contract<C: SolCall>(&self, rpc_url: &str, call: C) -> Result<C::Return, Erc20Error> {
        let provider = ProviderBuilder::new()
            .connect_http(rpc_url.parse().map_err(|e| Erc20Error::ProviderError(format!("{e}")))?);
        
        let call_data = call.abi_encode();
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(self.address)
            .input(call_data.into());
        
        let result = provider.call(tx).await
            .map_err(|e| Erc20Error::ContractError(format!("{e}")))?;
        
        C::abi_decode_returns(&result)
            .map_err(|e| Erc20Error::ContractError(format!("Decode error: {e}")))
    }
}

#[async_trait]
impl Erc20Adapter for UsdcAdapter {
    fn contract_address(&self) -> Address {
        self.address
    }

    async fn name(&self, rpc_url: &str) -> Result<String, Erc20Error> {
        self.call_contract(rpc_url, nameCall {}).await
    }

    async fn symbol(&self, rpc_url: &str) -> Result<String, Erc20Error> {
        self.call_contract(rpc_url, symbolCall {}).await
    }

    async fn decimals(&self, rpc_url: &str) -> Result<u8, Erc20Error> {
        self.call_contract(rpc_url, decimalsCall {}).await
    }

    async fn total_supply(&self, rpc_url: &str) -> Result<U256, Erc20Error> {
        self.call_contract(rpc_url, totalSupplyCall {}).await
    }

    async fn balance_of(&self, rpc_url: &str, owner: Address) -> Result<U256, Erc20Error> {
        self.call_contract(rpc_url, balanceOfCall { account: owner }).await
    }

    async fn allowance(&self, rpc_url: &str, owner: Address, spender: Address) -> Result<U256, Erc20Error> {
        self.call_contract(rpc_url, allowanceCall { owner, spender }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // USDC Address Constants Tests
    // ============================================================================

    #[test]
    fn test_usdc_mainnet_address() {
        // USDC mainnet address should be the well-known address
        let expected = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(USDC_MAINNET, expected);
    }

    #[test]
    fn test_usdc_mainnet_address_format() {
        // Verify address can be formatted to string
        let addr_str = format!("{}", USDC_MAINNET);
        // Address should be lowercase hex with 0x prefix
        assert!(addr_str.starts_with("0x") || addr_str.starts_with("0X"));
        assert_eq!(addr_str.len(), 42); // 0x + 40 hex chars
    }

    // ============================================================================
    // UsdcAdapter Construction Tests
    // ============================================================================

    #[test]
    fn test_usdc_adapter_default() {
        let adapter = UsdcAdapter::default();
        assert_eq!(adapter.contract_address(), USDC_MAINNET);
    }

    #[test]
    fn test_usdc_adapter_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        assert_eq!(adapter.contract_address(), USDC_MAINNET);
    }

    #[test]
    fn test_usdc_adapter_custom_address() {
        let custom_address = address!("dAC17F958D2ee523a2206206994597C13D831ec7"); // USDT address
        let adapter = UsdcAdapter::new(custom_address);
        assert_eq!(adapter.contract_address(), custom_address);
    }

    #[test]
    fn test_usdc_adapter_clone() {
        let adapter1 = UsdcAdapter::mainnet();
        let adapter2 = adapter1.clone();
        assert_eq!(adapter1.contract_address(), adapter2.contract_address());
    }

    #[test]
    fn test_usdc_adapter_debug() {
        let adapter = UsdcAdapter::mainnet();
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("UsdcAdapter"));
        assert!(debug_str.contains("address"));
    }

    // ============================================================================
    // ERC20 Call Encoding Tests
    // ============================================================================

    #[test]
    fn test_name_call_encoding() {
        let call = nameCall {};
        let encoded = call.abi_encode();
        // name() function selector is 0x06fdde03
        assert_eq!(&encoded[0..4], &[0x06, 0xfd, 0xde, 0x03]);
    }

    #[test]
    fn test_symbol_call_encoding() {
        let call = symbolCall {};
        let encoded = call.abi_encode();
        // symbol() function selector is 0x95d89b41
        assert_eq!(&encoded[0..4], &[0x95, 0xd8, 0x9b, 0x41]);
    }

    #[test]
    fn test_decimals_call_encoding() {
        let call = decimalsCall {};
        let encoded = call.abi_encode();
        // decimals() function selector is 0x313ce567
        assert_eq!(&encoded[0..4], &[0x31, 0x3c, 0xe5, 0x67]);
    }

    #[test]
    fn test_total_supply_call_encoding() {
        let call = totalSupplyCall {};
        let encoded = call.abi_encode();
        // totalSupply() function selector is 0x18160ddd
        assert_eq!(&encoded[0..4], &[0x18, 0x16, 0x0d, 0xdd]);
    }

    #[test]
    fn test_balance_of_call_encoding() {
        let account = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let call = balanceOfCall { account };
        let encoded = call.abi_encode();
        
        // balanceOf(address) function selector is 0x70a08231
        assert_eq!(&encoded[0..4], &[0x70, 0xa0, 0x82, 0x31]);
        
        // Encoded data should be 36 bytes (4 selector + 32 address)
        assert_eq!(encoded.len(), 36);
    }

    #[test]
    fn test_allowance_call_encoding() {
        let owner = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        let spender = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        let call = allowanceCall { owner, spender };
        let encoded = call.abi_encode();
        
        // allowance(address,address) function selector is 0xdd62ed3e
        assert_eq!(&encoded[0..4], &[0xdd, 0x62, 0xed, 0x3e]);
        
        // Encoded data should be 68 bytes (4 selector + 32 owner + 32 spender)
        assert_eq!(encoded.len(), 68);
    }

    // ============================================================================
    // Erc20Adapter Trait Tests
    // ============================================================================

    #[test]
    fn test_usdc_implements_erc20_adapter() {
        // This test verifies UsdcAdapter implements the Erc20Adapter trait
        fn assert_erc20_adapter<T: Erc20Adapter>(_: &T) {}
        let adapter = UsdcAdapter::mainnet();
        assert_erc20_adapter(&adapter);
    }

    #[test]
    fn test_usdc_adapter_is_send_sync() {
        // ERC20 adapters must be Send + Sync for async usage
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UsdcAdapter>();
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[tokio::test]
    async fn test_invalid_rpc_url_error() {
        let adapter = UsdcAdapter::mainnet();
        let result = adapter.name("not-a-valid-url").await;
        assert!(result.is_err());
        
        if let Err(Erc20Error::ProviderError(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected ProviderError");
        }
    }

    #[tokio::test]
    async fn test_unreachable_rpc_error() {
        let adapter = UsdcAdapter::mainnet();
        // Use a valid URL format but unreachable endpoint
        let result = adapter.decimals("http://127.0.0.1:59999").await;
        assert!(result.is_err());
    }
}

// ============================================================================
// Integration Tests (require network access)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    const MAINNET_RPC: &str = "https://eth.llamarpc.com";
    
    // Vitalik's address - known to have tokens
    const VITALIK: Address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_name_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        let name = adapter.name(MAINNET_RPC).await.expect("Failed to get name");
        assert_eq!(name, "USD Coin");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_symbol_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        let symbol = adapter.symbol(MAINNET_RPC).await.expect("Failed to get symbol");
        assert_eq!(symbol, "USDC");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_decimals_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        let decimals = adapter.decimals(MAINNET_RPC).await.expect("Failed to get decimals");
        assert_eq!(decimals, 6); // USDC has 6 decimals
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_total_supply_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        let total_supply = adapter.total_supply(MAINNET_RPC).await.expect("Failed to get total supply");
        // USDC total supply should be > 0
        assert!(total_supply > U256::ZERO);
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_balance_of_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        // Just verify the call works, balance may be 0
        let _balance = adapter.balance_of(MAINNET_RPC, VITALIK).await
            .expect("Failed to get balance");
    }

    #[tokio::test]
    #[ignore = "Requires network access"]
    async fn test_usdc_allowance_mainnet() {
        let adapter = UsdcAdapter::mainnet();
        let spender = address!("0000000000000000000000000000000000000001");
        // Just verify the call works
        let allowance = adapter.allowance(MAINNET_RPC, VITALIK, spender).await
            .expect("Failed to get allowance");
        // Allowance to random address should be 0
        assert_eq!(allowance, U256::ZERO);
    }
}
