use core::fmt;
use core::fmt::Display;
use std::ops;

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct MoneroAmount {
    piconero: u64,
}

impl Serialize for MoneroAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MoneroAmount", 1)?;
        state.serialize_field("piconero", &self.piconero)?;
        state.end()
    }
}
impl MoneroAmount {
    pub fn from_xmr(xmr_amount: f64) -> Self {
        let piconero = (xmr_amount * f64::powf(10.0, 12.0)) as u64;
        Self { piconero }
    }

    pub fn from_piconero(piconero: u64) -> Self {
        Self { piconero }
    }

    #[allow(non_snake_case)]
    pub fn as_XMR(&self) -> f64 {
        (self.piconero as f64) / (u64::pow(10, 12) as f64)
    }

    pub fn as_piconero(&self) -> u64 {
        self.piconero
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.piconero.to_le_bytes()
    }
}

impl Display for MoneroAmount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Monero Amount: {} XMR, {} piconero",
            self.as_XMR(),
            self.as_piconero()
        )?;
        Ok(())
    }
}

impl ops::Add<Self> for MoneroAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            piconero: self.piconero + rhs.piconero,
        }
    }
}

impl ops::AddAssign for MoneroAmount {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            piconero: self.piconero + other.piconero,
        }
    }
}

impl ops::Sub for MoneroAmount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            piconero: self.piconero - rhs.piconero,
        }
    }
}

impl ops::Mul for MoneroAmount {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            piconero: self.piconero * rhs.piconero,
        }
    }
}

impl ops::Mul<f64> for MoneroAmount {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            piconero: ((self.piconero as f64) * rhs) as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Construction Tests
    // ============================================================================

    #[test]
    fn test_from_xmr() {
        let amount = MoneroAmount::from_xmr(1.0);
        assert_eq!(amount.as_piconero(), 1_000_000_000_000); // 10^12 piconero
    }

    #[test]
    fn test_from_xmr_fractional() {
        let amount = MoneroAmount::from_xmr(0.5);
        assert_eq!(amount.as_piconero(), 500_000_000_000);
    }

    #[test]
    fn test_from_xmr_small() {
        let amount = MoneroAmount::from_xmr(0.000001);
        assert_eq!(amount.as_piconero(), 1_000_000); // 10^6 piconero
    }

    #[test]
    fn test_from_piconero() {
        let amount = MoneroAmount::from_piconero(1_000_000_000_000);
        assert!((amount.as_XMR() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_from_piconero_zero() {
        let amount = MoneroAmount::from_piconero(0);
        assert_eq!(amount.as_piconero(), 0);
        assert_eq!(amount.as_XMR(), 0.0);
    }

    // ============================================================================
    // Conversion Tests
    // ============================================================================

    #[test]
    fn test_as_xmr() {
        let amount = MoneroAmount::from_piconero(1_500_000_000_000);
        assert!((amount.as_XMR() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_roundtrip_xmr() {
        let original = 2.5;
        let amount = MoneroAmount::from_xmr(original);
        assert!((amount.as_XMR() - original).abs() < 0.0001);
    }

    #[test]
    fn test_to_bytes() {
        let amount = MoneroAmount::from_piconero(0x0102030405060708);
        let bytes = amount.to_bytes();
        assert_eq!(bytes, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]); // Little endian
    }

    #[test]
    fn test_to_bytes_zero() {
        let amount = MoneroAmount::from_piconero(0);
        let bytes = amount.to_bytes();
        assert_eq!(bytes, [0u8; 8]);
    }

    // ============================================================================
    // Arithmetic Tests
    // ============================================================================

    #[test]
    fn test_add() {
        let a = MoneroAmount::from_xmr(1.0);
        let b = MoneroAmount::from_xmr(2.0);
        let sum = a + b;
        assert!((sum.as_XMR() - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_add_piconero() {
        let a = MoneroAmount::from_piconero(100);
        let b = MoneroAmount::from_piconero(50);
        let sum = a + b;
        assert_eq!(sum.as_piconero(), 150);
    }

    #[test]
    fn test_add_assign() {
        let mut amount = MoneroAmount::from_xmr(1.0);
        amount += MoneroAmount::from_xmr(0.5);
        assert!((amount.as_XMR() - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_sub() {
        let a = MoneroAmount::from_xmr(3.0);
        let b = MoneroAmount::from_xmr(1.0);
        let diff = a - b;
        assert!((diff.as_XMR() - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_mul_amount() {
        let a = MoneroAmount::from_piconero(100);
        let b = MoneroAmount::from_piconero(2);
        let product = a * b;
        assert_eq!(product.as_piconero(), 200);
    }

    #[test]
    fn test_mul_f64() {
        let amount = MoneroAmount::from_xmr(2.0);
        let scaled = amount * 1.5;
        assert!((scaled.as_XMR() - 3.0).abs() < 0.0001);
    }

    // ============================================================================
    // Comparison Tests
    // ============================================================================

    #[test]
    fn test_eq() {
        let a = MoneroAmount::from_xmr(1.0);
        let b = MoneroAmount::from_xmr(1.0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne() {
        let a = MoneroAmount::from_xmr(1.0);
        let b = MoneroAmount::from_xmr(2.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_ordering() {
        let small = MoneroAmount::from_xmr(1.0);
        let large = MoneroAmount::from_xmr(2.0);
        
        assert!(small < large);
        assert!(large > small);
        assert!(small <= small);
        assert!(large >= large);
    }

    // ============================================================================
    // Default and Clone Tests
    // ============================================================================

    #[test]
    fn test_default() {
        let amount = MoneroAmount::default();
        assert_eq!(amount.as_piconero(), 0);
    }

    #[test]
    fn test_clone() {
        let original = MoneroAmount::from_xmr(1.5);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_copy() {
        let original = MoneroAmount::from_xmr(1.5);
        let copied = original;
        assert_eq!(original.as_piconero(), copied.as_piconero());
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_serialize() {
        let amount = MoneroAmount::from_piconero(1_000_000_000_000);
        let json = serde_json::to_string(&amount).unwrap();
        assert!(json.contains("piconero"));
        assert!(json.contains("1000000000000"));
    }

    #[test]
    fn test_deserialize() {
        let json = r#"{"piconero":1000000000000}"#;
        let amount: MoneroAmount = serde_json::from_str(json).unwrap();
        assert_eq!(amount.as_piconero(), 1_000_000_000_000);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let original = MoneroAmount::from_xmr(2.5);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MoneroAmount = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    // ============================================================================
    // Display Tests
    // ============================================================================

    #[test]
    fn test_display() {
        let amount = MoneroAmount::from_xmr(1.5);
        let display = format!("{}", amount);
        assert!(display.contains("XMR"));
        assert!(display.contains("piconero"));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_large_amount() {
        // Maximum circulating supply of Monero is about 18.4 million
        let large = MoneroAmount::from_xmr(18_400_000.0);
        assert!(large.as_piconero() > 0);
    }

    #[test]
    fn test_minimum_amount() {
        let min = MoneroAmount::from_piconero(1);
        assert_eq!(min.as_piconero(), 1);
        assert!(min.as_XMR() > 0.0);
    }
}
