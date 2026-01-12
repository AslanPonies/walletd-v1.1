//! Ledger hardware wallet support

use async_trait::async_trait;
use crate::{error::*, types::*, HardwareWallet};

const LEDGER_VENDOR_ID: u16 = 0x2c97;

pub struct LedgerDevice {
    device_id: String,
    #[allow(dead_code)]
    handle: Option<hidapi::HidDevice>,
}

impl LedgerDevice {
    pub fn connect(device_id: &str) -> HardwareResult<Self> {
        Ok(Self { device_id: device_id.to_string(), handle: None })
    }

    fn send_apdu(&self, cla: u8, ins: u8, p1: u8, p2: u8, data: &[u8]) -> HardwareResult<Vec<u8>> {
        let mut apdu = vec![cla, ins, p1, p2, data.len() as u8];
        apdu.extend_from_slice(data);
        // In real implementation, send via HID
        Ok(vec![0x90, 0x00]) // Success response
    }
}

pub fn discover_devices() -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    if let Ok(api) = hidapi::HidApi::new() {
        for device in api.device_list() {
            if device.vendor_id() == LEDGER_VENDOR_ID {
                devices.push(DeviceInfo {
                    id: format!("ledger:{:04x}:{:04x}", device.vendor_id(), device.product_id()),
                    device_type: DeviceType::Ledger,
                    model: device.product_string().unwrap_or("Ledger").to_string(),
                    firmware_version: "2.0.0".to_string(),
                    is_initialized: true,
                    has_passphrase: false,
                });
            }
        }
    }
    devices
}

#[async_trait(?Send)]
impl HardwareWallet for LedgerDevice {
    async fn get_info(&self) -> HardwareResult<DeviceInfo> {
        Ok(DeviceInfo {
            id: self.device_id.clone(),
            device_type: DeviceType::Ledger,
            model: "Nano S/X".to_string(),
            firmware_version: "2.0.0".to_string(),
            is_initialized: true,
            has_passphrase: false,
        })
    }

    async fn get_public_key(&self, path: &DerivationPath) -> HardwareResult<Vec<u8>> {
        let path_bytes = path.to_bytes();
        let response = self.send_apdu(0xe0, 0x40, 0x00, 0x00, &path_bytes)?;
        if response.len() < 2 { return Err(HardwareError::InvalidResponse("Short response".into())); }
        Ok(response[..response.len()-2].to_vec())
    }

    async fn get_address(&self, path: &DerivationPath, display: bool) -> HardwareResult<String> {
        let path_bytes = path.to_bytes();
        let p1 = if display { 0x01 } else { 0x00 };
        let response = self.send_apdu(0xe0, 0x40, p1, 0x00, &path_bytes)?;
        Ok(hex::encode(&response[..response.len().saturating_sub(2)]))
    }

    async fn sign_transaction(&self, path: &DerivationPath, tx: &[u8]) -> HardwareResult<Vec<u8>> {
        let mut data = path.to_bytes();
        data.extend_from_slice(tx);
        self.send_apdu(0xe0, 0x04, 0x00, 0x00, &data)
    }

    async fn sign_message(&self, path: &DerivationPath, message: &[u8]) -> HardwareResult<Vec<u8>> {
        let mut data = path.to_bytes();
        data.extend_from_slice(message);
        self.send_apdu(0xe0, 0x08, 0x00, 0x00, &data)
    }
}
