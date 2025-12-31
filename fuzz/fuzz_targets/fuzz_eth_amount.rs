#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;
use walletd_ethereum::EthereumAmount;
use walletd_ethereum::alloy::primitives::U256;

#[derive(Debug, Arbitrary)]
struct AmountInput {
    // Use 4 u64s to construct a U256
    wei_parts: [u64; 4],
    eth_value: f64,
    gwei_value: u64,
}

fuzz_target!(|input: AmountInput| {
    // Construct U256 from parts
    let wei_value = U256::from_limbs(input.wei_parts);
    
    // Test from_wei - should never panic
    let amount = EthereumAmount::from_wei(wei_value);
    
    // Verify round-trip consistency
    let wei_back = amount.wei();
    assert_eq!(wei_back, wei_value, "Wei round-trip failed");
    
    // Test from_gwei
    let gwei_amount = EthereumAmount::from_gwei(input.gwei_value);
    let _ = gwei_amount.wei();
    
    // Test from_eth with finite positive values
    if input.eth_value.is_finite() && input.eth_value >= 0.0 && input.eth_value < 1e30 {
        let eth_amount = EthereumAmount::from_eth(input.eth_value);
        let _ = eth_amount.wei();
    }
    
    // Test addition - should handle overflow gracefully
    let amount2 = EthereumAmount::from_gwei(1);
    let _ = amount.clone() + amount2;
    
    // Test subtraction
    let _ = amount.clone() - EthereumAmount::from_gwei(0);
});
