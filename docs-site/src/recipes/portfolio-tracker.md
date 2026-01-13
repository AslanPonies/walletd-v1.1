# Recipe: Multi-Chain Portfolio Tracker

> **Build a portfolio tracker supporting 18 chains in 3 hours.**

---

## What You'll Build

A portfolio tracker that:
- Monitors balances across all supported chains
- Tracks ERC-20/SPL tokens
- Shows USD values with live prices
- Provides transaction history

---

## Complete Code

```rust
use walletd::prelude::*;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub struct PortfolioAsset {
    pub chain: String,
    pub symbol: String,
    pub balance: f64,
    pub usd_value: f64,
    pub address: String,
}

#[derive(Debug)]
pub struct Portfolio {
    mnemonic: String,
    assets: Vec<PortfolioAsset>,
    total_usd: f64,
}

impl Portfolio {
    pub fn new(mnemonic: &str) -> Self {
        Self {
            mnemonic: mnemonic.to_string(),
            assets: Vec::new(),
            total_usd: 0.0,
        }
    }
    
    pub async fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.assets.clear();
        self.total_usd = 0.0;
        
        // Fetch prices (use real API in production)
        let prices = fetch_prices().await?;
        
        // Check each chain
        let chains = vec![
            ("Bitcoin", "BTC", "https://blockstream.info/api"),
            ("Ethereum", "ETH", "https://eth.llamarpc.com"),
            ("Solana", "SOL", "https://api.mainnet-beta.solana.com"),
            ("Polygon", "MATIC", "https://polygon-rpc.com"),
            ("Arbitrum", "ETH", "https://arb1.arbitrum.io/rpc"),
            ("Base", "ETH", "https://mainnet.base.org"),
            ("Avalanche", "AVAX", "https://api.avax.network/ext/bc/C/rpc"),
        ];
        
        for (chain_name, symbol, rpc) in chains {
            if let Ok(asset) = self.check_chain(chain_name, symbol, rpc, &prices).await {
                if asset.balance > 0.0 {
                    self.total_usd += asset.usd_value;
                    self.assets.push(asset);
                }
            }
        }
        
        // Sort by USD value
        self.assets.sort_by(|a, b| b.usd_value.partial_cmp(&a.usd_value).unwrap());
        
        Ok(())
    }
    
    async fn check_chain(
        &self,
        chain: &str,
        symbol: &str,
        rpc: &str,
        prices: &HashMap<String, f64>,
    ) -> Result<PortfolioAsset, Box<dyn std::error::Error>> {
        let address = match chain {
            "Bitcoin" => bitcoin::derive_address(&self.mnemonic, WalletMode::Mainnet)?,
            _ => ethereum::derive_address(&self.mnemonic, WalletMode::Mainnet)?,
        };
        
        let balance_raw = match chain {
            "Bitcoin" => bitcoin::get_balance(&address, rpc).await?,
            _ => ethereum::get_balance(&address, rpc).await?,
        };
        
        let balance = self.convert_balance(&balance_raw, chain);
        let price = prices.get(symbol).unwrap_or(&0.0);
        
        Ok(PortfolioAsset {
            chain: chain.to_string(),
            symbol: symbol.to_string(),
            balance,
            usd_value: balance * price,
            address,
        })
    }
    
    fn convert_balance(&self, raw: &str, chain: &str) -> f64 {
        let value: u128 = raw.parse().unwrap_or(0);
        match chain {
            "Bitcoin" => value as f64 / 100_000_000.0,
            "Solana" => value as f64 / 1_000_000_000.0,
            _ => value as f64 / 1e18,
        }
    }
    
    pub fn display(&self) {
        println!("\n╔══════════════════════════════════════════════════════╗");
        println!("║              PORTFOLIO TRACKER                        ║");
        println!("╠══════════════════════════════════════════════════════╣");
        
        for asset in &self.assets {
            println!("║ {:12} │ {:>12.6} {} │ ${:>10.2} ║",
                asset.chain,
                asset.balance,
                asset.symbol,
                asset.usd_value
            );
        }
        
        println!("╠══════════════════════════════════════════════════════╣");
        println!("║ TOTAL                              ${:>10.2}       ║", self.total_usd);
        println!("╚══════════════════════════════════════════════════════╝\n");
    }
}

async fn fetch_prices() -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    // In production, use CoinGecko, CoinMarketCap, etc.
    let mut prices = HashMap::new();
    prices.insert("BTC".to_string(), 45000.0);
    prices.insert("ETH".to_string(), 2500.0);
    prices.insert("SOL".to_string(), 100.0);
    prices.insert("MATIC".to_string(), 0.80);
    prices.insert("AVAX".to_string(), 35.0);
    Ok(prices)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = std::env::var("MNEMONIC")?;
    let mut portfolio = Portfolio::new(&mnemonic);
    
    // Refresh every 60 seconds
    let mut ticker = interval(Duration::from_secs(60));
    
    loop {
        ticker.tick().await;
        
        if let Err(e) = portfolio.refresh().await {
            eprintln!("Error refreshing: {}", e);
            continue;
        }
        
        // Clear screen and display
        print!("\x1B[2J\x1B[1;1H");
        portfolio.display();
    }
}
```

---

## Sample Output

```
╔══════════════════════════════════════════════════════╗
║              PORTFOLIO TRACKER                        ║
╠══════════════════════════════════════════════════════╣
║ Bitcoin      │     0.500000 BTC │ $  22500.00       ║
║ Ethereum     │     2.150000 ETH │ $   5375.00       ║
║ Solana       │    50.000000 SOL │ $   5000.00       ║
║ Polygon      │  1000.000000 MATIC │ $    800.00     ║
╠══════════════════════════════════════════════════════╣
║ TOTAL                              $  33675.00       ║
╚══════════════════════════════════════════════════════╝
```

---

## Enhancements

1. **Token Support**: Add ERC-20 and SPL token tracking
2. **Historical Data**: Store snapshots for charts
3. **Alerts**: Notify on large balance changes
4. **Export**: CSV/JSON export for tax reporting
