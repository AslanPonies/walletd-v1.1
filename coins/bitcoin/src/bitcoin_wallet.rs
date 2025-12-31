use crate::Error;
use bdk::bitcoin::{Address, Txid};
use bdk::blockchain::{Blockchain, GetHeight, WalletSync};
use bdk::keys::bip39::Mnemonic;
use bdk::keys::{DerivableKey, ExtendedKey};
use bdk::template::Bip84;
use bdk::wallet::AddressInfo;
use walletd_hd_key::slip44::Coin;
pub use bdk::bitcoin::AddressType;
use bdk::{bitcoin::Network, database::MemoryDatabase, wallet::AddressIndex, Wallet};
use bdk::{Balance, KeychainKind, SignOptions, SyncOptions};
use std::str::FromStr;
use walletd_hd_key::HDPurpose;

/// Represents a Hierarchical Deterministic (HD) Bitcoin wallet.
pub struct BitcoinWallet {
    wallet: Option<Wallet<MemoryDatabase>>,
    address_format: AddressType,
}

impl Default for BitcoinWallet {
    fn default() -> Self {
        Self {
            wallet: None,
            address_format: AddressType::P2wpkh,
        }
    }
}

impl BitcoinWallet {
    /// Returns the bitcoin balance of the wallet.
    pub async fn balance(&self) -> Result<Balance, Error> {
        let balance = self.wallet.as_ref().unwrap().get_balance().unwrap();
        Ok(balance)
    }

    /// Builds and sends a transaction to the blockchain.
    pub async fn transfer<B: Blockchain>(
        &self,
        blockchain: &B,
        send_amount: u64,
        to_public_address: &str,
    ) -> Result<Txid, Error> {
        let recipient_address = Address::from_str(to_public_address)
            .unwrap()
            .assume_checked();

        let wallet = self.wallet.as_ref().unwrap();
        let mut tx_builder = wallet.build_tx();
        tx_builder
            .add_recipient(recipient_address.script_pubkey(), send_amount)
            .enable_rbf();
        let (mut psbt, tx_details) = tx_builder.finish().unwrap();

        println!("Transaction details: {:#?}", tx_details);

        let finalized = wallet.sign(&mut psbt, SignOptions::default()).unwrap();
        assert!(finalized, "Tx has not been finalized");
        println!("Transaction Signed: {}", finalized);

        let raw_transaction = psbt.extract_tx();
        let txid = raw_transaction.txid();
        blockchain.broadcast(&raw_transaction).unwrap();

        Ok(txid)
    }

    /// Syncs the wallet with the blockchain by adding previously used addresses to the wallet.
    pub async fn sync<B: WalletSync + GetHeight>(&mut self, blockchain: &B) -> Result<(), Error> {
        let _ = self
            .wallet
            .as_mut()
            .unwrap()
            .sync(blockchain, SyncOptions::default());
        Ok(())
    }

    /// Retrieves the next receive address of the wallet.
    pub fn receive_address(&self) -> Result<String, Error> {
        let next_receive_address = self.next_address()?;
        Ok(next_receive_address.to_string())
    }

    /// Returns the coin type id num based on the [Bitcoin network][Network].
    /// Returns an [error][Error] if the network is not supported.
    pub fn coin_type_id(&self) -> Result<u32, Error> {
        match self.network()? {
            Network::Bitcoin => Ok(Coin::Bitcoin.id()),
            Network::Testnet | Network::Regtest => Ok(Coin::Testnet.id()),
            other => Err(Error::CurrentlyNotSupported(format!(
                "Network {} currently not supported",
                other
            ))),
        }
    }

    /// Returns the [default HDPurpose][HDPurpose] based on the [address format][AddressType]
    /// Returns an [error][Error] if the address format is not currently supported
    ///
    /// If the address format is [AddressType::P2pkh] the default purpose is [HDPurpose::BIP44]
    /// If the address format is [AddressType::P2sh] the default purpose is [HDPurpose::BIP49]
    /// If the address format is [AddressType::P2wpkh] the default purpose is [HDPurpose::BIP84]
    /// Other address formats are currently not supported and will return an [error][Error]
    pub fn default_hd_purpose(&self) -> Result<HDPurpose, Error> {
        match self.address_format() {
            AddressType::P2pkh => Ok(HDPurpose::BIP44),
            AddressType::P2sh => Ok(HDPurpose::BIP49),
            AddressType::P2wpkh => Ok(HDPurpose::BIP84),
            other => Err(Error::CurrentlyNotSupported(format!(
                "Address format {} currently not supported",
                other
            ))),
        }
    }

