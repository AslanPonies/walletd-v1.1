# 60-Second Quickstart

## Step 1: Add Dependency

```toml
[dependencies]
walletd = "1.4"
tokio = { version = "1", features = ["full"] }
```

## Step 2: Run This Code

```rust
use walletd::{bitcoin, ethereum};
use walletd::types::WalletMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = "abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon abandon abandon about";
    
    let btc = bitcoin::derive_address(mnemonic, WalletMode::Mainnet)?;
    let eth = ethereum::derive_address(mnemonic, WalletMode::Mainnet)?;
    
    println!("Bitcoin:  {}", btc);
    println!("Ethereum: {}", eth);
    
    let balance = bitcoin::get_balance(
        "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq",
        "https://blockstream.info/api"
    ).await?;
    
    println!("Balance: {} satoshis", balance);
    
    Ok(())
}
```

## Expected Output

```
Bitcoin:  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
Ethereum: 0x9858EfFD232B4033E47d90003D41EC34EcaEda94
Balance: 1234567 satoshis
```

🎉 **You just queried Bitcoin in 3 lines of code!**

## Next Steps

- [Bitcoin Deep Dive](../chains/bitcoin.md)
- [Ethereum Guide](../chains/ethereum.md)
- [Build a Payment Gateway](../recipes/payment-gateway.md)
