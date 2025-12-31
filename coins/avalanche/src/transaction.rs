use alloy::primitives::U256;
use serde::{Deserialize, Serialize};

/// Avalanche C-Chain transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvalancheTransaction {
    pub from: Option<String>,
    pub to: String,
    pub value: U256,
    pub gas_limit: Option<u64>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub nonce: Option<u64>,
    pub data: Option<Vec<u8>>,
    pub chain_id: u64,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl AvalancheTransaction {
    /// Create a simple AVAX transfer transaction
    pub fn transfer(to: &str, value: U256, chain_id: u64) -> Self {
        Self {
            from: None,
            to: to.to_string(),
            value,
            gas_limit: Some(21000),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            data: None,
            chain_id,
        }
    }

    /// Create a contract call transaction
    pub fn contract_call(to: &str, data: Vec<u8>, chain_id: u64) -> Self {
        Self {
            from: None,
            to: to.to_string(),
            value: U256::ZERO,
            gas_limit: Some(200000), // Higher limit for Avalanche contracts
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            data: Some(data),
            chain_id,
        }
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    /// Set EIP-1559 gas parameters (Avalanche uses EIP-1559)
    pub fn with_eip1559_gas(
        mut self,
        max_fee_per_gas: u128,
        max_priority_fee_per_gas: u128,
    ) -> Self {
        self.max_fee_per_gas = Some(max_fee_per_gas);
        self.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set from address
    pub fn with_from(mut self, from: &str) -> Self {
        self.from = Some(from.to_string());
        self
    }

    /// Check if transaction uses EIP-1559
    pub fn is_eip1559(&self) -> bool {
        self.max_fee_per_gas.is_some() && self.max_priority_fee_per_gas.is_some()
    }

    /// Estimate transaction cost in wei
    pub fn estimate_cost(&self) -> U256 {
        let gas_limit = self.gas_limit.unwrap_or(21000);
        let gas_price = self.max_fee_per_gas
            .unwrap_or(25_000_000_000); // 25 nAVAX minimum
        
        let gas_cost = U256::from(gas_limit) * U256::from(gas_price);
        gas_cost + self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVALANCHE_MAINNET: u64 = 43114;

    #[test]
    fn test_transfer_transaction() {
        let tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1_000_000_000_000_000_000u64), // 1 AVAX
            AVALANCHE_MAINNET,
        );
        assert_eq!(tx.chain_id, 43114);
        assert_eq!(tx.gas_limit, Some(21000));
        assert!(tx.data.is_none());
    }

    #[test]
    fn test_contract_call_transaction() {
        let data = vec![0xa9, 0x05, 0x9c, 0xbb]; // transfer selector
        let tx = AvalancheTransaction::contract_call(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            data.clone(),
            AVALANCHE_MAINNET,
        );
        assert_eq!(tx.value, U256::ZERO);
        assert_eq!(tx.gas_limit, Some(200000));
        assert_eq!(tx.data, Some(data));
    }

    #[test]
    fn test_builder_pattern() {
        let tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64),
            AVALANCHE_MAINNET,
        )
        .with_gas_limit(50000)
        .with_nonce(5)
        .with_eip1559_gas(50_000_000_000, 2_000_000_000);

        assert_eq!(tx.gas_limit, Some(50000));
        assert_eq!(tx.nonce, Some(5));
        assert!(tx.is_eip1559());
    }

    #[test]
    fn test_is_eip1559() {
        let non_eip1559_tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64),
            AVALANCHE_MAINNET,
        );

        let eip1559_tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1000u64),
            AVALANCHE_MAINNET,
        )
        .with_eip1559_gas(50_000_000_000, 2_000_000_000);

        assert!(!non_eip1559_tx.is_eip1559());
        assert!(eip1559_tx.is_eip1559());
    }

    #[test]
    fn test_estimate_cost() {
        let tx = AvalancheTransaction::transfer(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f5fFb9",
            U256::from(1_000_000_000_000_000_000u64), // 1 AVAX
            AVALANCHE_MAINNET,
        )
        .with_gas_limit(21000)
        .with_eip1559_gas(50_000_000_000, 2_000_000_000);

        let cost = tx.estimate_cost();
        let expected_gas = U256::from(21000u64) * U256::from(50_000_000_000u64);
        let expected = U256::from(1_000_000_000_000_000_000u64) + expected_gas;
        assert_eq!(cost, expected);
    }
}