    /// Returns the address format
    pub fn address_format(&self) -> AddressType {
        self.address_format
    }

    /// Returns the network based on the master HDKey
    pub fn network(&self) -> Result<Network, Error> {
        match &self.wallet {
            Some(wallet) => Ok(wallet.network()),
            None => Err(Error::MissingNetwork),
        }
    }

    /// Returns a [AddressInfo] object on the the next available address on the first account (account_index = 0).
    ///
    /// Returns an [error][Error] with details if it encounters a problem while deriving the next address
    pub fn next_address(&self) -> Result<AddressInfo, Error> {
        let address = self
            .wallet
            .as_ref()
            .unwrap()
            .get_address(AddressIndex::New)
            .unwrap();
        Ok(address)
    }

    /// Returns the Builder for [BitcoinWallet]
    pub fn builder() -> BitcoinWalletBuilder {
        BitcoinWalletBuilder::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Builder for [BitcoinWallet] that allows for the creation of a [BitcoinWallet] with a custom configuration
pub struct BitcoinWalletBuilder {
    /// The address format used to generate the wallet, if the address format is not provided, the default address format is P2wpkh
    address_format: AddressType,
    /// The HD purpose used to generate the wallet
    hd_purpose: Option<HDPurpose>,
    /// The mnemonic seed used to import the wallet
    mnemonic: Option<Mnemonic>,
    /// The default network type is Network::Bitcoin
    network_type: Network,
}

impl Default for BitcoinWalletBuilder {
    fn default() -> Self {
        Self {
            address_format: AddressType::P2wpkh,
            hd_purpose: Some(HDPurpose::BIP84),
            mnemonic: None,
            network_type: Network::Bitcoin,
        }
    }
}

impl BitcoinWalletBuilder {
    /// Generates a new BitcoinWalletBuilder with the default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows specification of the mnemonic seed for the wallet
    pub fn mnemonic(&mut self, mnemonic: Mnemonic) -> &mut Self {
        self.mnemonic = Some(mnemonic);
        self
    }

    /// Allows specification of the address format to use for the wallet
    pub fn address_format(&mut self, address_format: AddressType) -> &mut Self {
        self.address_format = address_format;
        self
    }

    /// Allows specification of the network type for the wallet, the default is Network::Bitcoin
    pub fn network_type(&mut self, network_type: Network) -> &mut Self {
        self.network_type = network_type;
        self
    }

    /// Used to import an existing wallet from a mnemonic seed and specified network type
    pub fn build(&self) -> Result<BitcoinWallet, Error> {
        if self.mnemonic.is_none() {
            return Err(Error::MissingMnemonicSeed);
        }
        let mnemonic_words = self.mnemonic.clone();
        let mnemonic = Mnemonic::parse(mnemonic_words.unwrap().to_string()).unwrap();

        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(self.network_type).unwrap();
        let wallet: Wallet<MemoryDatabase> = Wallet::new(
            Bip84(xprv, KeychainKind::External),
            Some(Bip84(xprv, KeychainKind::Internal)),
            self.network_type,
            MemoryDatabase::new(),
        )
        .unwrap();

        let wall = BitcoinWallet {
            wallet: Some(wallet),
            address_format: self.address_format,
        };

        Ok(wall)
    }

    /// Returns the default HDPurpose based on the address format
    /// Returns an error[Error] if the address format is not currently supported
    pub fn default_hd_purpose(&self) -> Result<HDPurpose, Error> {
        match self.address_format {
            AddressType::P2pkh => Ok(HDPurpose::BIP44),
            AddressType::P2sh => Ok(HDPurpose::BIP49),
            AddressType::P2wpkh => Ok(HDPurpose::BIP84),
            other => Err(Error::CurrentlyNotSupported(format!(
                "Address format {} currently not supported",
                other
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test mnemonic (DO NOT USE IN PRODUCTION)
    const TEST_MNEMONIC: &str = "outer ride neither foil glue number place usage ball shed dry point";

    // ============================================================================
    // Default and Builder Tests
    // ============================================================================

    #[test]
    fn test_default() -> Result<(), Error> {
        let expected_default = BitcoinWallet {
            wallet: None,
            address_format: AddressType::P2wpkh,
        };
        let wallet = BitcoinWallet::default();
        assert_eq!(wallet.address_format, expected_default.address_format);
        Ok(())
    }

    #[test]
    fn test_default_builder() {
        let btc_builder = BitcoinWallet::builder();
        let default_btc_builder = BitcoinWalletBuilder::default();
        assert_eq!(btc_builder, default_btc_builder);
    }

    #[test]
    fn test_new() -> Result<(), Error> {
        let builder = BitcoinWalletBuilder::new();
        let default = BitcoinWalletBuilder::default();
        assert_eq!(builder.address_format, default.address_format);
        assert!(builder.mnemonic.is_none());
        assert_eq!(builder.network_type, default.network_type);
        Ok(())
    }

    #[test]
    fn test_with_mnemonic_seed() -> Result<(), Error> {
        let mnemonic_phrase = "outer ride neither foil glue number place usage ball shed dry point";
        let mnemonic = Mnemonic::parse(mnemonic_phrase).unwrap();
        let mut builder = BitcoinWalletBuilder::default();
        builder.mnemonic(mnemonic.clone());
        assert!(builder.mnemonic.is_some());
        assert_eq!(
            builder
                .mnemonic
                .clone()
                .expect("should be some due to previous check"),
            mnemonic
        );
        Ok(())
    }

    #[test]
    fn test_with_address_format() -> Result<(), Error> {
        let mut builder = BitcoinWalletBuilder::default();
        builder.address_format(AddressType::P2pkh);
        assert_eq!(builder.address_format, AddressType::P2pkh);
        Ok(())
    }

    #[test]
    fn test_with_network_type() -> Result<(), Error> {
        let mut builder = BitcoinWalletBuilder::default();
        builder.network_type(bdk::bitcoin::Network::Testnet);
        assert_eq!(builder.network_type, bdk::bitcoin::Network::Testnet);
        Ok(())
    }

    // ============================================================================
    // Wallet Creation Tests
    // ============================================================================

    #[test]
    fn test_build_wallet_from_mnemonic() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .expect("Failed to build wallet");
        
        assert!(wallet.wallet.is_some());
        assert_eq!(wallet.address_format, AddressType::P2wpkh);
    }

    #[test]
    fn test_build_wallet_without_mnemonic_fails() {
        let result = BitcoinWallet::builder().build();
        assert!(result.is_err());
        
        if let Err(Error::MissingMnemonicSeed) = result {
            // Expected error
        } else {
            panic!("Expected MissingMnemonicSeed error");
        }
    }

    #[test]
    fn test_wallet_mainnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Bitcoin)
            .build()
            .unwrap();
        
        assert_eq!(wallet.network().unwrap(), Network::Bitcoin);
    }

    #[test]
    fn test_wallet_testnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Testnet)
            .build()
            .unwrap();
        
        assert_eq!(wallet.network().unwrap(), Network::Testnet);
    }

    #[test]
    fn test_wallet_regtest() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Regtest)
            .build()
            .unwrap();
        
        assert_eq!(wallet.network().unwrap(), Network::Regtest);
    }

