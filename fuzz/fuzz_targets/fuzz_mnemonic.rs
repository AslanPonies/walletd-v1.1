#![no_main]

use libfuzzer_sys::fuzz_target;
use bip39::Mnemonic;
use std::str::FromStr;

fuzz_target!(|data: &str| {
    // Fuzz BIP39 mnemonic parsing
    // This should never panic, only return errors for invalid input
    let _ = Mnemonic::from_str(data);
    
    // Also test with normalized whitespace
    let normalized: String = data.split_whitespace().collect::<Vec<_>>().join(" ");
    let _ = Mnemonic::from_str(&normalized);
    
    // Test word count validation
    let word_count = data.split_whitespace().count();
    if word_count == 12 || word_count == 24 {
        // Valid word counts - parsing may still fail due to invalid words/checksum
        let _ = Mnemonic::from_str(&normalized);
    }
});
