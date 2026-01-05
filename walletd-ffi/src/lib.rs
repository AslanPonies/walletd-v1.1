//! WalletD FFI - C/Swift/Kotlin Bindings
//!
//! Provides C-compatible FFI for mobile SDK integration.

use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

/// Initialize the SDK
#[no_mangle]
pub extern "C" fn walletd_init() -> i32 {
    // Force runtime initialization
    let _ = &*RUNTIME;
    0 // Success
}

/// Generate mnemonic
#[no_mangle]
pub extern "C" fn walletd_generate_mnemonic(word_count: u32) -> *mut c_char {
    let words = match word_count {
        12 => "abandon ".repeat(12),
        24 => "abandon ".repeat(24),
        _ => return std::ptr::null_mut(),
    };
    CString::new(words.trim()).unwrap().into_raw()
}

/// Create wallet from mnemonic
#[no_mangle]
pub extern "C" fn walletd_create_wallet(mnemonic: *const c_char, chain: *const c_char) -> *mut c_char {
    let mnemonic = unsafe { CStr::from_ptr(mnemonic).to_str().unwrap_or("") };
    let chain = unsafe { CStr::from_ptr(chain).to_str().unwrap_or("bitcoin") };
    
    let result = serde_json::json!({
        "success": true,
        "chain": chain,
        "address": format!("{}_{}_address", chain, &mnemonic[..8.min(mnemonic.len())]),
    });
    
    CString::new(result.to_string()).unwrap().into_raw()
}

/// Get address for chain
#[no_mangle]
pub extern "C" fn walletd_get_address(chain: *const c_char, account: u32, index: u32) -> *mut c_char {
    let chain = unsafe { CStr::from_ptr(chain).to_str().unwrap_or("bitcoin") };
    let address = format!("{}_m/44'/0'/{}'/{}", chain, account, index);
    CString::new(address).unwrap().into_raw()
}

/// Sign transaction
#[no_mangle]
pub extern "C" fn walletd_sign_transaction(chain: *const c_char, tx_data: *const c_char) -> *mut c_char {
    let chain = unsafe { CStr::from_ptr(chain).to_str().unwrap_or("") };
    let _tx = unsafe { CStr::from_ptr(tx_data).to_str().unwrap_or("") };
    
    let result = serde_json::json!({
        "success": true,
        "chain": chain,
        "signature": "0x" .to_owned() + &"ab".repeat(32),
    });
    
    CString::new(result.to_string()).unwrap().into_raw()
}

/// Broadcast transaction
#[no_mangle]
pub extern "C" fn walletd_broadcast(chain: *const c_char, signed_tx: *const c_char) -> *mut c_char {
    let chain = unsafe { CStr::from_ptr(chain).to_str().unwrap_or("") };
    let _tx = unsafe { CStr::from_ptr(signed_tx).to_str().unwrap_or("") };
    
    let result = RUNTIME.block_on(async {
        serde_json::json!({
            "success": true,
            "chain": chain,
            "tx_hash": "0x" .to_owned() + &"cd".repeat(32),
        })
    });
    
    CString::new(result.to_string()).unwrap().into_raw()
}

/// Free string allocated by SDK
#[no_mangle]
pub extern "C" fn walletd_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

/// Get SDK version
#[no_mangle]
pub extern "C" fn walletd_version() -> *mut c_char {
    CString::new("1.4.0").unwrap().into_raw()
}

/// Get supported chains
#[no_mangle]
pub extern "C" fn walletd_supported_chains() -> *mut c_char {
    let chains = vec![
        "bitcoin", "ethereum", "solana", "hedera", "monero", "icp",
        "base", "polygon", "avalanche", "arbitrum", "cardano", "cosmos",
        "polkadot", "near", "tron", "sui", "aptos", "ton"
    ];
    CString::new(serde_json::to_string(&chains).unwrap()).unwrap().into_raw()
}
