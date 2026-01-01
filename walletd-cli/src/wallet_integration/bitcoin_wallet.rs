//! Bitcoin Wallet - Real Implementation
//!
//! Features: HD derivation (BIP-84), balance checking, transaction broadcasting

use super::hd_derivation::{self, paths};
use anyhow::Result;
use bitcoin::{
    bip32::{DerivationPath, Xpriv, Xpub},
    consensus::encode::serialize,
    secp256k1::{Message, Secp256k1},
    sighash::{EcdsaSighashType, SighashCache},
    transaction::Version,
    absolute::LockTime,
    Address, Amount, Network, OutPoint, PrivateKey, PublicKey,
    ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
}

pub struct BitcoinWallet {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: String,
    pub network: Network,
    secp: Secp256k1<bitcoin::secp256k1::All>,
}

impl BitcoinWallet {
    /// Create new random wallet
    pub fn new(network: Network) -> Result<Self> {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut rand::thread_rng());
        let private_key = PrivateKey::new(secret_key, network);
        let public_key = private_key.public_key(&secp);
        let address = Address::p2wpkh(&public_key, network)?;

        Ok(Self {
            private_key,
            public_key,
            address: address.to_string(),
            network,
            secp,
        })
    }

    /// Create wallet from mnemonic using BIP-84 derivation
    pub fn from_mnemonic(mnemonic: &str, network: Network) -> Result<Self> {
        let secp = Secp256k1::new();
        let master = hd_derivation::derive_bitcoin_xpriv(mnemonic, network)?;
        
        let path: DerivationPath = paths::BITCOIN.parse()?;
        let derived = master.derive_priv(&secp, &path)?;
        
        let private_key = derived.to_priv();
        let public_key = private_key.public_key(&secp);
        let address = Address::p2wpkh(&public_key, network)?;

        Ok(Self {
            private_key,
            public_key,
            address: address.to_string(),
            network,
            secp,
        })
    }

    /// Import from WIF
    pub fn from_wif(wif: &str) -> Result<Self> {
        let private_key = PrivateKey::from_wif(wif)?;
        let secp = Secp256k1::new();
        let public_key = private_key.public_key(&secp);
        let network = private_key.network;
        let address = Address::p2wpkh(&public_key, network)?;

        Ok(Self {
            private_key,
            public_key,
            address: address.to_string(),
            network,
            secp,
        })
    }

    fn api_url(&self) -> &str {
        match self.network {
            Network::Bitcoin => "https://blockstream.info/api",
            Network::Testnet => "https://blockstream.info/testnet/api",
            _ => "https://blockstream.info/testnet/api",
        }
    }

    /// Get balance in satoshis
    pub async fn get_balance(&self) -> Result<u64> {
        let url = format!("{}/address/{}", self.api_url(), self.address);
        let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
        
        let funded = resp["chain_stats"]["funded_txo_sum"].as_u64().unwrap_or(0);
        let spent = resp["chain_stats"]["spent_txo_sum"].as_u64().unwrap_or(0);
        Ok(funded.saturating_sub(spent))
    }

    /// Get UTXOs
    pub async fn get_utxos(&self) -> Result<Vec<Utxo>> {
        let url = format!("{}/address/{}/utxo", self.api_url(), self.address);
        let utxos: Vec<Utxo> = reqwest::get(&url).await?.json().await?;
        Ok(utxos)
    }

    /// Send Bitcoin
    pub async fn send(&self, to_address: &str, amount_sats: u64) -> Result<String> {
        let utxos = self.get_utxos().await?;
        if utxos.is_empty() {
            return Err(anyhow::anyhow!("No UTXOs available"));
        }

        let to_addr = to_address.parse::<Address<_>>()?.require_network(self.network)?;
        let from_addr: Address = self.address.parse::<Address<_>>()?.require_network(self.network)?;
        
        let fee = 10000u64;
        let mut total_in = 0u64;
        let mut inputs = vec![];
        let mut selected_utxos = vec![];

        for utxo in utxos {
            if total_in >= amount_sats + fee { break; }
            
            inputs.push(TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str(&utxo.txid)?,
                    vout: utxo.vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::default(),
            });
            selected_utxos.push(utxo.value);
            total_in += utxo.value;
        }

        if total_in < amount_sats + fee {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }

        let mut outputs = vec![TxOut {
            value: Amount::from_sat(amount_sats),
            script_pubkey: to_addr.script_pubkey(),
        }];

        let change = total_in - amount_sats - fee;
        if change > 546 {
            outputs.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: from_addr.script_pubkey(),
            });
        }

        let mut tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: inputs,
            output: outputs,
        };

        // Sign
        let mut witnesses = vec![];
        {
            let mut cache = SighashCache::new(&tx);
            for (i, &value) in selected_utxos.iter().enumerate() {
                let sighash = cache.p2wpkh_signature_hash(
                    i,
                    &from_addr.script_pubkey(),
                    Amount::from_sat(value),
                    EcdsaSighashType::All,
                )?;
                let msg = Message::from_digest_slice(&sighash[..])?;
                let sig = self.secp.sign_ecdsa(&msg, &self.private_key.inner);
                
                let mut witness = Witness::new();
                witness.push_ecdsa_signature(&bitcoin::ecdsa::Signature {
                    sig,
                    hash_ty: EcdsaSighashType::All,
                });
                witness.push(self.public_key.to_bytes());
                witnesses.push(witness);
            }
        }

        for (i, w) in witnesses.into_iter().enumerate() {
            tx.input[i].witness = w;
        }

        // Broadcast
        let tx_hex = hex::encode(serialize(&tx));
        let url = format!("{}/tx", self.api_url());
        let resp = reqwest::Client::new().post(&url).body(tx_hex).send().await?;
        
        if resp.status().is_success() {
            Ok(resp.text().await?)
        } else {
            Err(anyhow::anyhow!("Broadcast failed: {}", resp.text().await?))
        }
    }

    pub fn get_wif(&self) -> String {
        self.private_key.to_wif()
    }

    pub fn explorer_url(&self) -> String {
        match self.network {
            Network::Bitcoin => format!("https://mempool.space/address/{}", self.address),
            _ => format!("https://mempool.space/testnet/address/{}", self.address),
        }
    }
}
