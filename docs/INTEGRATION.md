# Integration Guide

## iOS Integration (Swift)

```swift
import WalletD

// Initialize SDK
WalletD.initialize()

// Create wallet
let result = WalletD.createWallet(mnemonic: "...", chain: "ethereum")

// Get address
let address = WalletD.getAddress(chain: "ethereum", account: 0, index: 0)

// Sign and broadcast
let signed = WalletD.signTransaction(chain: "ethereum", txData: "...")
let txHash = WalletD.broadcast(chain: "ethereum", signedTx: signed)
```

## Android Integration (Kotlin)

```kotlin
import com.walletd.WalletD

// Initialize SDK
WalletD.init()

// Create wallet
val result = WalletD.createWallet(mnemonic = "...", chain = "ethereum")

// Get address
val address = WalletD.getAddress(chain = "ethereum", account = 0, index = 0)

// Sign and broadcast
val signed = WalletD.signTransaction(chain = "ethereum", txData = "...")
val txHash = WalletD.broadcast(chain = "ethereum", signedTx = signed)
```

## Hardware Wallet Integration

```rust
use walletd_hardware::{DeviceManager, DerivationPath};

let manager = DeviceManager::new();

// List connected devices
for device in manager.list_devices() {
    println!("{}: {} ({})", device.id, device.model, device.firmware_version);
}

// Connect to Ledger
let ledger = manager.connect("ledger:2c97:0001")?;

// Get Bitcoin address (BIP-84)
let path = DerivationPath::bip84(0, 0, 0);
let address = ledger.get_address(&path, true).await?; // Display on device

// Sign transaction
let signature = ledger.sign_transaction(&path, &tx_bytes).await?;
```

## Multi-Sig Wallet Setup

```rust
use walletd_multisig::{MultisigWallet, MultisigConfig, SignerInfo, MultisigChain};

// Create 2-of-3 multisig
let config = MultisigConfig {
    threshold: 2,
    total_signers: 3,
    signers: vec![
        SignerInfo { id: "ceo".into(), public_key: ceo_pubkey, weight: 1, label: Some("CEO".into()) },
        SignerInfo { id: "cfo".into(), public_key: cfo_pubkey, weight: 1, label: Some("CFO".into()) },
        SignerInfo { id: "cto".into(), public_key: cto_pubkey, weight: 1, label: Some("CTO".into()) },
    ],
    chain: MultisigChain::Bitcoin,
    timelock: None,
};

let wallet = MultisigWallet::new(config)?;
let address = wallet.address()?; // bc1q...

// Create transaction
let tx_id = wallet.create_transaction(tx_bytes)?;

// Collect signatures
wallet.add_signature(&tx_id, "ceo", ceo_signature)?;
wallet.add_signature(&tx_id, "cfo", cfo_signature)?;

// Finalize and broadcast
assert!(wallet.is_ready(&tx_id));
let final_tx = wallet.finalize(&tx_id)?;
```
