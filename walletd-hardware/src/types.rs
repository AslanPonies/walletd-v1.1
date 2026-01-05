use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub device_type: DeviceType,
    pub model: String,
    pub firmware_version: String,
    pub is_initialized: bool,
    pub has_passphrase: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType { Ledger, Trezor }

#[derive(Debug, Clone)]
pub struct DerivationPath {
    pub purpose: u32,
    pub coin_type: u32,
    pub account: u32,
    pub change: u32,
    pub address_index: u32,
}

impl DerivationPath {
    pub fn bip44(coin_type: u32, account: u32, change: u32, index: u32) -> Self {
        Self { purpose: 44, coin_type, account, change, address_index: index }
    }
    
    pub fn bip84(account: u32, change: u32, index: u32) -> Self {
        Self { purpose: 84, coin_type: 0, account, change, address_index: index }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(20);
        for component in [self.purpose | 0x80000000, self.coin_type | 0x80000000, 
                         self.account | 0x80000000, self.change, self.address_index] {
            bytes.extend_from_slice(&component.to_be_bytes());
        }
        bytes
    }
}

impl std::fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m/{}'/{}'/{}'/{}'/{}",
            self.purpose, self.coin_type, self.account, self.change, self.address_index)
    }
}
