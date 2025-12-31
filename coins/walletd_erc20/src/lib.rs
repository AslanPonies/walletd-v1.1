//! WalletD ERC‑20 module
//!
//! This crate provides a lightweight abstraction over the ERC‑20 token
//! interface for use with the WalletD SDK.  Each supported token
//! implements the [`Erc20Adapter`] trait defined in the [`adapter`]
//! module.  Tokens expose common operations such as querying balances,
//! approving allowances and transferring funds.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

pub mod adapter;
pub mod usdc;

/// Exposes commonly used types when working with ERC‑20 tokens.
pub mod prelude {
    pub use super::adapter::Erc20Adapter;
    pub use super::usdc::UsdcAdapter;
}
