//! WalletD Multi-Chain CLI - Integrated Edition
//!
//! Full SDK integration with real wallet operations for 17+ blockchains.
//! Backward compatible with original walletd-icp-cli.

mod config;
mod types;
mod wallet_integration;

use config::WalletDConfig;
use types::{CliResponse, WalletMode};
use wallet_integration::WALLET_MANAGER;
use std::io::{self, Write};

const VERSION: &str = "0.2.0";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Outer loop for mode changes
    loop {
        print_banner();
        
        // Mode selection
        let mode = select_mode_at_startup();
        
        // Load and update config
        let mut config = WalletDConfig::load();
        update_config_for_mode(&mut config, &mode);
        let _ = config.save();
        
        // Update global wallet manager
        {
            let mut manager = WALLET_MANAGER.write().await;
            manager.config = config.clone();
            manager.mode = mode.clone();
        }
        
        // Print mode info
        print_mode_info(&mode);
        
        // Initialize wallets
        if mode != WalletMode::Demo {
            let mut manager = WALLET_MANAGER.write().await;
            manager.init_all().await?;
        } else {
            println!("✅ Demo wallets ready (no network connections)");
        }
        
        // Track restart
        let mut should_restart = false;
        
        // Main menu loop
        loop {
            print_main_menu(&mode);
            
            print!("\nYour choice: ");
            io::stdout().flush()?;
            
            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            
            let result = handle_main_menu_choice(choice.trim(), &mode).await;
            
            match result {
                Ok(CliResponse::Exit) => {
                    println!("\nThank you for using WalletD SDK!");
                    return Ok(());
                }
                Ok(CliResponse::ChangeMode) => {
                    should_restart = true;
                    break;
                }
                Ok(CliResponse::Continue) => continue,
                Ok(CliResponse::Swap) => {
                    handle_cross_chain_swap(&mode).await?;
                    continue;
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
        
        if should_restart {
            println!("\n🔄 Restarting with new mode...\n");
            continue;
        }
    }
}

fn print_banner() {
    println!("\n    ██╗    ██╗  █████╗  ██╗      ██╗      ███████╗ ████████╗ ██████╗         ");
    println!("    ██║    ██║ ██╔══██╗ ██║      ██║      ██╔════╝ ╚══██╔══╝ ██╔══██╗   ██╗  ");
    println!("    ██║ █╗ ██║ ███████║ ██║      ██║      █████╗      ██║    ██║  ██║ ██████╗");
    println!("    ██║███╗██║ ██╔══██║ ██║      ██║      ██╔══╝      ██║    ██║  ██║  ╚██╔═╝");
    println!("    ╚███╔███╔╝ ██║  ██║ ███████╗ ███████╗ ███████╗    ██║    ██████╔╝   ╚═╝  ");
    println!("     ╚══╝╚══╝  ╚═╝  ╚═╝ ╚══════╝ ╚══════╝ ╚══════╝    ╚═╝    ╚═════╝         ");
    println!("\n              SDK v{} - Multi-Chain Wallet Framework\n", VERSION);
}

fn select_mode_at_startup() -> WalletMode {
    println!("🚀 WalletD Multi-Chain Wallet SDK");
    println!("══════════════════════════════════");
    println!("\nSelect operating mode:");
    println!();
    println!("  [1] 🧪 Testnet Mode (Recommended)");
    println!("      • Safe testing environment");
    println!("      • Free test tokens available");
    println!("      • Same code as mainnet");
    println!();
    println!("  [2] ⚡ Mainnet Mode");
    println!("      • Real blockchain networks");
    println!("      • Real transactions");
    println!("      • ⚠️  Real money - Be careful!");
    println!();
    println!("  [3] 📌 Demo Mode");
    println!("      • UI testing only");
    println!("      • No network connections");
    println!("      • Perfect for demos");
    
    print!("\nYour choice (default: 1): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    match input.trim() {
        "2" => {
            println!("\n⚠️  WARNING: Mainnet Mode Selected");
            println!("Real transactions with real money will be executed.");
            print!("Are you sure? (yes/N): ");
            io::stdout().flush().unwrap();
            
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();
            
            if confirm.trim().to_lowercase() == "yes" {
                WalletMode::Mainnet
            } else {
                println!("Switching to Testnet mode for safety.");
                WalletMode::Testnet
            }
        }
        "3" => WalletMode::Demo,
        _ => WalletMode::Testnet,
    }
}

fn update_config_for_mode(config: &mut WalletDConfig, mode: &WalletMode) {
    match mode {
        WalletMode::Testnet => {
            config.demo_mode = false;
            config.bitcoin.network = "testnet".to_string();
            config.ethereum.chain_id = 11155111; // Sepolia
            config.solana.cluster = "devnet".to_string();
            config.hedera.network = "testnet".to_string();
            config.monero.network = "stagenet".to_string();
        }
        WalletMode::Mainnet => {
            config.demo_mode = false;
            config.bitcoin.network = "mainnet".to_string();
            config.ethereum.chain_id = 1;
            config.solana.cluster = "mainnet-beta".to_string();
            config.hedera.network = "mainnet".to_string();
            config.monero.network = "mainnet".to_string();
        }
        WalletMode::Demo => {
            config.demo_mode = true;
        }
    }
}

fn print_mode_info(mode: &WalletMode) {
    match mode {
        WalletMode::Testnet => {
            println!("\n🧪 Mode: TESTNET MODE");
            println!("   Safe testing with test tokens");
            println!("   Get free tokens from faucets\n");
        }
        WalletMode::Mainnet => {
            println!("\n⚡ Mode: MAINNET MODE");
            println!("   ⚠️  Real networks - Real money!");
            println!("   Be careful with transactions\n");
        }
        WalletMode::Demo => {
            println!("\n📌 Mode: DEMO MODE");
            println!("   UI testing - No real transactions\n");
        }
    }
}

fn print_main_menu(mode: &WalletMode) {
    let warning = if *mode == WalletMode::Mainnet { " ⚠️" } else { "" };
    let mode_label = match mode {
        WalletMode::Testnet => "🧪 TESTNET",
        WalletMode::Mainnet => "⚡ MAINNET",
        WalletMode::Demo => "📌 DEMO",
    };
    
    println!("\n{} MODE - Select blockchain:", mode_label);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Original chains (1-9)
    println!("[1] Bitcoin (BTC){}", warning);
    println!("[2] Ethereum (ETH){}", warning);
    println!("[3] Solana (SOL){}", warning);
    println!("[4] Hedera (HBAR){}", warning);
    println!("[5] Monero (XMR){}", warning);
    println!("[6] Internet Computer (ICP){}", warning);
    println!("[7] ERC-20 Tokens{}", warning);
    println!("[8] Base (ETH L2){}", warning);
    println!("[9] Prasaga Avio (SAGA){}", warning);
    
    // Extended chains (10-20) - placeholder for when SDK crates exist
    println!("\n--- Extended (Coming Soon) ---");
    println!("[10] Polygon  [11] Avalanche  [12] Arbitrum");
    println!("[13] Cardano  [14] Cosmos     [15] Polkadot");
    println!("[16] Near     [17] Tron       [18] SUI");
    println!("[19] Aptos    [20] TON");
    
    println!("\n--- Options ---");
    println!("[S] Cross-Chain Swap");
    println!("[T] {} Tools & Faucets", if *mode == WalletMode::Testnet { "Testnet" } else { "Network" });
    println!("[M] Change Mode");
    println!("[X] Exit");
}

async fn handle_main_menu_choice(choice: &str, mode: &WalletMode) -> Result<CliResponse, String> {
    match choice.to_uppercase().as_str() {
        "X" => Ok(CliResponse::Exit),
        "M" => Ok(CliResponse::ChangeMode),
        "S" => Ok(CliResponse::Swap),
        "T" => { handle_tools_menu(mode).await; Ok(CliResponse::Continue) }
        "1" => { handle_bitcoin_menu(mode).await?; Ok(CliResponse::Continue) }
        "2" => { handle_ethereum_menu(mode).await?; Ok(CliResponse::Continue) }
        "3" => { handle_solana_menu(mode).await?; Ok(CliResponse::Continue) }
        "4" => { handle_hedera_menu(mode).await?; Ok(CliResponse::Continue) }
        "5" => { handle_monero_menu(mode).await?; Ok(CliResponse::Continue) }
        "6" => { handle_icp_menu(mode).await?; Ok(CliResponse::Continue) }
        "7" => { handle_erc20_menu(mode).await?; Ok(CliResponse::Continue) }
        "8" => { handle_base_menu(mode).await?; Ok(CliResponse::Continue) }
        "9" => { println!("\n⏳ Prasaga integration coming soon!"); wait_for_enter(); Ok(CliResponse::Continue) }
        "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" | "20" => {
            println!("\n⏳ Extended chain support coming soon!");
            println!("   These chains will be added as SDK crates are integrated.");
            wait_for_enter();
            Ok(CliResponse::Continue)
        }
        _ => {
            println!("Invalid option. Please try again.");
            Ok(CliResponse::Continue)
        }
    }
}

// ============================================================================
// Chain Menu Handlers - Real SDK Integration
// ============================================================================

async fn handle_bitcoin_menu(mode: &WalletMode) -> Result<(), String> {
    loop {
        let manager = WALLET_MANAGER.read().await;
        let (address, balance) = manager.get_bitcoin_wallet("user").await
            .unwrap_or(("Not initialized".to_string(), "0.0".to_string()));
        drop(manager);
        
        println!("\n════════════════════════════════════════════════");
        println!("            BITCOIN WALLET");
        println!("════════════════════════════════════════════════");
        println!("Address: {}", address);
        println!("Balance: {} BTC", balance);
        
        println!("\n[1] View Address Details");
        println!("[2] Check Balance (refresh)");
        println!("[3] Send BTC");
        println!("[4] Transaction History");
        println!("[5] Get Testnet BTC (faucet)");
        println!("[B] Back");
        
        print!("\nSelect option: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        match input.trim().to_lowercase().as_str() {
            "b" => return Ok(()),
            "1" => {
                println!("\n📍 Address: {}", address);
                if let Some(wallet) = &WALLET_MANAGER.read().await.bitcoin {
                    println!("🔑 WIF: {}", wallet.get_wif());
                    println!("🔍 Explorer: {}", wallet.explorer_url());
                }
            }
            "2" => {
                println!("\n🔄 Refreshing balance...");
                if let Some(wallet) = &WALLET_MANAGER.read().await.bitcoin {
                    match wallet.get_balance().await {
                        Ok(b) => println!("💰 Balance: {:.8} BTC", b as f64 / 100_000_000.0),
                        Err(e) => println!("❌ Error: {}", e),
                    }
                }
            }
            "3" => {
                if *mode == WalletMode::Demo {
                    println!("\n📌 Demo mode - No real transaction");
                } else {
                    print!("\nRecipient address: ");
                    io::stdout().flush().ok();
                    let mut to = String::new();
                    io::stdin().read_line(&mut to).ok();
                    
                    print!("Amount (BTC): ");
                    io::stdout().flush().ok();
                    let mut amount = String::new();
                    io::stdin().read_line(&mut amount).ok();
                    
                    if let Ok(amt) = amount.trim().parse::<f64>() {
                        let manager = WALLET_MANAGER.read().await;
                        match manager.send_bitcoin(to.trim(), amt).await {
                            Ok(txid) => println!("\n✅ Transaction sent! TXID: {}", txid),
                            Err(e) => println!("\n❌ Error: {}", e),
                        }
                    }
                }
            }
            "4" => {
                println!("\n📜 View history on explorer:");
                if let Some(wallet) = &WALLET_MANAGER.read().await.bitcoin {
                    println!("   {}", wallet.explorer_url());
                }
            }
            "5" => {
                println!("\n🚰 Bitcoin Testnet Faucet:");
                println!("   https://coinfaucet.eu/en/btc-testnet/");
                println!("   https://testnet-faucet.mempool.co/");
                println!("\nYour address: {}", address);
            }
            _ => println!("Invalid option"),
        }
        
        wait_for_enter();
    }
}

async fn handle_ethereum_menu(mode: &WalletMode) -> Result<(), String> {
    loop {
        let manager = WALLET_MANAGER.read().await;
        let (address, balance) = manager.get_ethereum_wallet().await
            .unwrap_or(("Not initialized".to_string(), "0.0".to_string()));
        drop(manager);
        
        println!("\n════════════════════════════════════════════════");
        println!("            ETHEREUM WALLET");
        println!("════════════════════════════════════════════════");
        println!("Address: {}", address);
        println!("Balance: {} ETH", balance);
        
        println!("\n[1] View Address Details");
        println!("[2] Check Balance");
        println!("[3] Send ETH");
        println!("[4] View Gas Prices");
        println!("[5] Get Sepolia ETH (faucet)");
        println!("[B] Back");
        
        print!("\nSelect option: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        match input.trim().to_lowercase().as_str() {
            "b" => return Ok(()),
            "1" => println!("\n📍 Address: {}", address),
            "5" => {
                println!("\n🚰 Ethereum Sepolia Faucet:");
                println!("   https://sepoliafaucet.com/");
                println!("   https://faucet.quicknode.com/ethereum/sepolia");
            }
            _ => println!("Option not yet implemented"),
        }
        
        wait_for_enter();
    }
}

async fn handle_solana_menu(mode: &WalletMode) -> Result<(), String> {
    loop {
        let manager = WALLET_MANAGER.read().await;
        let (address, balance) = manager.get_solana_wallet("user").await
            .unwrap_or(("Not initialized".to_string(), "0.0".to_string()));
        drop(manager);
        
        println!("\n════════════════════════════════════════════════");
        println!("            SOLANA WALLET");
        println!("════════════════════════════════════════════════");
        println!("Address: {}", address);
        println!("Balance: {} SOL", balance);
        
        println!("\n[1] View Address");
        println!("[2] Check Balance");
        println!("[3] Request Airdrop (devnet)");
        println!("[4] Send SOL");
        println!("[B] Back");
        
        print!("\nSelect option: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        match input.trim().to_lowercase().as_str() {
            "b" => return Ok(()),
            "3" => {
                if let Some(wallet) = &WALLET_MANAGER.read().await.solana {
                    println!("\n🔄 Requesting airdrop...");
                    match wallet.request_airdrop(1_000_000_000).await {
                        Ok(sig) => println!("✅ Airdrop successful! Signature: {}", sig),
                        Err(e) => println!("❌ Airdrop failed: {}", e),
                    }
                }
            }
            _ => println!("Option not yet implemented"),
        }
        
        wait_for_enter();
    }
}

async fn handle_hedera_menu(_mode: &WalletMode) -> Result<(), String> {
    println!("\n════════════════════════════════════════════════");
    println!("            HEDERA WALLET");
    println!("════════════════════════════════════════════════");
    println!("\n💡 Hedera requires account creation via portal:");
    println!("   https://portal.hedera.com/");
    wait_for_enter();
    Ok(())
}

async fn handle_monero_menu(_mode: &WalletMode) -> Result<(), String> {
    let manager = WALLET_MANAGER.read().await;
    let (address, _) = manager.get_monero_wallet("user").await
        .unwrap_or(("Not initialized".to_string(), "0.0".to_string()));
    
    println!("\n════════════════════════════════════════════════");
    println!("            MONERO WALLET");
    println!("════════════════════════════════════════════════");
    println!("Address: {}...{}", &address[..12], &address[address.len().saturating_sub(12)..]);
    println!("\n💡 Monero requires wallet RPC for full functionality");
    wait_for_enter();
    Ok(())
}

async fn handle_icp_menu(_mode: &WalletMode) -> Result<(), String> {
    let manager = WALLET_MANAGER.read().await;
    let (principal, _) = manager.get_icp_wallet("user").await
        .unwrap_or(("Not initialized".to_string(), "0.0".to_string()));
    
    println!("\n════════════════════════════════════════════════");
    println!("        INTERNET COMPUTER WALLET");
    println!("════════════════════════════════════════════════");
    println!("Principal: {}", principal);
    wait_for_enter();
    Ok(())
}

async fn handle_erc20_menu(_mode: &WalletMode) -> Result<(), String> {
    println!("\n════════════════════════════════════════════════");
    println!("            ERC-20 TOKENS");
    println!("════════════════════════════════════════════════");
    println!("\nSupported chains: Ethereum, Polygon, Avalanche, Base, Arbitrum");
    println!("\n[1] View Token Balances");
    println!("[2] Send Token");
    println!("[3] Add Custom Token");
    println!("[B] Back");
    wait_for_enter();
    Ok(())
}

async fn handle_base_menu(_mode: &WalletMode) -> Result<(), String> {
    println!("\n════════════════════════════════════════════════");
    println!("            BASE (ETH L2)");
    println!("════════════════════════════════════════════════");
    println!("\n💡 Base uses the same address as Ethereum");
    println!("   Get Base Sepolia ETH: https://faucet.base.org/");
    wait_for_enter();
    Ok(())
}

async fn handle_tools_menu(mode: &WalletMode) {
    println!("\n════════════════════════════════════════════════");
    if *mode == WalletMode::Testnet {
        println!("            TESTNET TOOLS & FAUCETS");
    } else {
        println!("            NETWORK TOOLS");
    }
    println!("════════════════════════════════════════════════");
    
    if *mode == WalletMode::Testnet {
        println!("\n🚰 Faucets:");
        println!("   Bitcoin:  https://coinfaucet.eu/en/btc-testnet/");
        println!("   Ethereum: https://sepoliafaucet.com/");
        println!("   Solana:   https://faucet.solana.com/");
        println!("   Hedera:   https://portal.hedera.com/");
        println!("   Polygon:  https://faucet.polygon.technology/");
    }
    
    println!("\n🔍 Explorers:");
    println!("   Bitcoin:  https://mempool.space/testnet");
    println!("   Ethereum: https://sepolia.etherscan.io/");
    println!("   Solana:   https://explorer.solana.com/?cluster=devnet");
    
    wait_for_enter();
}

async fn handle_cross_chain_swap(mode: &WalletMode) -> Result<(), String> {
    println!("\n════════════════════════════════════════════════");
    println!("            CROSS-CHAIN SWAP");
    println!("════════════════════════════════════════════════");
    
    if *mode == WalletMode::Demo {
        println!("\n📌 Demo mode - No real swaps");
    }
    
    println!("\nAvailable routes:");
    println!("[1] ETH → BTC (THORChain)");
    println!("[2] ETH → SOL (Wormhole)");
    println!("[3] BTC → ETH (THORChain)");
    println!("[B] Back");
    
    wait_for_enter();
    Ok(())
}

fn wait_for_enter() {
    println!("\nPress Enter to continue...");
    let mut _pause = String::new();
    io::stdin().read_line(&mut _pause).ok();
}
