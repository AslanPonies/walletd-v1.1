use crate::wallet_integration::WALLET_MANAGER;
use alloy::primitives::U256;
use std::io::{self, Write};

/// Format wei to ETH string
fn format_eth(wei: U256) -> String {
    if wei.is_zero() {
        return "0.0".to_string();
    }
    // 1 ETH = 10^18 wei
    let wei_u128: u128 = wei.try_into().unwrap_or(0);
    let eth = wei_u128 as f64 / 1e18;
    format!("{:.6}", eth)
}

/// Parse ETH to wei
fn parse_eth(eth: f64) -> U256 {
    U256::from((eth * 1e18) as u128)
}

pub async fn handle_send_ethereum_real() -> Result<(), String> {
    let manager = WALLET_MANAGER.read().await;

    if let Some(eth_wallet) = &manager.ethereum {
        // Check balance first
        let balance = eth_wallet
            .get_balance()
            .await
            .map_err(|e| format!("Failed to get balance: {e}"))?;

        let balance_eth = format_eth(balance);

        println!("\n=== Send Ethereum ===");
        println!("Network: Sepolia Testnet (Chain ID: 11155111)");
        println!("From: {:?}", eth_wallet.address);
        println!("Balance: {balance_eth} ETH");

        if balance.is_zero() {
            println!("\n⚠️  Your wallet has 0 ETH!");
            println!("To get Sepolia testnet ETH:");
            println!("1. Copy your address: {:?}", eth_wallet.address);
            println!("2. Visit one of these faucets:");
            println!("   - https://sepoliafaucet.com/");
            println!("   - https://faucet.sepolia.dev/");
            println!("   - https://sepolia-faucet.pk910.de/");
            println!("3. Paste your address and request ETH");
            println!("4. Wait a few minutes for confirmation");
            return Ok(());
        }

        print!("To address: ");
        io::stdout().flush().unwrap();
        let mut to_address = String::new();
        io::stdin().read_line(&mut to_address).unwrap();
        let to_address = to_address.trim();

        print!("Amount (ETH): ");
        io::stdout().flush().unwrap();
        let mut amount_str = String::new();
        io::stdin().read_line(&mut amount_str).unwrap();
        let amount: f64 = amount_str.trim().parse().map_err(|_| "Invalid amount")?;

        // Check if user has enough balance
        let amount_wei = parse_eth(amount);
        let gas_estimate = parse_eth(0.001);

        if amount_wei + gas_estimate > balance {
            println!("\n❌ Insufficient funds!");
            println!("You have: {balance_eth} ETH");
            println!("You need: {amount} ETH + ~0.001 ETH for gas");
            return Ok(());
        }

        println!("\n📋 Transaction Summary:");
        println!("From: {:?}", eth_wallet.address);
        println!("To: {to_address}");
        println!("Amount: {amount} ETH");
        println!("Network: Sepolia Testnet");
        println!("Estimated Gas: ~0.001 ETH");
        println!("Total needed: ~{} ETH", amount + 0.001);

        print!("\nConfirm? (yes/no): ");
        io::stdout().flush().unwrap();
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).unwrap();

        if confirm.trim().to_lowercase() == "yes" {
            println!("\n🔐 Signing and broadcasting transaction...");

            match eth_wallet.send_transaction(to_address, amount).await {
                Ok(tx_hash) => {
                    println!("\n✅ TRANSACTION BROADCAST SUCCESSFULLY!");
                    println!("Transaction Hash: {tx_hash}");
                    println!("\n🔍 View on Sepolia Etherscan:");
                    println!("https://sepolia.etherscan.io/tx/{tx_hash}");
                    println!("\n📊 Transaction Details:");
                    println!("- From: {:?}", eth_wallet.address);
                    println!("- To: {to_address}");
                    println!("- Amount: {amount} ETH");
                    println!("- Network: Sepolia Testnet");
                    println!("\n⏳ Transaction is being mined...");
                }
                Err(e) => {
                    println!("\n❌ Transaction failed: {e}");
                    if e.to_string().contains("insufficient funds") {
                        println!("💡 You need more ETH to cover gas fees");
                    } else if e.to_string().contains("nonce") {
                        println!("💡 Try again - there might be a pending transaction");
                    }
                }
            }
        } else {
            println!("Transaction cancelled.");
        }
    } else {
        return Err("Ethereum wallet not initialized".to_string());
    }

    Ok(())
}
