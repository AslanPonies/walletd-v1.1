//! WalletD CLI - Multi-chain cryptocurrency wallet
//!
//! Compatible with the original walletd-icp-cli interface
//! Extended to support 17+ blockchain networks

mod config;
mod types;
mod wallet_integration;

use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Select, Input, Confirm};
use std::io::{self, Write};

use config::WalletConfig;
use types::{Chain, WalletMode};
use wallet_integration::WalletManager;

const VERSION: &str = "0.2.1";
const BANNER: &str = r#"
╦ ╦┌─┐┬  ┬  ┌─┐┌┬┐╔╦╗
║║║├─┤│  │  ├┤  │  ║║
╚╩╝┴ ┴┴─┘┴─┘└─┘ ┴ ═╩╝
Multi-Chain Wallet SDK
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Print banner
    println!("{}", BANNER.cyan().bold());
    println!("Version {} - Supporting 17+ Blockchains\n", VERSION.yellow());
    
    // Mode selection
    let mode = select_mode()?;
    
    println!("\n{} Mode Selected\n", mode.as_str().green().bold());
    
    // Create wallet manager
    let mut manager = WalletManager::new(mode)?;
    
    // Initialize wallet (generate or import mnemonic)
    initialize_wallet(&mut manager)?;
    
    // Main menu loop
    loop {
        match main_menu(&manager).await {
            Ok(should_exit) => {
                if should_exit {
                    println!("\n{}", "Thank you for using WalletD!".green());
                    break;
                }
            }
            Err(e) => {
                println!("{} {}", "Error:".red(), e);
            }
        }
    }
    
    Ok(())
}

fn select_mode() -> Result<WalletMode> {
    println!("Select Network Mode:\n");
    
    let options = vec![
        "1. Testnet  - For development and testing",
        "2. Mainnet  - Real transactions (use with caution!)",
        "3. Demo     - Simulated operations",
    ];
    
    let selection = Select::new()
        .items(&options)
        .default(0)
        .interact()?;
    
    Ok(match selection {
        0 => WalletMode::Testnet,
        1 => WalletMode::Mainnet,
        2 => WalletMode::Demo,
        _ => WalletMode::Testnet,
    })
}

fn initialize_wallet(manager: &mut WalletManager) -> Result<()> {
    println!("\n{}", "Wallet Initialization".cyan().bold());
    println!("─────────────────────\n");
    
    let options = vec![
        "1. Generate new wallet (creates new mnemonic)",
        "2. Import existing wallet (enter mnemonic)",
    ];
    
    let selection = Select::new()
        .items(&options)
        .default(0)
        .interact()?;
    
    let mnemonic = match selection {
        0 => {
            // Generate new mnemonic
            let mnemonic = WalletManager::generate_mnemonic()?;
            println!("\n{}", "⚠️  IMPORTANT: Write down your recovery phrase!".yellow().bold());
            println!("{}", "─".repeat(50));
            println!("\n{}\n", mnemonic.green());
            println!("{}", "─".repeat(50));
            println!("{}", "Store this safely - you'll need it to recover your wallet.".yellow());
            
            // Confirm user has saved it
            if !Confirm::new()
                .with_prompt("Have you safely stored your recovery phrase?")
                .default(false)
                .interact()? 
            {
                println!("{}", "Please write down your recovery phrase before continuing.".red());
                return Err(anyhow!("User did not confirm mnemonic backup"));
            }
            
            mnemonic
        }
        1 => {
            // Import existing mnemonic
            println!("\nEnter your 12 or 24 word recovery phrase:");
            let mnemonic: String = Input::new()
                .with_prompt("Mnemonic")
                .interact_text()?;
            
            mnemonic
        }
        _ => return Err(anyhow!("Invalid selection")),
    };
    
    // Initialize all chain wallets from mnemonic
    println!("\n{}", "Initializing wallets...".cyan());
    manager.init_from_mnemonic(&mnemonic)?;
    
    println!("{}", "✓ All wallets initialized successfully!".green());
    
    // Show addresses
    println!("\n{}", "Your Wallet Addresses:".cyan().bold());
    println!("{}", "─".repeat(60));
    
    for chain in Chain::original_chains() {
        if let Some(addr) = manager.get_address(*chain) {
            println!("{:<12} {}", 
                format!("{}:", chain.symbol()).yellow(),
                truncate_address(&addr, 20)
            );
        }
    }
    
    println!("{}", "─".repeat(60));
    println!("{}", "(Extended chains available in chain menu)".dimmed());
    
    Ok(())
}

fn truncate_address(addr: &str, max_len: usize) -> String {
    if addr.len() <= max_len {
        addr.to_string()
    } else {
        format!("{}...{}", &addr[..8], &addr[addr.len()-8..])
    }
}