    // ============================================================================
    // Address Generation Tests
    // ============================================================================

    #[test]
    fn test_receive_address() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();
        
        let address = wallet.receive_address().expect("Failed to get address");
        // Native SegWit addresses start with bc1 on mainnet
        assert!(address.starts_with("bc1"), "Expected bc1 prefix, got: {}", address);
    }

    #[test]
    fn test_receive_address_testnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Testnet)
            .build()
            .unwrap();
        
        let address = wallet.receive_address().expect("Failed to get address");
        // Native SegWit addresses start with tb1 on testnet
        assert!(address.starts_with("tb1"), "Expected tb1 prefix, got: {}", address);
    }

    #[test]
    fn test_deterministic_address_generation() {
        let mnemonic1 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let mnemonic2 = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        
        let wallet1 = BitcoinWallet::builder()
            .mnemonic(mnemonic1)
            .build()
            .unwrap();
        
        let wallet2 = BitcoinWallet::builder()
            .mnemonic(mnemonic2)
            .build()
            .unwrap();
        
        // Same mnemonic should produce same first address
        assert_eq!(
            wallet1.receive_address().unwrap(),
            wallet2.receive_address().unwrap()
        );
    }

    #[test]
    fn test_next_address_increments() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();
        
        let addr1 = wallet.next_address().unwrap();
        let addr2 = wallet.next_address().unwrap();
        
        // Each call should produce a different address
        assert_ne!(addr1.address, addr2.address);
        assert_eq!(addr1.index, 0);
        assert_eq!(addr2.index, 1);
    }

    // ============================================================================
    // Coin Type and HD Purpose Tests
    // ============================================================================

    #[test]
    fn test_coin_type_id_mainnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Bitcoin)
            .build()
            .unwrap();
        
        // Bitcoin mainnet coin type is 0
        assert_eq!(wallet.coin_type_id().unwrap(), 0);
    }

    #[test]
    fn test_coin_type_id_testnet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .network_type(Network::Testnet)
            .build()
            .unwrap();
        
        // Testnet coin type is 1
        assert_eq!(wallet.coin_type_id().unwrap(), 1);
    }

    #[test]
    fn test_default_hd_purpose_p2wpkh() {
        let wallet = BitcoinWallet::default();
        assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP84);
    }

    #[test]
    fn test_default_hd_purpose_p2pkh() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .address_format(AddressType::P2pkh)
            .build()
            .unwrap();
        
        assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP44);
    }

    #[test]
    fn test_default_hd_purpose_p2sh() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .address_format(AddressType::P2sh)
            .build()
            .unwrap();
        
        assert_eq!(wallet.default_hd_purpose().unwrap(), HDPurpose::BIP49);
    }

    // ============================================================================
    // Address Format Tests
    // ============================================================================

    #[test]
    fn test_address_format_default() {
        let wallet = BitcoinWallet::default();
        assert_eq!(wallet.address_format(), AddressType::P2wpkh);
    }

    #[test]
    fn test_address_format_preserved() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .address_format(AddressType::P2pkh)
            .build()
            .unwrap();
        
        assert_eq!(wallet.address_format(), AddressType::P2pkh);
    }

    // ============================================================================
    // Network Error Tests
    // ============================================================================

    #[test]
    fn test_network_missing_when_no_wallet() {
        let wallet = BitcoinWallet::default();
        let result = wallet.network();
        assert!(result.is_err());
        
        if let Err(Error::MissingNetwork) = result {
            // Expected error
        } else {
            panic!("Expected MissingNetwork error");
        }
    }

    // ============================================================================
    // Builder Default HD Purpose Tests
    // ============================================================================

    #[test]
    fn test_builder_default_hd_purpose_p2wpkh() {
        let builder = BitcoinWalletBuilder::new();
        assert_eq!(builder.default_hd_purpose().unwrap(), HDPurpose::BIP84);
    }

    #[test]
    fn test_builder_default_hd_purpose_p2pkh() {
        let mut builder = BitcoinWalletBuilder::new();
        builder.address_format(AddressType::P2pkh);
        assert_eq!(builder.default_hd_purpose().unwrap(), HDPurpose::BIP44);
    }

    #[test]
    fn test_builder_default_hd_purpose_p2sh() {
        let mut builder = BitcoinWalletBuilder::new();
        builder.address_format(AddressType::P2sh);
        assert_eq!(builder.default_hd_purpose().unwrap(), HDPurpose::BIP49);
    }

    // ============================================================================
    // Balance Tests (without blockchain)
    // ============================================================================

    #[tokio::test]
    async fn test_balance_empty_wallet() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build()
            .unwrap();
        
        let balance = wallet.balance().await.unwrap();
        // New wallet should have zero balance
        assert_eq!(balance.confirmed, 0);
        assert_eq!(balance.immature, 0);
        assert_eq!(balance.trusted_pending, 0);
        assert_eq!(balance.untrusted_pending, 0);
    }

    // ============================================================================
    // Mnemonic Parsing Tests  
    // ============================================================================

    #[test]
    fn test_mnemonic_12_words() {
        let mnemonic = Mnemonic::parse(TEST_MNEMONIC).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_mnemonic_24_words() {
        let mnemonic_24 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let mnemonic = Mnemonic::parse(mnemonic_24).unwrap();
        let wallet = BitcoinWallet::builder()
            .mnemonic(mnemonic)
            .build();
        assert!(wallet.is_ok());
    }

    #[test]
    fn test_invalid_mnemonic() {
        let result = Mnemonic::parse("invalid mnemonic phrase that should not work");
        assert!(result.is_err());
    }
}