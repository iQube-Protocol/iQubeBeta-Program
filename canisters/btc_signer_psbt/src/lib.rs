use candid::{CandidType, Deserialize};
use ic_cdk::{query, update, api::management_canister::ecdsa::{ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument}};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone)]
pub struct BitcoinAddress {
    pub address: String,
    pub public_key: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UTXO {
    pub txid: String,
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub utxo: UTXO,
    pub sequence: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub address: String,
    pub amount: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct UnsignedTransaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub locktime: u32,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct SignedTransaction {
    pub txid: String,
    pub raw_tx: String,
    pub size: u32,
    pub fee: u64,
}

thread_local! {
    static ADDRESSES: std::cell::RefCell<HashMap<String, BitcoinAddress>> = std::cell::RefCell::new(HashMap::new());
    static TRANSACTIONS: std::cell::RefCell<HashMap<String, SignedTransaction>> = std::cell::RefCell::new(HashMap::new());
}

const KEY_NAME: &str = "test_key_1";

#[update]
pub async fn get_btc_address(derivation_path: Vec<Vec<u8>>) -> Result<BitcoinAddress, String> {
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: KEY_NAME.to_string(),
    };

    let public_key_arg = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    match ecdsa_public_key(public_key_arg).await {
        Ok((public_key_response,)) => {
            let public_key = public_key_response.public_key;
            
            // Generate Bitcoin testnet address (simplified - should use proper Bitcoin address encoding)
            let address = format!("tb1q{}", hex::encode(&public_key[..20]));
            
            let btc_address = BitcoinAddress {
                address: address.clone(),
                public_key: public_key.clone(),
                derivation_path,
            };

            ADDRESSES.with(|a| a.borrow_mut().insert(address.clone(), btc_address.clone()));
            
            Ok(btc_address)
        }
        Err(e) => Err(format!("Failed to get public key: {:?}", e)),
    }
}

#[update]
pub async fn create_anchor_transaction(
    data_hash: String,
    utxos: Vec<UTXO>,
    fee_rate: u64,
) -> Result<UnsignedTransaction, String> {
    if utxos.is_empty() {
        return Err("No UTXOs provided".to_string());
    }

    let total_input: u64 = utxos.iter().map(|u| u.amount).sum();
    let estimated_fee = fee_rate * 250; // Rough estimate for anchor tx size
    
    if total_input <= estimated_fee {
        return Err("Insufficient funds for transaction".to_string());
    }

    let change_amount = total_input - estimated_fee;
    
    // Create OP_RETURN output with data hash
    let _op_return_script = format!("6a20{}", data_hash); // OP_RETURN + 32 bytes
    
    let inputs: Vec<TransactionInput> = utxos.into_iter().map(|utxo| TransactionInput {
        utxo,
        sequence: 0xfffffffd, // Enable RBF
    }).collect();

    // For now, create a simplified transaction structure
    // In production, this would need proper Bitcoin transaction encoding
    let unsigned_tx = UnsignedTransaction {
        inputs,
        outputs: vec![
            TransactionOutput {
                address: "OP_RETURN".to_string(),
                amount: 0,
            },
            TransactionOutput {
                address: "change_address".to_string(),
                amount: change_amount,
            }
        ],
        locktime: 0,
    };

    Ok(unsigned_tx)
}

#[update]
pub async fn sign_transaction(
    unsigned_tx: UnsignedTransaction,
    derivation_path: Vec<Vec<u8>>,
) -> Result<SignedTransaction, String> {
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: KEY_NAME.to_string(),
    };

    // Create transaction hash for signing (simplified)
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}", unsigned_tx).as_bytes());
    let tx_hash = hasher.finalize().to_vec();

    let sign_arg = SignWithEcdsaArgument {
        message_hash: tx_hash,
        derivation_path,
        key_id,
    };

    match sign_with_ecdsa(sign_arg).await {
        Ok((signature_response,)) => {
            let signature = signature_response.signature;
            
            // In production, this would properly encode the Bitcoin transaction
            let txid = hex::encode(&signature[..32]);
            let raw_tx = format!("signed_tx_{}", txid);
            
            let signed_tx = SignedTransaction {
                txid: txid.clone(),
                raw_tx,
                size: 250,
                fee: 1000,
            };

            TRANSACTIONS.with(|t| t.borrow_mut().insert(txid.clone(), signed_tx.clone()));
            
            Ok(signed_tx)
        }
        Err(e) => Err(format!("Failed to sign transaction: {:?}", e)),
    }
}

#[update]
pub async fn broadcast_transaction(raw_tx: String) -> Result<String, String> {
    // In production, this would broadcast to Bitcoin network via HTTP outcalls
    // For now, return mock txid
    let mut hasher = Sha256::new();
    hasher.update(raw_tx.as_bytes());
    let txid = hex::encode(hasher.finalize());
    
    Ok(txid)
}

#[query]
pub fn get_transaction(txid: String) -> Option<SignedTransaction> {
    TRANSACTIONS.with(|t| t.borrow().get(&txid).cloned())
}

#[query]
pub fn get_address_info(address: String) -> Option<BitcoinAddress> {
    ADDRESSES.with(|a| a.borrow().get(&address).cloned())
}

#[query]
pub fn get_all_addresses() -> Vec<BitcoinAddress> {
    ADDRESSES.with(|a| a.borrow().values().cloned().collect())
}

// Export Candid interface
ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_tx_requires_utxos() {
        let res = futures::executor::block_on(create_anchor_transaction(
            "abcd".to_string(),
            vec![],
            10,
        ));
        assert!(res.is_err());
    }

    #[test]
    fn anchor_tx_builds_with_change() {
        let utxo = UTXO {
            txid: "txid".to_string(),
            vout: 0,
            amount: 100_000,
            script_pubkey: vec![],
        };
        let res = futures::executor::block_on(create_anchor_transaction(
            "deadbeef".to_string(),
            vec![utxo],
            10,
        ));
        let tx = res.expect("should build unsigned tx");
        // Two outputs: OP_RETURN and change
        assert_eq!(tx.outputs.len(), 2);
        // Change output should be > 0 with our fake fee calc
        let change = tx.outputs.iter().find(|o| o.address == "change_address").unwrap();
        assert!(change.amount > 0);
    }
}