async fn main_menu(manager: &WalletManager) -> Result<bool> {
    println!("\n{}", "═".repeat(50));
    println!("{}", "Main Menu".cyan().bold());
    println!("{}", "═".repeat(50));
    
    let options = vec![
        " 1. Bitcoin (BTC)".to_string(),
        " 2. Ethereum (ETH)".to_string(),
        " 3. Solana (SOL)".to_string(),
        " 4. Hedera (HBAR)".to_string(),
        " 5. Monero (XMR)".to_string(),
        " 6. Internet Computer (ICP)".to_string(),
        " 7. ERC-20 Tokens".to_string(),
        " 8. Base L2 (BASE)".to_string(),
        " 9. Prasaga (PRA)".to_string(),
        "10. More Chains →".to_string(),
        "─────────────────".to_string(),
        "11. Portfolio Overview".to_string(),
        "12. Tools & Utilities".to_string(),
        "13. Settings".to_string(),
        "14. Exit".to_string(),
    ];
    
    let selection = Select::new()
        .items(&options)
        .default(0)
        .interact()?;
    
    match selection {
        0..=8 => {
            // Original chains (1-9)
            let chain = Chain::from_menu_number((selection + 1) as u8)
                .ok_or_else(|| anyhow!("Invalid chain selection"))?;
            chain_menu(manager, chain).await?;
        }
        9 => {
            // Extended chains menu
            extended_chains_menu(manager).await?;
        }
        10 => {
            // Separator - do nothing
        }
        11 => {
            // Portfolio overview
            portfolio_overview(manager).await?;
        }
        12 => {
            // Tools menu
            tools_menu(manager).await?;
        }
        13 => {
            // Settings
            settings_menu(manager)?;
        }
        14 => {
            // Exit
            return Ok(true);
        }
        _ => {}
    }
    
    Ok(false)
}

