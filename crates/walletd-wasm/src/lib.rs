//! # WalletD WASM
//!
//! WebAssembly bindings for the WalletD multi-chain wallet SDK.
//!
//! ## Usage in JavaScript/TypeScript
//!
//! ```javascript
//! import init, { EthereumWallet, generateMnemonic } from 'walletd-wasm';
//!
//! async function main() {
//!     await init();
//!     
//!     // Generate a new mnemonic
//!     const mnemonic = generateMnemonic(12);
//!     console.log("Mnemonic:", mnemonic);
//!     
//!     // Create Ethereum wallet
//!     const wallet = EthereumWallet.fromMnemonic(mnemonic);
//!     console.log("Address:", wallet.address());
//! }
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Initialize panic hook for better error messages in browser console
#[cfg(feature = "console_error_panic_hook")]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Initialization
// ============================================================================

/// Initialize the WASM module. Call this before using any other functions.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    set_panic_hook();
}

/// Returns the WalletD WASM version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============================================================================
// Mnemonic Generation
// ============================================================================

/// Generate a BIP-39 mnemonic phrase
///
/// # Arguments
/// * `word_count` - Number of words (12 or 24)
///
/// # Returns
/// A space-separated mnemonic phrase
#[wasm_bindgen(js_name = generateMnemonic)]
pub fn generate_mnemonic(word_count: u8) -> Result<String, JsError> {
    use bip32::Mnemonic;
    
    match word_count {
        12 => {
            let mut entropy = [0u8; 16]; // 128 bits for 12 words
            getrandom::getrandom(&mut entropy).map_err(|e| JsError::new(&e.to_string()))?;
            
            // Pad to 32 bytes as required by the API
            let mut full_entropy = [0u8; 32];
            full_entropy[..16].copy_from_slice(&entropy);
            
            // Use bip32's built-in entropy handling
            let mnemonic = Mnemonic::from_entropy(full_entropy, bip32::Language::English);
            // Take only first 12 words
            let words: Vec<&str> = mnemonic.phrase().split_whitespace().take(12).collect();
            Ok(words.join(" "))
        }
        24 => {
            let mut entropy = [0u8; 32]; // 256 bits for 24 words
            getrandom::getrandom(&mut entropy).map_err(|e| JsError::new(&e.to_string()))?;
            let mnemonic = Mnemonic::from_entropy(entropy, bip32::Language::English);
            Ok(mnemonic.phrase().to_string())
        }
        _ => Err(JsError::new("Word count must be 12 or 24")),
    }
}

/// Validate a mnemonic phrase
#[wasm_bindgen(js_name = validateMnemonic)]
pub fn validate_mnemonic(phrase: &str) -> bool {
    use bip32::Mnemonic;
    Mnemonic::new(phrase, bip32::Language::English).is_ok()
}

// ============================================================================
// Ethereum Wallet
// ============================================================================

/// Ethereum wallet for browser environments
#[wasm_bindgen]
pub struct EthereumWallet {
    private_key: [u8; 32],
    public_key: Vec<u8>,
    address: String,
}

#[wasm_bindgen]
impl EthereumWallet {
    /// Create a new random Ethereum wallet
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<EthereumWallet, JsError> {
        let mut private_key = [0u8; 32];
        getrandom::getrandom(&mut private_key)
            .map_err(|e| JsError::new(&e.to_string()))?;
        
        Self::from_private_key_bytes(&private_key)
    }
    
    /// Create wallet from mnemonic phrase
    #[wasm_bindgen(js_name = fromMnemonic)]
    pub fn from_mnemonic(mnemonic: &str) -> Result<EthereumWallet, JsError> {
        use bip32::{Mnemonic, XPrv, DerivationPath};
        use std::str::FromStr;
        
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English)
            .map_err(|e| JsError::new(&format!("Invalid mnemonic: {}", e)))?;
        
        let seed = mnemonic.to_seed("");
        
        // Derive key using BIP-44 path for Ethereum: m/44'/60'/0'/0/0
        let path = DerivationPath::from_str("m/44'/60'/0'/0/0")
            .map_err(|e| JsError::new(&format!("Invalid path: {}", e)))?;
        
