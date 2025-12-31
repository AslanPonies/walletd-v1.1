//! Simple Wallet CLI Example
//!
//! A basic interactive wallet that demonstrates WalletD capabilities.
//!
//! Run with:
//! ```bash
//! cargo run -p walletd --example simple_wallet --features "bitcoin,ethereum"
//! ```

use std::io::{self, Write};

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║         🦀 WalletD Simple Wallet Demo 🦀           ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!();

    loop {
        println!("Available commands:");
        println!("  1. Generate Bitcoin wallet");
        println!("  2. Generate Ethereum wallet");
        println!("  3. Amount conversion demo");
        println!("  4. Exit");
        println!();

        print!("Enter choice (1-4): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "1" => generate_bitcoin_wallet(),
            "2" => generate_ethereum_wallet(),
            "3" => amount_demo(),
            "4" => {
                println!("Goodbye! 👋");
                break;
            }
            _ => println!("Invalid choice, try again.\n"),
        }
    }
}

#[cfg(feature = "bitcoin")]
fn generate_bitcoin_wallet() {
    use bdk::bitcoin::Network;
    use bdk::keys::bip39::Mnemonic;
    use walletd_bitcoin::BitcoinWallet;

    println!("\n━━━ Generating Bitcoin Wallet ━━━\n");

    // Use a test mnemonic (in production, generate a new one securely)
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse(test_mnemonic).expect("Failed to parse mnemonic");

    let wallet = BitcoinWallet::builder()
        .mnemonic(mnemonic)
        .network_type(Network::Testnet)
        .build()
        .expect("Failed to build wallet");

    println!("✅ Bitcoin Wallet Created!");
    println!();
    println!("📬 Receive Address:");
    println!(
        "   {}",
        wallet.receive_address().expect("Failed to get address")
    );
    println!();
    println!("🌐 Network: Testnet");
    println!();
    println!("⚠️  This uses a test mnemonic. Generate a new one for real use!");
    println!();
}

#[cfg(not(feature = "bitcoin"))]
fn generate_bitcoin_wallet() {
    println!("\n❌ Bitcoin feature not enabled.");
    println!("   Run with: --features \"bitcoin\"\n");
}

#[cfg(feature = "ethereum")]
fn generate_ethereum_wallet() {
    use bdk::keys::bip39::Mnemonic;
    use walletd_ethereum::EthereumWallet;

    println!("\n━━━ Generating Ethereum Wallet ━━━\n");

    // Use a test mnemonic (in production, generate a new one securely)
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse(test_mnemonic).expect("Failed to parse mnemonic");

    let wallet = EthereumWallet::builder()
        .mnemonic(mnemonic)
        .chain_id(11155111) // Sepolia
        .build()
        .expect("Failed to build wallet");

    println!("✅ Ethereum Wallet Created!");
    println!();
    println!("📬 Address:");
    println!("   {}", wallet.public_address());
    println!();
    println!("🌐 Chain ID: {} (Sepolia Testnet)", wallet.chain_id());
    println!();
    println!("⚠️  This uses a test mnemonic. Generate a new one for real use!");
    println!();
}

#[cfg(not(feature = "ethereum"))]
fn generate_ethereum_wallet() {
    println!("\n❌ Ethereum feature not enabled.");
    println!("   Run with: --features \"ethereum\"\n");
}

fn amount_demo() {
    println!("\n━━━ Amount Conversion Demo ━━━\n");

    #[cfg(feature = "ethereum")]
    {
        use walletd_ethereum::EthereumAmount;

        println!("Ethereum Amounts:");
        let eth = EthereumAmount::from_eth(1.0);
        println!("  1 ETH = {} Wei", eth.wei());
        println!("  1 ETH = {} Gwei", eth.gwei());

        let gwei = EthereumAmount::from_gwei(21000);
        println!("  21000 Gwei = {} ETH", gwei.eth());
        println!();
    }

    #[cfg(feature = "monero")]
    {
        use walletd_monero::MoneroAmount;

        println!("Monero Amounts:");
        let xmr = MoneroAmount::from_xmr(1.0);
        println!("  1 XMR = {} piconero", xmr.as_piconero());

        let pico = MoneroAmount::from_piconero(1_000_000_000_000);
        println!("  1000000000000 piconero = {} XMR", pico.as_XMR());
        println!();
    }

    #[cfg(not(any(feature = "ethereum", feature = "monero")))]
    {
        println!("No amount features enabled.");
        println!("Run with: --features \"ethereum\" or --features \"monero\"");
        println!();
    }
}
