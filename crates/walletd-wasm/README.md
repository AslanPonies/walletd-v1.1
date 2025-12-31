# WalletD WASM 🌐

[![npm](https://img.shields.io/npm/v/walletd-wasm.svg)](https://www.npmjs.com/package/walletd-wasm)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

WebAssembly bindings for the WalletD multi-chain wallet SDK. Generate cryptocurrency addresses, sign messages, and manage wallet keys directly in the browser.

## Features

- 🔐 **Ethereum** - Address generation, message signing, EIP-55 checksums
- ₿ **Bitcoin** - Native SegWit (bech32) address generation
- 🔒 **Monero** - Amount conversions (XMR ↔ piconero)
- 🎲 **Mnemonic** - BIP-39 mnemonic generation and validation
- 🔧 **Utilities** - keccak256, sha256, hex conversions

## Installation

### npm / yarn

```bash
npm install walletd-wasm
# or
yarn add walletd-wasm
```

### Build from source

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for bundlers (webpack, vite, etc.)
wasm-pack build --target bundler

# Build for Node.js
wasm-pack build --target nodejs

# Build for web (no bundler)
wasm-pack build --target web
```

## Usage

### Browser (with bundler)

```javascript
import init, { 
  EthereumWallet, 
  BitcoinKeys,
  MoneroAmount,
  generateMnemonic 
} from 'walletd-wasm';

async function main() {
  // Initialize WASM module
  await init();
  
  // Generate a new mnemonic (12 or 24 words)
  const mnemonic = generateMnemonic(12);
  console.log('Mnemonic:', mnemonic);
  
  // Create Ethereum wallet from mnemonic
  const ethWallet = EthereumWallet.fromMnemonic(mnemonic);
  console.log('ETH Address:', ethWallet.address());
  console.log('ETH Private Key:', ethWallet.privateKey());
  
  // Sign a message
  const signature = ethWallet.signMessage('Hello, World!');
  console.log('Signature:', signature);
  
  // Create Bitcoin keys
  const btcKeys = BitcoinKeys.fromMnemonic(mnemonic, 'mainnet');
  console.log('BTC Address:', btcKeys.address());
  console.log('BTC WIF:', btcKeys.wif());
  
  // Monero amount conversions
  const amount = MoneroAmount.fromXmr(1.5);
  console.log('Piconero:', amount.piconero());
  console.log('Display:', amount.display());
}

main();
```

### Browser (no bundler)

```html
<script type="module">
  import init, { EthereumWallet } from './pkg/walletd_wasm.js';
  
  async function run() {
    await init();
    const wallet = new EthereumWallet();
    console.log('Address:', wallet.address());
  }
  
  run();
</script>
```

### Node.js

```javascript
const { EthereumWallet, generateMnemonic } = require('walletd-wasm');

const mnemonic = generateMnemonic(12);
const wallet = EthereumWallet.fromMnemonic(mnemonic);
console.log('Address:', wallet.address());
```

## API Reference

### Mnemonic Functions

```typescript
// Generate a new BIP-39 mnemonic (12 or 24 words)
function generateMnemonic(wordCount: 12 | 24): string;

// Validate a mnemonic phrase
function validateMnemonic(phrase: string): boolean;
```

### EthereumWallet

```typescript
class EthereumWallet {
  // Create a random wallet
  constructor();
  
  // Create from mnemonic (BIP-44 path: m/44'/60'/0'/0/0)
  static fromMnemonic(mnemonic: string): EthereumWallet;
  
  // Create from private key hex
  static fromPrivateKey(privateKeyHex: string): EthereumWallet;
  
  // Get checksummed address
  address(): string;
  
  // Get private key as hex
  privateKey(): string;
  
  // Get public key as hex (uncompressed)
  publicKey(): string;
  
  // Sign an Ethereum message
  signMessage(message: string): string;
  
  // Export as JSON
  toJson(): { address: string; public_key: string };
}
```

### BitcoinKeys

```typescript
class BitcoinKeys {
  // Create from mnemonic (BIP-84 for native SegWit)
  static fromMnemonic(mnemonic: string, network: 'mainnet' | 'testnet'): BitcoinKeys;
  
  // Get bech32 address (native SegWit)
  address(): string;
  
  // Get network
  network(): string;
  
  // Get WIF private key
  wif(): string;
  
  // Get compressed public key as hex
  publicKey(): string;
}
```

### MoneroAmount

```typescript
class MoneroAmount {
  // Create from XMR
  static fromXmr(xmr: number): MoneroAmount;
  
  // Create from piconero string
  static fromPiconero(piconero: string): MoneroAmount;
  
  // Get as XMR
  xmr(): number;
  
  // Get as piconero string
  piconero(): string;
  
  // Format for display
  display(): string;
  
  // Add amounts
  add(other: MoneroAmount): MoneroAmount;
  
  // Subtract amounts
  sub(other: MoneroAmount): MoneroAmount;
}
```

### Utility Functions

```typescript
// ETH <-> Wei conversions
function ethToWei(eth: number): string;
function weiToEth(wei: string): number;
function gweiToWei(gwei: number): string;

// Hash functions
function keccak256(data: Uint8Array): Uint8Array;
function sha256(data: Uint8Array): Uint8Array;

// Hex conversions
function hexToBytes(hex: string): Uint8Array;
function bytesToHex(bytes: Uint8Array): string;

// Version
function version(): string;
```

## Security Notes

⚠️ **Important Security Considerations:**

1. **Private keys should never be exposed** - The `privateKey()` and `wif()` methods are provided for wallet export/import. Handle with extreme care.

2. **Use secure random** - The WASM module uses `crypto.getRandomValues()` for entropy, which is cryptographically secure in browsers.

3. **No network operations** - This module is purely cryptographic. It does not make network requests or broadcast transactions.

4. **Memory safety** - While we use Rust's memory safety guarantees, be cautious about storing sensitive data in JavaScript variables.

## Building for Production

```bash
# Optimized build with size reduction
wasm-pack build --release --target bundler

# The output will be in ./pkg/
# - walletd_wasm_bg.wasm (WASM binary)
# - walletd_wasm.js (JavaScript glue)
# - walletd_wasm.d.ts (TypeScript definitions)
```

## Browser Compatibility

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## License

MIT OR Apache-2.0