        let child_xprv = XPrv::derive_from_path(&seed, &path)
            .map_err(|e| JsError::new(&format!("Derivation error: {}", e)))?;
        
        let private_key: [u8; 32] = child_xprv.private_key().to_bytes().into();
        
        Self::from_private_key_bytes(&private_key)
    }
    
    /// Create wallet from private key hex string
    #[wasm_bindgen(js_name = fromPrivateKey)]
    pub fn from_private_key(private_key_hex: &str) -> Result<EthereumWallet, JsError> {
        let private_key_hex = private_key_hex.trim_start_matches("0x");
        let bytes = hex::decode(private_key_hex)
            .map_err(|e| JsError::new(&format!("Invalid hex: {}", e)))?;
        
        if bytes.len() != 32 {
            return Err(JsError::new("Private key must be 32 bytes"));
        }
        
        let mut private_key = [0u8; 32];
        private_key.copy_from_slice(&bytes);
        
        Self::from_private_key_bytes(&private_key)
    }
    
    fn from_private_key_bytes(private_key: &[u8; 32]) -> Result<EthereumWallet, JsError> {
        use k256::ecdsa::SigningKey;
        use tiny_keccak::{Hasher, Keccak};
        
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| JsError::new(&format!("Invalid private key: {}", e)))?;
        
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let public_key = public_key_bytes.as_bytes()[1..].to_vec(); // Remove 0x04 prefix
        
        // Compute address: keccak256(public_key)[12:]
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(&public_key);
        hasher.finalize(&mut hash);
        
        let address = format!("0x{}", hex::encode(&hash[12..]));
        
        // Checksum the address
        let address = checksum_address(&address);
        
        Ok(EthereumWallet {
            private_key: *private_key,
            public_key,
            address,
        })
    }
    
    /// Get the wallet address (checksummed)
    #[wasm_bindgen]
    pub fn address(&self) -> String {
        self.address.clone()
    }
    
    /// Get the private key as hex string
    #[wasm_bindgen(js_name = privateKey)]
    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(&self.private_key))
    }
    
    /// Get the public key as hex string (uncompressed, without 0x04 prefix)
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> String {
        format!("0x{}", hex::encode(&self.public_key))
    }
    
    /// Sign a message (returns signature as hex)
    #[wasm_bindgen(js_name = signMessage)]
    pub fn sign_message(&self, message: &str) -> Result<String, JsError> {
        use k256::ecdsa::{SigningKey, Signature, signature::Signer};
        use tiny_keccak::{Hasher, Keccak};
        
        // Ethereum signed message format
        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(prefix.as_bytes());
        hasher.update(message.as_bytes());
        hasher.finalize(&mut hash);
        
        let signing_key = SigningKey::from_bytes((&self.private_key).into())
            .map_err(|e| JsError::new(&format!("Key error: {}", e)))?;
        
        let signature: Signature = signing_key.sign(&hash);
        
        Ok(format!("0x{}", hex::encode(signature.to_bytes())))
    }
    
    /// Export wallet as JSON
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<JsValue, JsError> {
        let wallet_data = WalletExport {
            address: self.address.clone(),
            public_key: format!("0x{}", hex::encode(&self.public_key)),
        };
        
        serde_wasm_bindgen::to_value(&wallet_data)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }
}

impl Default for EthereumWallet {
    fn default() -> Self {
        Self::new().expect("Failed to create wallet")
    }
}

// ============================================================================
// Bitcoin Keys (address generation only - no signing for security)
// ============================================================================

/// Bitcoin key pair for address generation
#[wasm_bindgen]
pub struct BitcoinKeys {
    private_key: [u8; 32],
    public_key: Vec<u8>,
    address: String,
    network: String,
}

