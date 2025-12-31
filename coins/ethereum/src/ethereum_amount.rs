use crate::Error;
use alloy::primitives::U256;
use std::ops;

/// Contains a field representing the amount of wei in the amount. Also has functions to convert to and from the main unit (ETH) and the smallest unit (wei).
#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct EthereumAmount {
    /// The number of wei (U256) in the amount
    pub wei: U256,
}

impl ops::Add<Self> for EthereumAmount {
    type Output = Result<Self, Error>;

    fn add(self, rhs: Self) -> Result<Self, Error> {
        Ok(Self {
            wei: self
                .wei
                .checked_add(rhs.wei)
                .ok_or(Error::Overflow(format!(
                    "Overflow in U256 when adding {} to {}",
                    self.wei, rhs.wei
                )))?,
        })
    }
}

impl ops::Sub for EthereumAmount {
    type Output = Result<Self, Error>;

    fn sub(self, rhs: Self) -> Result<Self, Error> {
        Ok(Self {
            wei: self
                .wei
                .checked_sub(rhs.wei)
                .ok_or(Error::Overflow(format!(
                    "Overflow in U256 when subtracting {} from {}",
                    self.wei, rhs.wei
                )))?,
        })
    }
}

impl ops::Mul<u64> for EthereumAmount {
    type Output = Result<Self, Error>;

    fn mul(self, rhs: u64) -> Self::Output {
        let result = self.wei.checked_mul(U256::from(rhs));
        
        match result {
            Some(val) => Ok(Self { wei: val }),
            None => Err(Error::Overflow(format!(
                "Overflow in U256 when multiplying {} by {}",
                self.wei, rhs
            ))),
        }
    }
}

impl EthereumAmount {
    /// Creates a new EthereumAmount from a decimal value in ETH
    /// 
    /// SECURITY FIX v1.1.0 (M-05): Safe conversion to prevent overflow
    /// Note: For very large ETH amounts (>18 quintillion), consider using from_wei directly.
    pub fn from_eth(eth_amount: f64) -> Self {
        // Use U256 directly to handle large values safely
        // 10^18 wei per ethereum
        let wei_f64 = eth_amount * 1e18;
        
        // Clamp to valid range and convert safely
        if wei_f64 < 0.0 {
            return Self { wei: U256::ZERO };
        }
        
        // For values within u128 range, use safe conversion
        if wei_f64 <= u128::MAX as f64 {
            let wei_u128 = wei_f64.round() as u128;
            Self { wei: U256::from(wei_u128) }
        } else {
            // For extremely large values, saturate to max U256
            // This is a safety measure; in practice, such large values are unrealistic
            Self { wei: U256::MAX }
        }
    }

    /// Creates a new EthereumAmount from a decimal value in ETH
    pub fn eth(&self) -> f64 {
        // SECURITY FIX v1.1.0 (M-05): Safe conversion for display
        // Note: For very large amounts, precision may be lost in f64 representation
        // Convert to u128 safely by taking lower limbs
        let bytes = self.wei.to_le_bytes::<32>();
        let low_u128 = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        low_u128 as f64 / 1e18 // 10^18 wei per ethereum
    }

    /// Returns the number of wei in the amount
    pub fn wei(&self) -> U256 {
        self.wei
    }

    /// Returns the amount in gwei (10^9 wei)
    pub fn gwei(&self) -> f64 {
        let bytes = self.wei.to_le_bytes::<32>();
        let low_u128 = u128::from_le_bytes(bytes[0..16].try_into().unwrap());
        low_u128 as f64 / 1e9 // 10^9 wei per gwei
    }

    /// Creates a new zero EthereumAmount
    pub fn zero() -> Self {
        Self { wei: U256::ZERO }
    }

    /// Creates a new EthereumAmount from gwei (10^9 wei)
    pub fn from_gwei(gwei_amount: u64) -> Self {
        let wei = U256::from(gwei_amount) * U256::from(1_000_000_000u64);
        Self { wei }
    }

    /// Creates a new EthereumAmount from the wei amount (U256)
    pub fn from_wei(wei_amount: U256) -> Self {
        Self { wei: wei_amount }
    }

    /// Creates a new EthereumAmount from the wei amount (u128)
    /// Convenience method for smaller values
    pub fn from_wei_u128(wei_amount: u128) -> Self {
        Self { wei: U256::from(wei_amount) }
    }
    /// Creates a new EthereumAmount from the eth amount
    pub fn from_main_unit_decimal_value(value: f64) -> Self {
        Self::from_eth(value)
    }
    /// Creates a new EthereumAmount from the wei amount
    pub fn from_smallest_unit_integer_value(value: u64) -> Self {
        Self::from_wei(U256::from(value))
    }
    /// Returns the number of eth in the amount
    pub fn to_main_unit_decimal_value(&self) -> f64 {
        self.eth()
    }
    /// Returns the number of wei in the amount
    pub fn to_smallest_unit_integer_value(&self) -> u64 {
        // Safe conversion - will truncate if value is too large
        let bytes = self.wei.to_le_bytes::<32>();
        u64::from_le_bytes(bytes[0..8].try_into().unwrap())
    }
}
