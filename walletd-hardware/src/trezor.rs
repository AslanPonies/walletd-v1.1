//! Trezor hardware wallet support

use async_trait::async_trait;
use crate::{error::*, types::*, HardwareWallet};

const TREZOR_VENDOR_ID: u16 = 0x534c;

pub struct TrezorDevice {
    device_id: String,
}

impl TrezorDevice {
    pub fn connect(device_id: &str) -> HardwareResult<Self> {
        Ok(Self { device_id: device_id.to_string() })
    }
}

pub fn discover_devices() -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    if let Ok(api) = hidapi::HidApi::new() {
        for device in api.device_list() {
            if device.vendor_id() == TREZOR_VENDOR_ID {
                devices.push(DeviceInfo {
                    id: format!("trezor:{:04x}:{:04x}", device.vendor_id(), device.product_id()),
                    device_type: DeviceType::Trezor,
                    model: device.product_string().unwrap_or("Trezor").to_string(),
                    firmware_version: "2.5.0".to_string(),
                    is_initialized: true,
                    has_passphrase: false,
                });
            }
        }
    }
    devices
}

#[async_trait(?Send)]
impl HardwareWallet for TrezorDevice {
    async fn get_info(&self) -> HardwareResult<DeviceInfo> {
        Ok(DeviceInfo {
            id: self.device_id.clone(),
            device_type: DeviceType::Trezor,
            model: "Model T".to_string(),
            firmware_version: "2.5.0".to_string(),
            is_initialized: true,
            has_passphrase: false,
        })
    }

    async fn get_public_key(&self, _path: &DerivationPath) -> HardwareResult<Vec<u8>> {
        Ok(vec![0x04; 65]) // Placeholder uncompressed pubkey
    }

    async fn get_address(&self, path: &DerivationPath, _display: bool) -> HardwareResult<String> {
        Ok(format!("trezor_addr_{}", path))
    }

    async fn sign_transaction(&self, _path: &DerivationPath, _tx: &[u8]) -> HardwareResult<Vec<u8>> {
        Ok(vec![0; 64]) // Placeholder signature
    }

    async fn sign_message(&self, _path: &DerivationPath, _message: &[u8]) -> HardwareResult<Vec<u8>> {
        Ok(vec![0; 64])
    }
}
