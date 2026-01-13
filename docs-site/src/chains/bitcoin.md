# Bitcoin

WalletD provides comprehensive Bitcoin support including SegWit, Taproot, and multi-signature wallets.

## Features

- BIP-32/39/44 HD wallets
- SegWit (P2WPKH, P2WSH)
- Taproot (P2TR)
- Multi-signature (P2SH, P2WSH)

## Quick Start
```rust
use walletd::bitcoin::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Address Types

| Type | Prefix | Description |
|------|--------|-------------|
| P2PKH | 1... | Legacy |
| P2SH | 3... | Script Hash |
| P2WPKH | bc1q... | Native SegWit |
| P2TR | bc1p... | Taproot |
