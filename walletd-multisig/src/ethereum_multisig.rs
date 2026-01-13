//! Ethereum Gnosis Safe style multi-signature support

use crate::{error::*, MultisigConfig, PendingTransaction};
use sha2::{Sha256, Digest};

/// Derive CREATE2 Safe address
pub fn derive_address(config: &MultisigConfig) -> MultisigResult<String> {
    let mut hasher = Sha256::new();
    hasher.update([config.threshold, config.total_signers]);
    for signer in &config.signers {
        hasher.update(&signer.public_key);
    }
    let hash = hasher.finalize();
    Ok(format!("0x{}", hex::encode(&hash[12..32])))
}

/// Finalize Ethereum multisig transaction
pub fn finalize(pending: &PendingTransaction, _config: &MultisigConfig) -> MultisigResult<Vec<u8>> {
    let mut final_tx = pending.tx_data.clone();
    
    // Sort signatures by signer address (Gnosis Safe requirement)
    let mut sorted_sigs: Vec<_> = pending.signatures.iter().collect();
    sorted_sigs.sort_by(|a, b| a.signer_id.cmp(&b.signer_id));
    
    // Concatenate signatures (r, s, v format)
    for sig in sorted_sigs {
        final_tx.extend_from_slice(&sig.signature);
    }
    
    Ok(final_tx)
}
