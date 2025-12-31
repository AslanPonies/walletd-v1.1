//! Multi-Chain Wallet Example
//!
//! This example demonstrates how to use WalletD with multiple blockchains.
//!
//! Run with:
//! ```bash
//! cargo run -p walletd --example multi_chain --features "bitcoin,ethereum,monero"
//! ```

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🦀 WalletD Multi-Chain Example\n");

    // Show which chains are enabled
    #[cfg(feature = "bitcoin")]
    bitcoin_example()?;

    #[cfg(feature = "ethereum")]
    ethereum_example()?;

    #[cfg(feature = "monero")]
    monero_example()?;

    #[cfg(not(any(feature = "bitcoin", feature = "ethereum", feature = "monero")))]
    {
        println!("No chain features enabled! Try:");
        println!("  cargo run -p walletd --example multi_chain --features \"bitcoin,ethereum\"");
    }

    Ok(())
}

#[cfg(feature = "bitcoin")]
fn bitcoin_example() -> Result<(), Box<dyn std::error::Error>> {
    use bdk::bitcoin::Network;
    use bdk::keys::bip39::Mnemonic;
    use walletd_bitcoin::BitcoinWallet;
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("₿ Bitcoin Wallet");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Use a test mnemonic (NEVER use this in production!)
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse(test_mnemonic)?;

    // Create wallet from mnemonic
    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()?;

    println!("Network: Testnet");
    println!("Address: {}", wallet.receive_address()?);
    println!("HD Purpose: {:?}", wallet.default_hd_purpose()?);
    println!();

    Ok(())
}

#[cfg(feature = "ethereum")]
fn ethereum_example() -> Result<(), Box<dyn std::error::Error>> {
    use bdk::keys::bip39::Mnemonic;
    use walletd_ethereum::{EthereumAmount, EthereumWallet};

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Ξ Ethereum Wallet");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Use a test mnemonic (NEVER use this in production!)
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse(test_mnemonic)?;

    // Create wallet from mnemonic
    let wallet = EthereumWallet::builder()
        .mnemonic(mnemonic)
        .chain_id(11155111) // Sepolia testnet
        .build()?;

    println!("Address: {}", wallet.public_address());
    println!("Chain ID: {} (Sepolia)", wallet.chain_id());

    // Demo amount conversions
    let amount = EthereumAmount::from_eth(1.5);
    println!("\nAmount Conversion Demo:");
    println!("  1.5 ETH = {} Gwei", amount.gwei());
    println!("  1.5 ETH = {} Wei", amount.wei());
    println!();

    Ok(())
}

#[cfg(feature = "monero")]
fn monero_example() -> Result<(), Box<dyn std::error::Error>> {
    use walletd_monero::MoneroAmount;
    use walletd_monero::MoneroPrivateKeys;
    use walletd_monero::MoneroPublicKeys;
    use walletd_monero::Address;
    use walletd_monero::AddressType;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ɱ Monero Wallet");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Generate keys from a seed
    let seed = [0x42u8; 32]; // Example seed (use secure random in production!)
    let private_keys = MoneroPrivateKeys::from_seed(&seed)?;
    let public_keys = MoneroPublicKeys::from_private_keys(&private_keys);

    // Create address
    let address = Address::new(&monero::Network::Mainnet, &public_keys, &AddressType::Standard)?;

    println!("Network: Mainnet");
    println!("Address: {}", address);

    // Demo amount conversions
    let amount = MoneroAmount::from_xmr(2.5);
    println!("\nAmount Conversion Demo:");
    println!("  2.5 XMR = {} piconero", amount.as_piconero());
    println!();

    Ok(())
}
