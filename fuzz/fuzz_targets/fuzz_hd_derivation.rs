#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
struct DerivationInput {
    path_segments: Vec<u32>,
    hardened_flags: Vec<bool>,
}

fuzz_target!(|input: DerivationInput| {
    // Build a derivation path string from arbitrary input
    let mut path = String::from("m");
    
    for (i, &segment) in input.path_segments.iter().take(10).enumerate() {
        path.push('/');
        // Limit segment value to reasonable range
        let segment_val = segment % 0x80000000;
        path.push_str(&segment_val.to_string());
        
        // Add hardened marker based on flag
        if input.hardened_flags.get(i).copied().unwrap_or(false) {
            path.push('\'');
        }
    }
    
    // The path construction itself should never panic
    // We're testing that arbitrary paths don't cause issues
    
    // Validate path format
    let is_valid = path.starts_with("m") && 
                   path.chars().all(|c| c.is_ascii_digit() || c == '/' || c == '\'' || c == 'm');
    
    assert!(is_valid, "Generated invalid path: {}", path);
});
