#!/bin/bash
cd /Users/aslanbooboo/Desktop/walletd/docs-site

# Bitcoin
cat > src/chains/bitcoin.md << 'END'
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
END

# Ethereum
cat > src/chains/ethereum.md << 'END'
# Ethereum

Full Ethereum support with EIP-1559 transactions and ERC-20 tokens.

## Features

- EIP-1559 transactions
- ERC-20 token support
- Smart contract calls
- Gas estimation

## Quick Start
```rust
use walletd::ethereum::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Networks

- Mainnet: Chain ID 1
- Sepolia: Chain ID 11155111
END

# Solana
cat > src/chains/solana.md << 'END'
# Solana

High-performance Solana blockchain support.

## Features

- HD wallet derivation
- SPL token support
- Transaction creation
- Staking support

## Quick Start
```rust
use walletd::solana::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# ICP
cat > src/chains/icp.md << 'END'
# Internet Computer (ICP)

DFINITY Internet Computer Protocol support.

## Features

- HD wallet derivation
- ICP transfers
- Rosetta API integration

## Quick Start
```rust
use walletd::icp::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Hedera
cat > src/chains/hedera.md << 'END'
# Hedera

Hedera Hashgraph network support.

## Features

- HD wallet derivation
- HBAR transfers
- Token support

## Quick Start
```rust
use walletd::hedera::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Monero
cat > src/chains/monero.md << 'END'
# Monero

Privacy-focused Monero blockchain support.

## Features

- HD wallet derivation
- Private transactions
- View keys and spend keys

## Quick Start
```rust
use walletd::monero::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Base
cat > src/chains/base.md << 'END'
# Base

Coinbase Base L2 network support.

## Features

- EVM compatible
- Low transaction fees
- Ethereum bridge support

## Quick Start
```rust
use walletd::base::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Network

- Chain ID: 8453
END

# Polygon
cat > src/chains/polygon.md << 'END'
# Polygon

Polygon PoS network support.

## Features

- EVM compatible
- MATIC token support
- Fast transactions

## Quick Start
```rust
use walletd::polygon::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Avalanche
cat > src/chains/avalanche.md << 'END'
# Avalanche

Avalanche C-Chain support.

## Features

- EVM compatible
- AVAX token support
- Sub-second finality

## Quick Start
```rust
use walletd::avalanche::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Arbitrum
cat > src/chains/arbitrum.md << 'END'
# Arbitrum

Arbitrum One L2 support.

## Features

- EVM compatible
- Ethereum L2 scaling
- Low fees

## Quick Start
```rust
use walletd::arbitrum::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Cardano
cat > src/chains/cardano.md << 'END'
# Cardano

Cardano blockchain support.

## Features

- HD wallet derivation
- ADA transfers
- Native tokens

## Quick Start
```rust
use walletd::cardano::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Polkadot
cat > src/chains/polkadot.md << 'END'
# Polkadot

Polkadot relay chain support.

## Features

- HD wallet derivation
- DOT transfers
- Staking support

## Quick Start
```rust
use walletd::polkadot::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Cosmos
cat > src/chains/cosmos.md << 'END'
# Cosmos

Cosmos Hub support.

## Features

- HD wallet derivation
- ATOM transfers
- IBC support

## Quick Start
```rust
use walletd::cosmos::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# NEAR
cat > src/chains/near.md << 'END'
# NEAR

NEAR Protocol support.

## Features

- HD wallet derivation
- NEAR transfers
- Named accounts

## Quick Start
```rust
use walletd::near::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Tron
cat > src/chains/tron.md << 'END'
# Tron

TRON network support.

## Features

- HD wallet derivation
- TRX transfers
- TRC-20 tokens

## Quick Start
```rust
use walletd::tron::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Sui
cat > src/chains/sui.md << 'END'
# Sui

Sui blockchain support.

## Features

- HD wallet derivation
- SUI transfers
- Object-based model

## Quick Start
```rust
use walletd::sui::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Aptos
cat > src/chains/aptos.md << 'END'
# Aptos

Aptos blockchain support.

## Features

- HD wallet derivation
- APT transfers
- Move language

## Quick Start
```rust
use walletd::aptos::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# TON
cat > src/chains/ton.md << 'END'
# TON

The Open Network support.

## Features

- HD wallet derivation
- TON transfers
- Smart contracts

## Quick Start
```rust
use walletd::ton::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```
END

# Prasaga Avio
cat > src/chains/prasaga-avio.md << 'END'
# Prasaga Avio

Prasaga Avio DAG-based blockchain support.

## Features

- HD wallet derivation
- XPRT token transfers
- High throughput

## Quick Start
```rust
use walletd::prasaga::{derive_address, get_balance};

let address = derive_address(&mnemonic, Network::Mainnet)?;
let balance = get_balance(&address).await?;
```

## Integration Status

- Phase 1: Core wallet functionality
- Phase 2: Transaction broadcasting
- Phase 3: Full SDK integration
END

echo "All chain docs created!"
