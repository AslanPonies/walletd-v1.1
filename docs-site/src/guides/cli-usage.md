# CLI Usage

The WalletD CLI provides an interactive interface for managing wallets across all supported chains.

## Launch
```bash
./walletd
```

## Main Menu
```
╔══════════════════════════════════════════════════════════════╗
║                    WalletD CLI v0.2.1                        ║
║              Multi-Chain Cryptocurrency Wallet               ║
╠══════════════════════════════════════════════════════════════╣
║  Supported Chains:                                           ║
║  [1]  Bitcoin      [2]  Ethereum    [3]  Solana             ║
║  [4]  ICP          [5]  Hedera      [6]  Monero             ║
║  [7]  Base         [8]  Polygon     [9]  Avalanche          ║
║  [10] Arbitrum     [11] Cardano     [12] Polkadot           ║
║  [13] Cosmos       [14] NEAR        [15] Tron               ║
║  [16] Sui          [17] Aptos       [18] TON                ║
╚══════════════════════════════════════════════════════════════╝
```

## Network Selection

- **Testnet** - For development and testing (recommended)
- **Mainnet** - Production use with real funds
- **Demo** - Simulated mode, no network connection

## Wallet Operations

### Generate New Wallet
Creates a new HD wallet with a 24-word mnemonic phrase.

### Import Wallet
Import an existing wallet using:
- Mnemonic phrase (12/24 words)
- Private key (hex format)

### Check Balance
Query the current balance for any address.

### Send Transaction
Create and broadcast transactions.

## Examples
```bash
# Generate Bitcoin testnet wallet
./walletd
> Select: 1 (Bitcoin)
> Select: Testnet
> Select: Generate New Wallet

# Check Ethereum balance
./walletd
> Select: 2 (Ethereum)
> Select: Mainnet
> Select: Check Balance
> Enter address: 0x...
```
