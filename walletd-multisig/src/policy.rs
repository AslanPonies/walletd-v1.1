//! Spending policy and approval workflows

use serde::{Deserialize, Serialize};

/// Spending policy for multisig wallets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingPolicy {
    pub daily_limit: Option<u64>,
    pub per_tx_limit: Option<u64>,
    pub whitelist: Vec<String>,
    pub require_2fa: bool,
    pub time_delay: Option<u64>,
}

impl Default for SpendingPolicy {
    fn default() -> Self {
        Self {
            daily_limit: None,
            per_tx_limit: None,
            whitelist: Vec::new(),
            require_2fa: false,
            time_delay: None,
        }
    }
}

impl SpendingPolicy {
    pub fn enterprise() -> Self {
        Self {
            daily_limit: Some(100_000_000_000), // 1000 units
            per_tx_limit: Some(10_000_000_000),
            whitelist: Vec::new(),
            require_2fa: true,
            time_delay: Some(3600), // 1 hour delay
        }
    }

    pub fn validate_transaction(&self, amount: u64, recipient: &str) -> bool {
        if let Some(limit) = self.per_tx_limit {
            if amount > limit { return false; }
        }
        if !self.whitelist.is_empty() && !self.whitelist.contains(&recipient.to_string()) {
            return false;
        }
        true
    }
}
