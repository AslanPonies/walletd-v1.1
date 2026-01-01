//! Real Bitcoin Wallet Integration
//!
//! Provides actual Bitcoin wallet operations using the bitcoin crate.
//! This integrates with Blockstream API for balance and transaction broadcasting.

use anyhow::Result;
use bitcoin::{
    consensus::encode::serialize,
    secp256k1::{rand, Message, Secp256k1},
    sighash::{EcdsaSighashType, SighashCache},
    transaction::Version,
    absolute::LockTime,
    Address, Amount, Network, OutPoint, PrivateKey, ScriptBuf, Sequence, 
    Transaction, TxIn, TxOut, Txid, Witness,
};
use serde::Deserialize;
use std::str::FromStr;

/// UTXO information from Blockstream API
#[derive(Debug, Clone, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub status: UtxoStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UtxoStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
}

/// Real Bitcoin wallet with actual blockchain integration
pub struct RealBitcoinWallet {
    pub private_key: PrivateKey,
    pub address: Address,
    pub network: Network,
    pub secp: Secp256k1<bitcoin::secp256k1::All>,
}

impl RealBitcoinWallet {
    /// Create a new Bitcoin wallet with a random key
    pub fn new(network: Network) -> Result<Self> {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut rand::thread_rng());
        let private_key = PrivateKey::new(secret_key, network);
        let public_key = private_key.public_key(&secp);
        let address = Address::p2wpkh(&public_key, network)?;

        Ok(Self {
            private_key,
            address,
            network,
            secp,
        })
    }

    /// Import wallet from WIF private key
    pub fn from_wif(wif: &str) -> Result<Self> {
        let private_key = PrivateKey::from_wif(wif)?;
        let secp = Secp256k1::new();
        let public_key = private_key.public_key(&secp);
        let network = private_key.network;
        let address = Address::p2wpkh(&public_key, network)?;

        Ok(Self {
            private_key,
            address,
            network,
            secp,
        })
    }

    /// Get API base URL for the network
    fn api_base(&self) -> &str {
        match self.network {
            Network::Testnet => "https://blockstream.info/testnet/api",
            Network::Bitcoin => "https://blockstream.info/api",
            _ => "https://blockstream.info/testnet/api",
        }
    }

    /// Get balance from blockchain
    pub async fn get_balance(&self) -> Result<u64> {
        let url = format!("{}/address/{}", self.api_base(), self.address);
        
        let response = reqwest::get(&url).await?;
        let text = response.text().await?;

        #[derive(Deserialize)]
        struct AddressInfo {
            chain_stats: ChainStats,
        }

        #[derive(Deserialize)]
        struct ChainStats {
            funded_txo_sum: u64,
            spent_txo_sum: u64,
        }

        let info: AddressInfo = serde_json::from_str(&text)?;
        let balance = info.chain_stats.funded_txo_sum
            .saturating_sub(info.chain_stats.spent_txo_sum);

        Ok(balance)
    }

    /// Get UTXOs for the wallet
    pub async fn get_utxos(&self) -> Result<Vec<Utxo>> {
        let url = format!("{}/address/{}/utxo", self.api_base(), self.address);
        
        let response = reqwest::get(&url).await?;
        let utxos: Vec<Utxo> = response.json().await?;

        Ok(utxos)
    }

    /// Create and broadcast a transaction
    pub async fn create_and_send_transaction(
        &self,
        to_address: &str,
        amount_sats: u64,
    ) -> Result<String> {
        // Get UTXOs
        let utxos = self.get_utxos().await?;
        if utxos.is_empty() {
            return Err(anyhow::anyhow!(
                "No UTXOs available. Please fund your wallet first."
            ));
        }

        // Parse destination address
        let to_addr = to_address
            .parse::<Address<_>>()
            .map_err(|_| anyhow::anyhow!("Invalid address"))?
            .require_network(self.network)?;

        // Build transaction
        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![],
        };

        // Select UTXOs and add inputs
        let fee = 10000u64; // 10k sats fee
        let mut total_input = 0u64;
        let mut selected_utxos = vec![];

        for utxo in utxos {
            if total_input >= amount_sats + fee {
                break;
            }

            let txid = Txid::from_str(&utxo.txid)?;
            tx.input.push(TxIn {
                previous_output: OutPoint { txid, vout: utxo.vout },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::default(),
            });

            selected_utxos.push(utxo.clone());
            total_input += utxo.value;
        }

        if total_input < amount_sats + fee {
            return Err(anyhow::anyhow!(
                "Insufficient funds. Have: {} sats, Need: {} sats",
                total_input,
                amount_sats + fee
            ));
        }

        // Add output to recipient
        tx.output.push(TxOut {
            value: Amount::from_sat(amount_sats),
            script_pubkey: to_addr.script_pubkey(),
        });

        // Add change output
        if total_input > amount_sats + fee {
            let change = total_input - amount_sats - fee;
            tx.output.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: self.address.script_pubkey(),
            });
        }

        // Sign inputs
        let mut witnesses = vec![];
        {
            let mut sighash_cache = SighashCache::new(&tx);

            for (index, utxo) in selected_utxos.iter().enumerate() {
                let sighash = sighash_cache.p2wpkh_signature_hash(
                    index,
                    &self.address.script_pubkey(),
                    Amount::from_sat(utxo.value),
                    EcdsaSighashType::All,
                )?;

                let message = Message::from_digest_slice(&sighash[..])?;
                let sig = self.secp.sign_ecdsa(&message, &self.private_key.inner);

                let mut witness = Witness::new();
                witness.push_ecdsa_signature(&bitcoin::ecdsa::Signature {
                    sig,
                    hash_ty: EcdsaSighashType::All,
                });
                witness.push(self.private_key.public_key(&self.secp).to_bytes());

                witnesses.push(witness);
            }
        }

        // Apply witnesses
        for (index, witness) in witnesses.into_iter().enumerate() {
            tx.input[index].witness = witness;
        }

        // Serialize and broadcast
        let tx_hex = hex::encode(serialize(&tx));
        let broadcast_url = format!("{}/tx", self.api_base());

        let client = reqwest::Client::new();
        let response = client.post(&broadcast_url).body(tx_hex).send().await?;

        if response.status().is_success() {
            let txid = response.text().await?;
            Ok(txid)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Failed to broadcast: {}", error_text))
        }
    }

    /// Get receive address
    pub fn get_receive_address(&self) -> String {
        self.address.to_string()
    }

    /// Get private key as WIF
    pub fn get_wif(&self) -> String {
        self.private_key.to_wif()
    }

    /// Get explorer URL for an address
    pub fn explorer_url(&self) -> String {
        match self.network {
            Network::Testnet => format!("https://mempool.space/testnet/address/{}", self.address),
            Network::Bitcoin => format!("https://mempool.space/address/{}", self.address),
            _ => String::new(),
        }
    }

    /// Get explorer URL for a transaction
    pub fn tx_explorer_url(&self, txid: &str) -> String {
        match self.network {
            Network::Testnet => format!("https://mempool.space/testnet/tx/{}", txid),
            Network::Bitcoin => format!("https://mempool.space/tx/{}", txid),
            _ => String::new(),
        }
    }
}
