//! Bitcoin P2SH and P2WSH multi-signature support

use crate::{error::*, MultisigConfig, PendingTransaction};
use bitcoin::{script::Builder, opcodes, PublicKey, ScriptBuf, Address, Network};

/// Derive P2WSH multisig address
pub fn derive_address(config: &MultisigConfig) -> MultisigResult<String> {
    let redeem_script = create_redeem_script(config)?;
    let script_hash = bitcoin::WScriptHash::hash(redeem_script.as_bytes());
    let address = Address::p2wsh(&ScriptBuf::new_p2wsh(&script_hash), Network::Bitcoin);
    Ok(address.to_string())
}

/// Create M-of-N redeem script
pub fn create_redeem_script(config: &MultisigConfig) -> MultisigResult<ScriptBuf> {
    let mut builder = Builder::new()
        .push_int(config.threshold as i64);
    
    for signer in &config.signers {
        let pubkey = PublicKey::from_slice(&signer.public_key)
            .map_err(|e| MultisigError::ScriptError(e.to_string()))?;
        builder = builder.push_key(&pubkey);
    }
    
    let script = builder
        .push_int(config.total_signers as i64)
        .push_opcode(opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    
    Ok(script)
}

/// Finalize Bitcoin multisig transaction
pub fn finalize(pending: &PendingTransaction, config: &MultisigConfig) -> MultisigResult<Vec<u8>> {
    let mut witness_stack = Vec::new();
    witness_stack.push(vec![]); // OP_0 for CHECKMULTISIG bug
    
    for sig in &pending.signatures {
        witness_stack.push(sig.signature.clone());
    }
    
    let redeem_script = create_redeem_script(config)?;
    witness_stack.push(redeem_script.to_bytes());
    
    // Combine with original tx
    let mut final_tx = pending.tx_data.clone();
    // In real impl, properly encode witness data
    for item in witness_stack {
        final_tx.extend_from_slice(&item);
    }
    
    Ok(final_tx)
}
