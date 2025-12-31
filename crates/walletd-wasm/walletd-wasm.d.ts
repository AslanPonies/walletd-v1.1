// TypeScript definitions for walletd-wasm
// These are generated alongside wasm-pack build but provided here for reference

/**
 * Initialize the WASM module. Must be called before using any other functions.
 */
export function init(): Promise<void>;

/**
 * Get the WalletD WASM version
 */
export function version(): string;

/**
 * Generate a BIP-39 mnemonic phrase
 * @param wordCount - Number of words (12 or 24)
 * @returns Space-separated mnemonic phrase
 */
export function generateMnemonic(wordCount: number): string;

/**
 * Validate a mnemonic phrase
 * @param phrase - The mnemonic phrase to validate
 * @returns true if valid, false otherwise
 */
export function validateMnemonic(phrase: string): boolean;

/**
 * Convert hex string to bytes
 */
export function hexToBytes(hex: string): Uint8Array;

/**
 * Convert bytes to hex string
 */
export function bytesToHex(bytes: Uint8Array): string;

/**
 * Compute keccak256 hash
 */
export function keccak256(data: Uint8Array): Uint8Array;

/**
 * Compute SHA256 hash
 */
export function sha256(data: Uint8Array): Uint8Array;

/**
 * Convert ETH to Wei
 */
export function ethToWei(eth: number): string;

/**
 * Convert Wei to ETH
 */
export function weiToEth(wei: string): number;

/**
 * Convert Gwei to Wei
 */
export function gweiToWei(gwei: number): string;

/**
 * Ethereum wallet for browser environments
 */
export class EthereumWallet {
  /**
   * Create a new random Ethereum wallet
   */
  constructor();
  
  /**
   * Create wallet from mnemonic phrase (BIP-44: m/44'/60'/0'/0/0)
   * @param mnemonic - BIP-39 mnemonic phrase
   */
  static fromMnemonic(mnemonic: string): EthereumWallet;
  
  /**
   * Create wallet from private key hex string
   * @param privateKeyHex - Private key as hex (with or without 0x prefix)
   */
  static fromPrivateKey(privateKeyHex: string): EthereumWallet;
  
  /**
   * Get the wallet address (EIP-55 checksummed)
   */
  address(): string;
  
  /**
   * Get the private key as hex string (with 0x prefix)
   */
  privateKey(): string;
  
  /**
   * Get the public key as hex string (uncompressed, without 0x04 prefix)
   */
  publicKey(): string;
  
  /**
   * Sign an Ethereum message
   * @param message - Message to sign
   * @returns Signature as hex string
   */
  signMessage(message: string): string;
  
  /**
   * Export wallet as JSON (excludes private key)
   */
  toJson(): { address: string; public_key: string };
}

/**
 * Bitcoin key pair for address generation
 */
export class BitcoinKeys {
  /**
   * Create Bitcoin keys from mnemonic (BIP-84 for native SegWit)
   * @param mnemonic - BIP-39 mnemonic phrase
   * @param network - "mainnet" or "testnet"
   */
  static fromMnemonic(mnemonic: string, network: string): BitcoinKeys;
  
  /**
   * Get the Bitcoin address (bech32/native SegWit)
   */
  address(): string;
  
  /**
   * Get the network ("mainnet" or "testnet")
   */
  network(): string;
  
  /**
   * Get the WIF (Wallet Import Format) private key
   */
  wif(): string;
  
  /**
   * Get compressed public key as hex
   */
  publicKey(): string;
}

/**
 * Monero amount handling (XMR has 12 decimal places)
 */
export class MoneroAmount {
  /**
   * Create from XMR (as floating point)
   */
  static fromXmr(xmr: number): MoneroAmount;
  
  /**
   * Create from piconero (as string for precision)
   */
  static fromPiconero(piconero: string): MoneroAmount;
  
  /**
   * Get amount as XMR (floating point)
   */
  xmr(): number;
  
  /**
   * Get amount as piconero (string for precision)
   */
  piconero(): string;
  
  /**
   * Format for display (e.g., "1.500000000000 XMR")
   */
  display(): string;
  
  /**
   * Add two amounts
   */
  add(other: MoneroAmount): MoneroAmount;
  
  /**
   * Subtract amount (returns 0 if would underflow)
   */
  sub(other: MoneroAmount): MoneroAmount;
}

export default init;
