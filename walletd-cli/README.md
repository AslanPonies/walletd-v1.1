# WalletD CLI v0.2.1

Multi-chain cryptocurrency wallet command-line interface supporting 17+ blockchain networks.

## Features

- **17+ Blockchain Support**: Bitcoin, Ethereum, Solana, Hedera, Monero, ICP, Base, Polygon, Avalanche, Arbitrum, Cardano, Cosmos, Polkadot, NEAR, Tron, SUI, Aptos, TON
- **HD Wallet**: Single mnemonic for all chains (BIP-39/44 compatible)
- **Three Modes**: Testnet, Mainnet, and Demo
- **Interactive UI**: Easy-to-use menu-driven interface
- **Backward Compatible**: Drop-in replacement for `walletd-icp-cli`

## Installation

### From Source

```bash
cd walletd-cli
cargo build --release
```

### Binary Names

Two binaries are produced:
- `walletd` - Primary CLI binary
- `walletd-icp-cli` - Backward-compatible name

## Usage

### Quick Start

```bash
# Run the CLI
./target/release/walletd

# Or using backward-compatible name
./target/release/walletd-icp-cli
```

### Mode Selection

1. **Testnet** - For development and testing (default)
2. **Mainnet** - Real transactions (use with caution!)
3. **Demo** - Simulated operations

### Wallet Initialization

You can either:
1. **Generate new wallet** - Creates a new 12-word mnemonic
2. **Import existing wallet** - Enter your recovery phrase

### Main Menu (Original Chains 1-9)

```
 1. Bitcoin (BTC)
 2. Ethereum (ETH)
 3. Solana (SOL)
 4. Hedera (HBAR)
 5. Monero (XMR)
 6. Internet Computer (ICP)
 7. ERC-20 Tokens
 8. Base L2 (BASE)
 9. Prasaga (PRA)
10. More Chains →
```

### Extended Chains (10-20)

```
10. Polygon (POL)
11. Avalanche (AVAX)
12. Arbitrum (ARB)
13. Cardano (ADA)
14. Cosmos (ATOM)
15. Polkadot (DOT)
16. NEAR Protocol (NEAR)
17. Tron (TRX)
18. Sui (SUI)
19. Aptos (APT)
20. TON
```

### Chain Operations

Each chain menu provides:
- View Balance
- View Address
- Send Transaction
- Receive (Show Address)
- Transaction History

## Configuration

Config file location: `~/.config/walletd/walletd_config.json`

Contains:
- RPC endpoints for all chains
- Testnet faucet URLs
- Wallet entries

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Run in Development

```bash
cargo run --bin walletd
```

## Security

- Mnemonics are never stored unencrypted
- Config files have restricted permissions (0600)
- Mainnet mode requires explicit confirmation
- Private keys are zeroized after use

## Compatibility

This CLI is designed as a drop-in replacement for the original `walletd-icp-cli`:

- Same menu structure (options 1-9)
- Same config file format
- Same binary name available
- Extended with additional chains (10-20)

## License

MIT License - See LICENSE file

## Links

- **SDK Documentation**: https://developer.walletd.org/
- **Token Info**: https://token.walletd.org/
- **Repository**: https://github.com/AslanPonies/walletd-v1.1
