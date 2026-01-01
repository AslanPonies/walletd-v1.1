//! CLI Types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum CliResponse {
    Continue,
    Exit,
    Swap,
    ChangeMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletMode {
    Testnet,
    Mainnet,
    Demo,
}

impl Default for WalletMode {
    fn default() -> Self { Self::Testnet }
}