async fn chain_menu(manager: &WalletManager, chain: Chain) -> Result<()> {
    loop {
        println!("\n{}", "═".repeat(50));
        println!("{} {}", chain.name().cyan().bold(), format!("({})", chain.symbol()).dimmed());
        println!("{}", "═".repeat(50));
        
        // Show address
        if let Some(addr) = manager.get_address(chain) {
            println!("Address: {}", addr.green());
        }
        
        let options = vec![
            "1. View Balance",
            "2. View Address",
            "3. Send Transaction",
            "4. Receive (Show QR)",
            "5. Transaction History",
            "6. Back to Main Menu",
        ];
        
        let selection = Select::new()
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                // View balance
                println!("\n{}", "Fetching balance...".cyan());
                match manager.get_balance(chain).await {
                    Ok(balance) => println!("Balance: {}", balance.green()),
                    Err(e) => println!("{} {}", "Error:".red(), e),
                }
            }
            1 => {
                // View address
                if let Some(addr) = manager.get_address(chain) {
                    println!("\n{}", "Your Address:".cyan());
                    println!("{}", addr.green());
                    println!("\n{}", "Copy and share this address to receive funds.".dimmed());
                }
            }
            2 => {
                // Send transaction
                send_transaction_menu(manager, chain).await?;
            }
            3 => {
                // Receive - show QR code placeholder
                if let Some(addr) = manager.get_address(chain) {
                    println!("\n{}", "Receive Address:".cyan());
                    println!("{}", addr.green());
                    println!("\n{}", "[QR Code would be displayed here]".dimmed());
                }
            }
            4 => {
                // Transaction history
                println!("\n{}", "Transaction history not yet implemented.".yellow());
            }
            5 => {
                // Back
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

async fn send_transaction_menu(manager: &WalletManager, chain: Chain) -> Result<()> {
    println!("\n{}", "Send Transaction".cyan().bold());
    println!("{}", "─".repeat(30));
    
    // Get recipient address
    let to: String = Input::new()
        .with_prompt("Recipient address")
        .interact_text()?;
    
    // Get amount
    let amount: String = Input::new()
        .with_prompt(format!("Amount ({})", chain.symbol()))
        .interact_text()?;
    
    // Confirm
    println!("\n{}", "Transaction Details:".yellow());
    println!("  To: {}", to);
    println!("  Amount: {} {}", amount, chain.symbol());
    
    if manager.mode.is_mainnet() {
        println!("\n{}", "⚠️  WARNING: This is a MAINNET transaction!".red().bold());
    }
    
    if !Confirm::new()
        .with_prompt("Confirm transaction?")
        .default(false)
        .interact()? 
    {
        println!("{}", "Transaction cancelled.".yellow());
        return Ok(());
    }
    
    // Send transaction
    println!("\n{}", "Sending transaction...".cyan());
    
    match manager.send_transaction(chain, &to, &amount).await {
        Ok(tx_hash) => {
            println!("\n{}", "✓ Transaction sent!".green().bold());
            println!("TX Hash: {}", tx_hash);
        }
        Err(e) => {
            println!("\n{} {}", "Transaction failed:".red(), e);
        }
    }
    
    Ok(())
}

async fn extended_chains_menu(manager: &WalletManager) -> Result<()> {
    loop {
        println!("\n{}", "═".repeat(50));
        println!("{}", "Extended Chains".cyan().bold());
        println!("{}", "═".repeat(50));
        
        let options = vec![
            "10. Polygon (POL)",
            "11. Avalanche (AVAX)",
            "12. Arbitrum (ARB)",
            "13. Cardano (ADA)",
            "14. Cosmos (ATOM)",
            "15. Polkadot (DOT)",
            "16. NEAR Protocol (NEAR)",
            "17. Tron (TRX)",
            "18. Sui (SUI)",
            "19. Aptos (APT)",
            "20. TON",
            "─────────────────",
            "Back to Main Menu",
        ];
        
        let selection = Select::new()
            .items(&options)
            .default(0)
            .interact()?;
        
        if selection == 11 || selection == 12 {
            break;
        }
        
        let chain = match selection {
            0 => Chain::Polygon,
            1 => Chain::Avalanche,
            2 => Chain::Arbitrum,
            3 => Chain::Cardano,
            4 => Chain::Cosmos,
            5 => Chain::Polkadot,
            6 => Chain::Near,
            7 => Chain::Tron,
            8 => Chain::Sui,
            9 => Chain::Aptos,
            10 => Chain::Ton,
            _ => continue,
        };
        
        chain_menu(manager, chain).await?;
    }
    
    Ok(())
}

async fn portfolio_overview(manager: &WalletManager) -> Result<()> {
    println!("\n{}", "═".repeat(60));
    println!("{}", "Portfolio Overview".cyan().bold());
    println!("{}", "═".repeat(60));
    
    println!("\n{}", "Fetching balances across all chains...".cyan());
    
    for chain in Chain::all() {
        if manager.is_initialized(*chain) {
            print!("{:<15}", format!("{}:", chain.symbol()));
            io::stdout().flush()?;
            
            match manager.get_balance(*chain).await {
                Ok(balance) => println!("{}", balance.green()),
                Err(_) => println!("{}", "Unable to fetch".dimmed()),
            }
        }
    }
    
    println!("\n{}", "─".repeat(60));
    println!("{}", "Press Enter to continue...".dimmed());
    
    let _: String = Input::new()
        .allow_empty(true)
        .interact_text()?;
    
    Ok(())
}

async fn tools_menu(manager: &WalletManager) -> Result<()> {
    loop {
        println!("\n{}", "═".repeat(50));
        println!("{}", "Tools & Utilities".cyan().bold());
        println!("{}", "═".repeat(50));
        
        let options = vec![
            "1. Address Book",
            "2. Transaction History (All Chains)",
            "3. Export Addresses",
            "4. Verify Mnemonic",
            "5. Network Status",
            "6. Testnet Faucets",
            "7. Back to Main Menu",
        ];
        
        let selection = Select::new()
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => println!("{}", "Address book not yet implemented.".yellow()),
            1 => println!("{}", "Transaction history not yet implemented.".yellow()),
            2 => {
                // Export addresses
                println!("\n{}", "Your Wallet Addresses:".cyan());
                println!("{}", "─".repeat(60));
                for chain in Chain::all() {
                    if let Some(addr) = manager.get_address(*chain) {
                        println!("{:<12} {}", chain.symbol(), addr);
                    }
                }
            }
            3 => println!("{}", "Mnemonic verification not yet implemented.".yellow()),
            4 => {
                // Network status
                println!("\n{}", "Network Status:".cyan());
                println!("Mode: {} ", manager.mode.as_str().green());
            }
            5 => {
                // Testnet faucets
                if manager.mode.is_testnet() {
                    println!("\n{}", "Testnet Faucets:".cyan());
                    let faucets = vec![
                        ("Bitcoin", "https://testnet-faucet.com/btc-testnet/"),
                        ("Ethereum", "https://sepolia-faucet.pk910.de/"),
                        ("Solana", "https://faucet.solana.com/"),
                        ("Base", "https://www.coinbase.com/faucets"),
                    ];
                    for (name, url) in faucets {
                        println!("{}: {}", name.yellow(), url);
                    }
                } else {
                    println!("{}", "Faucets only available in Testnet mode.".yellow());
                }
            }
            6 => break,
            _ => {}
        }
    }
    
    Ok(())
}

fn settings_menu(manager: &WalletManager) -> Result<()> {
    loop {
        println!("\n{}", "═".repeat(50));
        println!("{}", "Settings".cyan().bold());
        println!("{}", "═".repeat(50));
        
        println!("Current Mode: {}", manager.mode.as_str().green());
        
        let options = vec![
            "1. View Configuration",
            "2. Edit RPC Endpoints",
            "3. Export Configuration",
            "4. Back to Main Menu",
        ];
        
        let selection = Select::new()
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => {
                println!("\n{}", "Configuration:".cyan());
                println!("Config path: {:?}", WalletConfig::config_path());
                println!("Mode: {}", manager.mode.as_str());
            }
            1 => println!("{}", "RPC editing not yet implemented.".yellow()),
            2 => println!("{}", "Export not yet implemented.".yellow()),
            3 => break,
            _ => {}
        }
    }
    
    Ok(())
}