#[wasm_bindgen]
impl BitcoinKeys {
    /// Create Bitcoin keys from mnemonic
    ///
    /// # Arguments
    /// * `mnemonic` - BIP-39 mnemonic phrase
    /// * `network` - "mainnet" or "testnet"
    #[wasm_bindgen(js_name = fromMnemonic)]
    pub fn from_mnemonic(mnemonic: &str, network: &str) -> Result<BitcoinKeys, JsError> {
        use bip32::{Mnemonic, XPrv, DerivationPath};
        use std::str::FromStr;
        use sha2::{Sha256, Digest};
        
        let mnemonic = Mnemonic::new(mnemonic, bip32::Language::English)
            .map_err(|e| JsError::new(&format!("Invalid mnemonic: {}", e)))?;
        
        let seed = mnemonic.to_seed("");
        
        // BIP-84 path for native SegWit: m/84'/0'/0'/0/0 (mainnet) or m/84'/1'/0'/0/0 (testnet)
        let coin_type = if network == "testnet" { "1" } else { "0" };
        let path = DerivationPath::from_str(&format!("m/84'/{coin_type}'/0'/0/0"))
            .map_err(|e| JsError::new(&format!("Invalid path: {}", e)))?;
        
        let child_xprv = XPrv::derive_from_path(&seed, &path)
            .map_err(|e| JsError::new(&format!("Derivation error: {}", e)))?;
        
        let private_key: [u8; 32] = child_xprv.private_key().to_bytes().into();
        
        // Derive public key using k256
        use k256::ecdsa::SigningKey;
        let signing_key = SigningKey::from_bytes((&private_key).into())
            .map_err(|e| JsError::new(&format!("Key error: {}", e)))?;
        
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(true); // Compressed
        let public_key = public_key_bytes.as_bytes().to_vec();
        
        // Generate bech32 address (native SegWit)
        // Hash160 = RIPEMD160(SHA256(pubkey))
        let sha256_hash = Sha256::digest(&public_key);
        
        // RIPEMD160
        let mut ripemd = ripemd::Ripemd160::new();
        ripemd::Digest::update(&mut ripemd, &sha256_hash);
        let hash160: [u8; 20] = ripemd::Digest::finalize(ripemd).into();
        
        // Bech32 encoding
        let hrp = if network == "testnet" { "tb" } else { "bc" };
        let address = bech32_encode(hrp, &hash160)?;
        
        Ok(BitcoinKeys {
            private_key,
            public_key,
            address,
            network: network.to_string(),
        })
    }
    
    /// Get the Bitcoin address (bech32/native SegWit)
    #[wasm_bindgen]
    pub fn address(&self) -> String {
        self.address.clone()
    }
    
    /// Get the network ("mainnet" or "testnet")
    #[wasm_bindgen]
    pub fn network(&self) -> String {
        self.network.clone()
    }
    
    /// Get the WIF (Wallet Import Format) private key
    #[wasm_bindgen]
    pub fn wif(&self) -> String {
        use sha2::{Sha256, Digest};
        
        let prefix = if self.network == "testnet" { 0xef } else { 0x80 };
        let mut extended = vec![prefix];
        extended.extend_from_slice(&self.private_key);
        extended.push(0x01); // Compressed pubkey flag
        
        // Double SHA256 for checksum
        let hash1 = Sha256::digest(&extended);
        let hash2 = Sha256::digest(&hash1);
        extended.extend_from_slice(&hash2[..4]);
        
        bs58::encode(extended).into_string()
    }
    
    /// Get compressed public key as hex
    #[wasm_bindgen(js_name = publicKey)]
    pub fn public_key(&self) -> String {
        format!("0x{}", hex::encode(&self.public_key))
    }
}

// Simple bech32 encoding for native SegWit addresses
fn bech32_encode(hrp: &str, data: &[u8; 20]) -> Result<String, JsError> {
    const CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    
    // Convert 8-bit to 5-bit
    let mut result = Vec::new();
    result.push(0u8); // witness version 0
    
    let mut acc = 0u32;
    let mut bits = 0u8;
    for byte in data {
        acc = (acc << 8) | (*byte as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            result.push(((acc >> bits) & 0x1f) as u8);
        }
    }
    if bits > 0 {
        result.push(((acc << (5 - bits)) & 0x1f) as u8);
    }
    
    // Create checksum
    let mut chk = bech32_polymod_step(bech32_hrp_expand(hrp), 1);
    for d in &result {
        chk = bech32_polymod_step(chk, *d as u32);
    }
    for _ in 0..6 {
        chk = bech32_polymod_step(chk, 0);
    }
    chk ^= 0x2bc830a3; // bech32m constant
    
    let mut output = format!("{}1", hrp);
    for d in &result {
        output.push(CHARSET[*d as usize] as char);
    }
    for i in (0..6).rev() {
        output.push(CHARSET[((chk >> (5 * i)) & 0x1f) as usize] as char);
    }
    
    Ok(output)
}

