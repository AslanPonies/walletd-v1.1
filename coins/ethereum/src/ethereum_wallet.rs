use ::core::fmt;
use std::fmt::LowerHex;
use std::str::FromStr;

use crate::Error;
use crate::EthClient;
use crate::{EthereumAmount, EthereumFormat};

use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::network::TransactionBuilder;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;

use bdk::bitcoin::secp256k1::ffi::types::AlignedType;
use bdk::bitcoin::secp256k1::PublicKey;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::bip32::DerivationPath;
use bdk::bitcoin::bip32::ExtendedPrivKey;
use bdk::bitcoin::bip32::ExtendedPubKey;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::{DerivableKey, ExtendedKey};
use tiny_keccak::{Hasher, Keccak};

/// Represents an EthereumPublicKey, wraps a [PublicKey] from the secp256k1 crate
#[derive(Debug, Clone)]
pub struct EthereumPublicKey(PublicKey);

impl EthereumPublicKey {
    /// Converts the public key to a byte array
    pub fn to_bytes(&self) -> [u8; 33] {
        self.0.serialize()
    }

    /// Returns the public address of the public key in the specified format
    pub fn to_public_address(&self, address_format: EthereumFormat) -> Result<String, Error> {
        let public_key_full = self.0;

        match address_format {
            EthereumFormat::Checksummed => {
                let mut output = [0u8; 32];
                let mut hasher = Keccak::v256();
                hasher.update(&public_key_full.serialize_uncompressed()[1..]);
                hasher.finalize(&mut output);
                let address = hex::encode(&output[12..]).to_lowercase();

                let mut checksum_address = String::new();
                let mut digest_out2 = [0u8; 32];
                let mut hasher2 = Keccak::v256();
                let address_bytes = address.as_bytes();
                hasher2.update(address_bytes);
                hasher2.finalize(&mut digest_out2);
                let keccak_digest_hex = hex::encode(digest_out2);

                for (i, address_char) in address.chars().enumerate() {
                    let keccak_char = &keccak_digest_hex[i..i + 1];
                    if u8::from_str_radix(keccak_char, 16)? >= 8 {
                        checksum_address.push(address_char.to_ascii_uppercase());
                    } else {
                        checksum_address.push(address_char);
                    }
                }
                checksum_address = format!("{}{}", "0x", checksum_address);
                Ok(checksum_address)
            }
            EthereumFormat::NonChecksummed => {
                let mut output = [0u8; 32];
                let mut hasher = Keccak::v256();
                hasher.update(&public_key_full.serialize_uncompressed()[1..]);
                hasher.finalize(&mut output);
                let mut address = hex::encode(&output[12..]).to_lowercase();
                address = format!("{}{}", "0x", address);
                Ok(address)
            }
        }
    }
}

impl LowerHex for EthereumPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("0x")?;
        }

        for byte in &self.to_bytes() {
            write!(f, "{byte:02x}")?;
        }

        Ok(())
    }
}

/// Builder for [EthereumWallet], allows for specification of options for the ethereum wallet
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EthereumWalletBuilder {
    address_format: EthereumFormat,
    mnemonic: Option<Mnemonic>,
    chain_id: u64,
}

impl Default for EthereumWalletBuilder {
    /// Specifies the default options for the EthereumWalletBuilder
    fn default() -> Self {
        Self {
            address_format: EthereumFormat::Checksummed,
            mnemonic: None,
            chain_id: 1, // Mainnet
        }
    }
}

impl EthereumWalletBuilder {
    /// Creates a new EthereumWalletBuilder with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the EthereumWallet with the specified options
    pub fn build(&self) -> Result<EthereumWallet, Error> {
        if self.mnemonic.is_none() {
            return Err(Error::UnableToImportWallet(
                "The mnemonic seed was not provided".to_string(),
            ));
        }

        // we need secp256k1 context for key derivation
        let mut buf: Vec<AlignedType> = Vec::new();
        buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
        let secp = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();

        let mnemonic = &self.mnemonic.clone().unwrap();
        let xkey: ExtendedKey = mnemonic.clone().into_extended_key().unwrap();
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(bdk::bitcoin::Network::Bitcoin).unwrap();
        let path = DerivationPath::from_str("m/44h/60h/0h/0/0").unwrap();

        let child = xprv.derive_priv(&secp, &path).unwrap();
        let xpub = ExtendedPubKey::from_priv(&secp, &child);
        let public_key =
            EthereumPublicKey(PublicKey::from_slice(&xpub.public_key.serialize()).unwrap());
        let public_address = public_key.to_public_address(self.address_format)?;
        let wallet = EthereumWallet {
            address_format: self.address_format,
            public_address,
            private_key: Some(child),
            public_key: Some(xpub),
            chain_id: self.chain_id,
        };
        Ok(wallet)
    }

    /// Allows specification of the address format for the wallet
    pub fn address_format(&mut self, address_format: EthereumFormat) -> &mut Self {
        self.address_format = address_format;
        self
    }

    /// Allows specification of the mnemonic seed for the wallet
    pub fn mnemonic(&mut self, mnemonic: Mnemonic) -> &mut Self {
        self.mnemonic = Some(mnemonic);
        self
    }

    /// Allows specification of the chain ID for the wallet
    pub fn chain_id(&mut self, chain_id: u64) -> &mut Self {
        self.chain_id = chain_id;
        self
    }
}

/// Contains the information needed to interact with an Ethereum wallet with a single public address associated with it.
#[derive(Debug, Clone)]
pub struct EthereumWallet {
    address_format: EthereumFormat,
    public_address: String,
    private_key: Option<ExtendedPrivKey>,
    public_key: Option<ExtendedPubKey>,
    chain_id: u64,
}

