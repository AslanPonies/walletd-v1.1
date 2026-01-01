//! Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDConfig {
    pub bitcoin: BitcoinConfig,
    pub ethereum: EthereumConfig,
    pub solana: SolanaConfig,
    pub monero: MoneroConfig,
    pub hedera: HederaConfig,
    pub icp: IcpConfig,
    #[serde(default)]
    pub demo_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub network: String,
    pub rpc_url: String,
    #[serde(default)]
    pub rpc_user: Option<String>,
    #[serde(default)]
    pub rpc_password: Option<String>,
    #[serde(default)]
    pub electrum_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    #[serde(default)]
    pub etherscan_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub cluster: String,
    pub rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoneroConfig {
    pub network: String,
    pub daemon_url: String,
    #[serde(default)]
    pub wallet_rpc_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HederaConfig {
    pub network: String,
    pub operator_id: String,
    pub operator_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcpConfig {
    pub network: String,
    #[serde(default)]
    pub identity_path: Option<String>,
}

impl Default for WalletDConfig {
    fn default() -> Self {
        Self {
            bitcoin: BitcoinConfig {
                network: "testnet".to_string(),
                rpc_url: "http://localhost:18332".to_string(),
                rpc_user: None,
                rpc_password: None,
                electrum_url: Some("ssl://electrum.blockstream.info:60002".to_string()),
            },
            ethereum: EthereumConfig {
                chain_id: 11155111,
                rpc_url: "https://rpc.sepolia.org".to_string(),
                etherscan_api_key: None,
            },
            solana: SolanaConfig {
                cluster: "devnet".to_string(),
                rpc_url: "https://api.devnet.solana.com".to_string(),
            },
            monero: MoneroConfig {
                network: "stagenet".to_string(),
                daemon_url: "http://localhost:38081".to_string(),
                wallet_rpc_url: None,
            },
            hedera: HederaConfig {
                network: "testnet".to_string(),
                operator_id: "0.0.0".to_string(),
                operator_key: String::new(),
            },
            icp: IcpConfig {
                network: "local".to_string(),
                identity_path: None,
            },
            demo_mode: false,
        }
    }
}

impl WalletDConfig {
    pub fn load() -> Self {
        std::fs::read_to_string("walletd_config.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write("walletd_config.json", json)
    }
}