fn bech32_hrp_expand(hrp: &str) -> u32 {
    let mut chk = 1u32;
    for c in hrp.chars() {
        chk = bech32_polymod_step(chk, (c as u32) >> 5);
    }
    chk = bech32_polymod_step(chk, 0);
    for c in hrp.chars() {
        chk = bech32_polymod_step(chk, (c as u32) & 0x1f);
    }
    chk
}

fn bech32_polymod_step(pre: u32, val: u32) -> u32 {
    let b = pre >> 25;
    let chk = ((pre & 0x1ffffff) << 5) ^ val;
    let chk = if b & 1 != 0 { chk ^ 0x3b6a57b2 } else { chk };
    let chk = if b & 2 != 0 { chk ^ 0x26508e6d } else { chk };
    let chk = if b & 4 != 0 { chk ^ 0x1ea119fa } else { chk };
    let chk = if b & 8 != 0 { chk ^ 0x3d4233dd } else { chk };
    if b & 16 != 0 { chk ^ 0x2a1462b3 } else { chk }
}

// ============================================================================
// Monero Amount Utilities
// ============================================================================

/// Monero amount handling (XMR has 12 decimal places)
#[wasm_bindgen]
pub struct MoneroAmount {
    piconero: u64,
}

#[wasm_bindgen]
impl MoneroAmount {
    /// Create from XMR (as floating point)
    #[wasm_bindgen(js_name = fromXmr)]
    pub fn from_xmr(xmr: f64) -> MoneroAmount {
        let piconero = (xmr * 1e12) as u64;
        MoneroAmount { piconero }
    }
    
    /// Create from piconero (as string for precision)
    #[wasm_bindgen(js_name = fromPiconero)]
    pub fn from_piconero(piconero: &str) -> Result<MoneroAmount, JsError> {
        let piconero: u64 = piconero.parse()
            .map_err(|e| JsError::new(&format!("Invalid piconero: {}", e)))?;
        Ok(MoneroAmount { piconero })
    }
    
    /// Get amount as XMR (floating point)
    #[wasm_bindgen]
    pub fn xmr(&self) -> f64 {
        self.piconero as f64 / 1e12
    }
    
    /// Get amount as piconero (string for precision)
    #[wasm_bindgen]
    pub fn piconero(&self) -> String {
        self.piconero.to_string()
    }
    
    /// Format for display
    #[wasm_bindgen]
    pub fn display(&self) -> String {
        format!("{:.12} XMR", self.xmr())
    }
    
    /// Add two amounts
    #[wasm_bindgen]
    pub fn add(&self, other: &MoneroAmount) -> MoneroAmount {
        MoneroAmount {
            piconero: self.piconero + other.piconero,
        }
    }
    