impl EthereumWallet {
    /// Returns the builder for the [EthereumWallet].
    pub fn builder() -> EthereumWalletBuilder {
        EthereumWalletBuilder::new()
    }

    /// Returns the balance for this Ethereum Wallet.
    pub async fn balance(&self, rpc_url: &str) -> Result<EthereumAmount, Error> {
        let address = Address::from_str(&self.public_address())
            .map_err(|e| Error::FromStr(e.to_string()))?;
        let balance = EthClient::balance(rpc_url, address).await?;
        Ok(balance)
    }

    /// This function creates and broadcasts a basic Ethereum transfer transaction to the Ethereum mempool.
    pub async fn transfer(
        &self,
        rpc_url: &str,
        send_amount: EthereumAmount,
        to_address: &str,
    ) -> Result<String, Error> {
        let private_key = self.private_key
            .ok_or(Error::MissingPrivateKey)?;
        let private_key_bytes = private_key.private_key.secret_bytes();

        // Create signer from private key bytes
        let signer = PrivateKeySigner::from_slice(&private_key_bytes)
            .map_err(|e| Error::Custom(format!("Failed to create signer: {e}")))?;

        // Create provider with signer
        let provider = ProviderBuilder::new()
            .wallet(alloy::network::EthereumWallet::from(signer))
            .connect_http(rpc_url.parse().map_err(|e| Error::Custom(format!("Invalid URL: {e}")))?);

        // Parse destination address
        let to = Address::from_str(to_address)
            .map_err(|e| Error::FromStr(e.to_string()))?;

        // Build transaction request
        // 21000 = gas limit for basic ETH transfer
        let tx = TransactionRequest::default()
            .with_to(to)
            .with_value(send_amount.wei())
            .with_gas_limit(21000)
            .with_chain_id(self.chain_id);

        // Send transaction
        let pending_tx = provider
            .send_transaction(tx)
            .await
            .map_err(|e| Error::TxResponse(format!("Failed to send transaction: {e}")))?;

        // Wait for receipt
        let receipt = pending_tx
            .get_receipt()
            .await
            .map_err(|e| Error::TxResponse(format!("Failed to get receipt: {e}")))?;

        Ok(format!("{:?}", receipt.transaction_hash))
    }

    /// Syncs the wallet with the blockchain by adding previously used addresses to the wallet.
    pub async fn sync(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// Retrieves the next receive address of the wallet.
    pub fn receive_address(&self) -> Result<String, Error> {
        Ok(self.public_address())
    }

    /// Returns the address format used by the wallet
    pub fn address_format(&self) -> EthereumFormat {
        self.address_format
    }

    /// Returns the public address of the wallet
    pub fn public_address(&self) -> String {
        self.public_address.clone()
    }

    /// A convenience method for retrieving the string of a public_address
    pub fn address(&self) -> String {
        self.public_address()
    }

    /// Returns the extended public key of the eth wallet
    pub fn public_key(&self) -> Result<ExtendedPubKey, Error> {
        match &self.public_key {
            Some(public_key) => Ok(*public_key),
            None => Err(Error::MissingPublicKey),
        }
    }

    /// Returns the chain ID used by the wallet
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdk::keys::bip39::Mnemonic;

    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn test_wallet_builder_without_mnemonic() {
        let result = EthereumWallet::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_wallet_creation_from_mnemonic() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        // Known address for this mnemonic at m/44'/60'/0'/0/0
        assert_eq!(
            wallet.public_address().to_lowercase(),
            "0x9858effd232b4033e47d90003d41ec34ecaeda94"
        );
    }

    #[test]
    fn test_wallet_checksummed_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .address_format(EthereumFormat::Checksummed)
            .build()
            .unwrap();

        // Checksummed addresses have mixed case
        let addr = wallet.public_address();
        assert!(addr.starts_with("0x"));
        assert!(addr.chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn test_wallet_non_checksummed_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .address_format(EthereumFormat::NonChecksummed)
            .build()
            .unwrap();

        // Non-checksummed addresses are all lowercase
        let addr = wallet.public_address();
        assert!(addr.starts_with("0x"));
        assert!(!addr[2..].chars().any(|c| c.is_uppercase()));
    }

    #[test]
    fn test_wallet_chain_id() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        // Default chain ID is 1 (mainnet)
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic.clone())
            .build()
            .unwrap();
        assert_eq!(wallet.chain_id(), 1);

        // Custom chain ID
        let wallet_sepolia = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .chain_id(11155111)
            .build()
            .unwrap();
        assert_eq!(wallet_sepolia.chain_id(), 11155111);
    }

    #[test]
    fn test_wallet_receive_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let receive = wallet.receive_address().unwrap();
        assert_eq!(receive, wallet.public_address());
    }

    #[test]
    fn test_wallet_public_key() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let pubkey = wallet.public_key();
        assert!(pubkey.is_ok());
    }

    #[test]
    fn test_ethereum_public_key_to_bytes() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = EthereumWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();

        let pubkey = wallet.public_key().unwrap();
        assert_eq!(pubkey.public_key.serialize().len(), 33); // Compressed pubkey
    }

    #[test]
    fn test_different_mnemonics_different_addresses() {
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
        ).unwrap();

        let wallet1 = EthereumWallet::builder()
            .mnemonic(mnemonic1)
            .build()
            .unwrap();

        let wallet2 = EthereumWallet::builder()
            .mnemonic(mnemonic2)
            .build()
            .unwrap();

        assert_ne!(wallet1.public_address(), wallet2.public_address());
    }
}
