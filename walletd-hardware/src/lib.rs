//! WalletD Hardware - Ledger & Trezor Support
//!
//! Enterprise-grade hardware wallet integration for secure key management.

pub mod ledger;
pub mod trezor;
pub mod error;
pub mod types;

use async_trait::async_trait;
pub use error::{HardwareError, HardwareResult};
pub use types::*;

/// Hardware wallet trait
#[async_trait]
pub trait HardwareWallet: Send + Sync {
    /// Get device info
    async fn get_info(&self) -> HardwareResult<DeviceInfo>;
    
    /// Get public key at derivation path
    async fn get_public_key(&self, path: &DerivationPath) -> HardwareResult<Vec<u8>>;
    
    /// Get address at derivation path
    async fn get_address(&self, path: &DerivationPath, display: bool) -> HardwareResult<String>;
    
    /// Sign transaction
    async fn sign_transaction(&self, path: &DerivationPath, tx: &[u8]) -> HardwareResult<Vec<u8>>;
    
    /// Sign message
    async fn sign_message(&self, path: &DerivationPath, message: &[u8]) -> HardwareResult<Vec<u8>>;
}

/// Device discovery
pub struct DeviceManager {
    ledger_enabled: bool,
    trezor_enabled: bool,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self { ledger_enabled: true, trezor_enabled: true }
    }

    /// List connected hardware wallets
    pub fn list_devices(&self) -> Vec<DeviceInfo> {
        let mut devices = Vec::new();
        if self.ledger_enabled {
            devices.extend(ledger::discover_devices());
        }
        if self.trezor_enabled {
            devices.extend(trezor::discover_devices());
        }
        devices
    }

    /// Connect to a specific device
    pub fn connect(&self, device_id: &str) -> HardwareResult<Box<dyn HardwareWallet>> {
        if device_id.starts_with("ledger:") {
            Ok(Box::new(ledger::LedgerDevice::connect(device_id)?))
        } else if device_id.starts_with("trezor:") {
            Ok(Box::new(trezor::TrezorDevice::connect(device_id)?))
        } else {
            Err(HardwareError::DeviceNotFound(device_id.to_string()))
        }
    }
}

impl Default for DeviceManager {
    fn default() -> Self { Self::new() }
}
