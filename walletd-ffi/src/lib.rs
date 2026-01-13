//! WalletD FFI - C bindings for WalletD SDK
//!
//! This module provides C-compatible functions for use from other languages.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Returns the version string of the WalletD SDK
/// 
/// # Safety
/// This function is safe to call from C code.
#[no_mangle]
pub extern "C" fn walletd_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    CString::new(version)
        .unwrap_or_else(|_| CString::new("unknown").unwrap())
        .into_raw()
}

/// Returns a comma-separated list of supported chains
/// 
/// # Safety
/// This function is safe to call from C code.
#[no_mangle]
pub extern "C" fn walletd_supported_chains() -> *mut c_char {
    let chains = "bitcoin,ethereum,solana,hedera,monero,icp,base,polygon,avalanche,arbitrum,cardano,polkadot,cosmos,near,tron,sui,aptos,ton";
    CString::new(chains)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

/// Checks if a chain is supported
/// 
/// # Safety
/// - `chain` must be a valid null-terminated C string
/// - `chain` must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn walletd_is_chain_supported(chain: *const c_char) -> bool {
    if chain.is_null() {
        return false;
    }
    
    let chain_str = match CStr::from_ptr(chain).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let supported = [
        "bitcoin", "ethereum", "solana", "hedera", "monero", "icp",
        "base", "polygon", "avalanche", "arbitrum", "cardano", "polkadot",
        "cosmos", "near", "tron", "sui", "aptos", "ton",
    ];
    
    supported.contains(&chain_str.to_lowercase().as_str())
}

/// Creates a new wallet for the specified chain
/// Returns a JSON string with wallet details or an error
/// 
/// # Safety
/// - `chain` must be a valid null-terminated C string
/// - `chain` must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn walletd_create_wallet(chain: *const c_char) -> *mut c_char {
    if chain.is_null() {
        return CString::new(r#"{"error": "null chain parameter"}"#)
            .unwrap()
            .into_raw();
    }
    
    let chain_str = match CStr::from_ptr(chain).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"error": "invalid chain string"}"#)
                .unwrap()
                .into_raw();
        }
    };
    
    // Placeholder - actual implementation would create wallet
    let result = format!(r#"{{"chain": "{}", "status": "created"}}"#, chain_str);
    CString::new(result)
        .unwrap_or_else(|_| CString::new(r#"{"error": "string conversion failed"}"#).unwrap())
        .into_raw()
}

/// Signs a transaction
/// 
/// # Safety
/// - `chain` must be a valid null-terminated C string
/// - `tx_data` must be a valid null-terminated C string
/// - Both pointers must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn walletd_sign_transaction(
    chain: *const c_char,
    tx_data: *const c_char,
) -> *mut c_char {
    if chain.is_null() || tx_data.is_null() {
        return CString::new(r#"{"error": "null parameter"}"#)
            .unwrap()
            .into_raw();
    }
    
    let chain_str = match CStr::from_ptr(chain).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"error": "invalid chain string"}"#)
                .unwrap()
                .into_raw();
        }
    };
    
    let _tx_str = match CStr::from_ptr(tx_data).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"error": "invalid tx_data string"}"#)
                .unwrap()
                .into_raw();
        }
    };
    
    // Placeholder - actual implementation would sign transaction
    let result = format!(r#"{{"chain": "{}", "status": "signed", "signature": "0x..."}}"#, chain_str);
    CString::new(result)
        .unwrap_or_else(|_| CString::new(r#"{"error": "string conversion failed"}"#).unwrap())
        .into_raw()
}

/// Broadcasts a signed transaction
/// 
/// # Safety
/// - `chain` must be a valid null-terminated C string
/// - `signed_tx` must be a valid null-terminated C string
/// - Both pointers must point to valid memory
#[no_mangle]
pub unsafe extern "C" fn walletd_broadcast_transaction(
    chain: *const c_char,
    signed_tx: *const c_char,
) -> *mut c_char {
    if chain.is_null() || signed_tx.is_null() {
        return CString::new(r#"{"error": "null parameter"}"#)
            .unwrap()
            .into_raw();
    }
    
    let chain_str = match CStr::from_ptr(chain).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"error": "invalid chain string"}"#)
                .unwrap()
                .into_raw();
        }
    };
    
    let _signed_tx_str = match CStr::from_ptr(signed_tx).to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(r#"{"error": "invalid signed_tx string"}"#)
                .unwrap()
                .into_raw();
        }
    };
    
    // Placeholder - actual implementation would broadcast
    let result = format!(r#"{{"chain": "{}", "status": "broadcast", "txid": "..."}}"#, chain_str);
    CString::new(result)
        .unwrap_or_else(|_| CString::new(r#"{"error": "string conversion failed"}"#).unwrap())
        .into_raw()
}

/// Frees a string allocated by WalletD
/// 
/// # Safety
/// - `s` must be a pointer returned by a walletd_* function
/// - `s` must not have been freed before
/// - After calling this function, `s` must not be used
#[no_mangle]
pub unsafe extern "C" fn walletd_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_version() {
        let version = walletd_version();
        assert!(!version.is_null());
        unsafe { walletd_free_string(version); }
    }

    #[test]
    fn test_supported_chains() {
        let chains = walletd_supported_chains();
        assert!(!chains.is_null());
        unsafe {
            let chains_str = CStr::from_ptr(chains).to_str().unwrap();
            assert!(chains_str.contains("bitcoin"));
            assert!(chains_str.contains("ethereum"));
            walletd_free_string(chains);
        }
    }

    #[test]
    fn test_is_chain_supported() {
        let btc = CString::new("bitcoin").unwrap();
        let fake = CString::new("fakecoin").unwrap();
        
        unsafe {
            assert!(walletd_is_chain_supported(btc.as_ptr()));
            assert!(!walletd_is_chain_supported(fake.as_ptr()));
            assert!(!walletd_is_chain_supported(std::ptr::null()));
        }
    }

    #[test]
    fn test_create_wallet_null() {
        unsafe {
            let result = walletd_create_wallet(std::ptr::null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert!(result_str.contains("error"));
            walletd_free_string(result);
        }
    }
}
