# Installation

## System Requirements

### macOS
```bash
brew install openssl pkg-config
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Ubuntu/Debian
```bash
sudo apt-get install libssl-dev pkg-config build-essential
```

### Windows
Use WSL2 with Ubuntu.

## Add to Your Project

```toml
[dependencies]
walletd = "1.4"
tokio = { version = "1", features = ["full"] }
```

## CLI Installation

```bash
git clone https://github.com/AslanPonies/walletd-v1.1
cd walletd-v1.1/walletd-cli
cargo build --release
./target/release/walletd
```

## Verify Installation

```rust
fn main() {
    let addr = walletd::bitcoin::derive_address(
        "abandon abandon abandon abandon abandon abandon \
         abandon abandon abandon abandon abandon about",
        walletd::types::WalletMode::Testnet
    ).unwrap();
    
    assert!(addr.starts_with("tb1q"));
    println!("✅ WalletD installed successfully!");
}
```