    /// Subtract amount (returns 0 if would underflow)
    #[wasm_bindgen]
    pub fn sub(&self, other: &MoneroAmount) -> MoneroAmount {
        MoneroAmount {
            piconero: self.piconero.saturating_sub(other.piconero),
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn checksum_address(address: &str) -> String {
    use tiny_keccak::{Hasher, Keccak};
    
    let address = address.trim_start_matches("0x").to_lowercase();
    
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(address.as_bytes());
    hasher.finalize(&mut hash);
    
    let hash_hex = hex::encode(&hash);
    
    let checksummed: String = address
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_alphabetic() {
                let hash_char = hash_hex.chars().nth(i).unwrap();
                if hash_char >= '8' {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            } else {
                c
            }
        })
        .collect();
    
    format!("0x{}", checksummed)
}

/// Convert hex string to bytes
#[wasm_bindgen(js_name = hexToBytes)]
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsError> {
    let hex = hex.trim_start_matches("0x");
    hex::decode(hex).map_err(|e| JsError::new(&format!("Invalid hex: {}", e)))
}

/// Convert bytes to hex string
#[wasm_bindgen(js_name = bytesToHex)]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Compute keccak256 hash
#[wasm_bindgen]
pub fn keccak256(data: &[u8]) -> Vec<u8> {
    use tiny_keccak::{Hasher, Keccak};
    
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut hash);
    hash.to_vec()
}

/// Compute SHA256 hash
#[wasm_bindgen]
pub fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

// ============================================================================
// Amount Utilities
// ============================================================================

/// Convert ETH to Wei (as string for precision)
#[wasm_bindgen(js_name = ethToWei)]
pub fn eth_to_wei(eth: f64) -> String {
    let wei = (eth * 1e18) as u128;
    wei.to_string()
}

/// Convert Wei to ETH
#[wasm_bindgen(js_name = weiToEth)]
pub fn wei_to_eth(wei: &str) -> Result<f64, JsError> {
    let wei: u128 = wei.parse()
        .map_err(|e| JsError::new(&format!("Invalid wei value: {}", e)))?;
    Ok(wei as f64 / 1e18)
}

/// Convert Gwei to Wei
#[wasm_bindgen(js_name = gweiToWei)]
pub fn gwei_to_wei(gwei: f64) -> String {
    let wei = (gwei * 1e9) as u128;
    wei.to_string()
}

// ============================================================================
// Types for JS Interop
// ============================================================================

#[derive(Serialize, Deserialize)]
struct WalletExport {
    address: String,
    public_key: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let hash = keccak256(b"hello");
        assert_eq!(hash.len(), 32);
        // Known keccak256("hello") hash
        let expected = hex::decode("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8").unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = hex_to_bytes("0x48656c6c6f");
        assert!(bytes.is_ok());
        assert_eq!(bytes.unwrap(), b"Hello");
        
        let hex = bytes_to_hex(b"Hello");
        assert_eq!(hex, "0x48656c6c6f");
    }
    
    #[test]
    fn test_eth_to_wei() {
        assert_eq!(eth_to_wei(1.0), "1000000000000000000");
        assert_eq!(eth_to_wei(0.1), "100000000000000000");
    }

    #[test]
    fn test_wei_to_eth() {
        let result = wei_to_eth("1000000000000000000");
        assert!(result.is_ok());
        assert!((result.unwrap() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_gwei_to_wei() {
        assert_eq!(gwei_to_wei(1.0), "1000000000");
        assert_eq!(gwei_to_wei(21.0), "21000000000");
    }
    
    #[test]
    fn test_monero_amount_from_xmr() {
        let amount = MoneroAmount::from_xmr(1.5);
        assert_eq!(amount.piconero(), "1500000000000");
        assert!((amount.xmr() - 1.5).abs() < 0.0001);
    }
    
    #[test]
    fn test_monero_amount_from_piconero() {
        let amount = MoneroAmount::from_piconero("1000000000000").unwrap();
        assert!((amount.xmr() - 1.0).abs() < 0.0001);
    }
    
    #[test]
    fn test_monero_amount_add() {
        let a = MoneroAmount::from_xmr(1.0);
        let b = MoneroAmount::from_xmr(0.5);
        let c = a.add(&b);
        assert!((c.xmr() - 1.5).abs() < 0.0001);
    }
    
    #[test]
    fn test_monero_amount_sub() {
        let a = MoneroAmount::from_xmr(1.5);
        let b = MoneroAmount::from_xmr(0.5);
        let c = a.sub(&b);
        assert!((c.xmr() - 1.0).abs() < 0.0001);
    }
    
    #[test]
    fn test_monero_amount_display() {
        let amount = MoneroAmount::from_xmr(1.5);
        assert!(amount.display().contains("XMR"));
    }
    
    #[test]
    fn test_checksum_address_simple() {
        // Test lowercase -> checksummed
        let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
        let checksummed = checksum_address(address);
        assert!(checksummed.starts_with("0x"));
        assert_eq!(checksummed.len(), 42);
        // The checksum should make some letters uppercase
        assert!(checksummed.chars().any(|c| c.is_uppercase()));
    }
    
    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
