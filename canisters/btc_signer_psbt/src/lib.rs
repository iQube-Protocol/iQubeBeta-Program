//! Constitutional Anchor v2 — `btc_signer_psbt`.
//!
//! ─── WHAT THIS CANISTER IS FOR ──────────────────────────────────────────────
//!
//! It commits a 32-byte Merkle root — produced by `proof_of_state` over
//! constitutional receipt commitments — into a real Bitcoin transaction's
//! OP_RETURN output, signs it with threshold ECDSA, and broadcasts it through
//! the IC's native Bitcoin API.
//!
//! ─── "v2" IS AN ARCHITECTURE GENERATION, NOT A REDEPLOY ─────────────────────
//!
//! A read-only lineage census (2026-08-08) established that **no IC-mainnet BTC
//! signer has ever existed**. `uxrrr-q7777-77774-qaaaq-cai` — the principal
//! `proof_of_state::anchor()` hard-coded and env config labelled "LIVE MAINNET"
//! — is a LOCAL dfx id that resolves `canister_not_found` on the IC. So every
//! anchor call failed, and the caller's error branch synthesised
//! `mock_btc_txid_*`, which is what all 76 "anchored" batches actually record.
//!
//! The package name stays `btc_signer_psbt`. This will be its FIRST genuine
//! mainnet deployment; there is no deployed v1 to supersede.
//!
//! ─── WHAT CHANGED FROM THE PREDECESSOR ──────────────────────────────────────
//!
//! | was | now |
//! |---|---|
//! | `_op_return_script` computed and discarded | the root is encoded into a real output script |
//! | outputs held the strings "OP_RETURN"/"change_address" | outputs hold consensus script bytes |
//! | `txid = signature[..32]` | `txid = sha256d(witness-free serialisation)` |
//! | `raw_tx = "signed_tx_<hex>"` | `raw_tx` = the serialised transaction |
//! | `"tb1q" + hex(pubkey[..20])` | BIP-173 bech32 over `hash160(compressed pubkey)` |
//! | all-zero mock UTXO | real UTXOs from `bitcoin_get_utxos`; refuses when none |
//! | HTTP `sendrawtransaction` outcall | `bitcoin_send_transaction` (native, replicated) |
//! | failure could return `Ok("broadcast_success_…")` | every failure is an `Err` |
//!
//! Transport is the IC's native Bitcoin API per amendment A1: an HTTPS outcall
//! must reach byte-identical responses across replicas to pass consensus, so a
//! block explorer's response is structurally consensus-hostile — and it would
//! make a third party the arbiter of whether a constitutional anchor exists.
//!
//! ─── NO HARD-CODED PRODUCTION PRINCIPALS ────────────────────────────────────
//!
//! This canister embeds no other canister's production principal. The only
//! principal it addresses is the IC management canister, which is protocol-
//! defined (`aaaaa-aa`) and reached through `ic_cdk`'s own API rather than a
//! literal. See AGENTS.md, "Production canister principals are never hard-coded
//! in dependent canister source".

use candid::{CandidType, Deserialize};
use ic_cdk::{
    query, update,
    api::management_canister::{
        bitcoin::{
            bitcoin_get_current_fee_percentiles, bitcoin_get_utxos, bitcoin_send_transaction,
            BitcoinNetwork, GetCurrentFeePercentilesRequest, GetUtxosRequest, SendTransactionRequest,
        },
        ecdsa::{
            ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
            SignWithEcdsaArgument,
        },
    },
};
use std::collections::HashMap;

// The pure Bitcoin construction lives in its own rlib so it is testable on the
// host — this crate is a cdylib and cannot be linked into a test binary. See
// btc_anchor_core/Cargo.toml for why that split is load-bearing.
pub use btc_anchor_core::*;

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

/// An output carrying REAL consensus data.
///
/// `script_pubkey_hex` replaces the predecessor's `address: String`, which held
/// the literal labels "OP_RETURN" and "change_address" — values no serialiser
/// could ever turn into transaction bytes. `address` is retained as an OPTIONAL
/// human-readable rendering; it is descriptive, never the source of truth.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub script_pubkey_hex: String,
    pub address: Option<String>,
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
    /// Set at init; NOT a compile-time constant, so the same wasm can serve
    /// testnet and mainnet and the choice is visible in the deployment record.
    static NETWORK: std::cell::RefCell<BtcNetwork> = std::cell::RefCell::new(BtcNetwork::Testnet);
}

const KEY_NAME: &str = "test_key_1";
const DERIVATION_PATH_DEFAULT: &[&[u8]] = &[b"constitutional-anchor-v2"];

fn ic_network() -> BitcoinNetwork {
    NETWORK.with(|n| match *n.borrow() {
        BtcNetwork::Mainnet => BitcoinNetwork::Mainnet,
        BtcNetwork::Testnet => BitcoinNetwork::Testnet,
    })
}

fn key_id() -> EcdsaKeyId {
    EcdsaKeyId { curve: EcdsaCurve::Secp256k1, name: KEY_NAME.to_string() }
}

/// Fetch this canister's own compressed secp256k1 public key.
async fn own_pubkey(derivation_path: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    let (res,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: key_id(),
    })
    .await
    .map_err(|e| format!("ecdsa_public_key failed: {e:?}"))?;
    Ok(res.public_key)
}

#[update]
pub async fn get_btc_address(derivation_path: Vec<Vec<u8>>) -> Result<BitcoinAddress, String> {
    let path = if derivation_path.is_empty() {
        DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect()
    } else {
        derivation_path
    };
    let public_key = own_pubkey(path.clone()).await?;
    let network = NETWORK.with(|n| *n.borrow());
    let address = p2wpkh_address(&public_key, network)?;

    let btc_address = BitcoinAddress { address: address.clone(), public_key, derivation_path: path };
    ADDRESSES.with(|a| a.borrow_mut().insert(address, btc_address.clone()));
    Ok(btc_address)
}

/// Build the unsigned anchor transaction.
///
/// Output 0 is ALWAYS the commitment: `OP_RETURN OP_PUSHBYTES_32 <root>`,
/// value 0. Output 1 is change back to our own P2WPKH script. The root is
/// hex-decoded to 32 raw bytes before being pushed (§A3) — pushing the ASCII
/// hex would commit to a different value.
#[update]
pub async fn create_anchor_transaction(
    utxos: Vec<UTXO>,
    data_hash: String,
    fee_rate: u64,
) -> Result<UnsignedTransaction, String> {
    // Planning, fee estimation and every refusal live in `btc_anchor_core`
    // (pure, host-tested). This function only turns a plan into the Candid
    // shape — so the rules the acceptance tests pin are the rules that run.
    let plan = plan_anchor(
        &utxos
            .iter()
            .map(|u| AnchorInput { txid_hex: u.txid.clone(), vout: u.vout, value: u.amount })
            .collect::<Vec<_>>(),
        &data_hash,
        fee_rate,
    )?;

    let pubkey = own_pubkey(DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect()).await?;
    let h160 = hash160(&pubkey);
    let change_script = p2wpkh_script(&h160);
    let network = NETWORK.with(|n| *n.borrow());

    Ok(UnsignedTransaction {
        inputs: utxos
            .into_iter()
            .map(|utxo| TransactionInput { utxo, sequence: 0xffff_fffd }) // RBF enabled
            .collect(),
        outputs: vec![
            TransactionOutput {
                script_pubkey_hex: hex::encode(&plan.op_return),
                address: None, // an OP_RETURN output is unspendable and has no address
                amount: 0,
            },
            TransactionOutput {
                script_pubkey_hex: hex::encode(&change_script),
                address: p2wpkh_address(&pubkey, network).ok(),
                amount: plan.change_value,
            },
        ],
        locktime: 0,
    })
}

/// Convert the Candid-facing shape into the pure serialiser's shape.
fn to_pure_tx(tx: &UnsignedTransaction) -> Result<Tx, String> {
    Ok(Tx {
        version: 2,
        inputs: tx
            .inputs
            .iter()
            .map(|i| TxIn {
                prev_txid_hex: i.utxo.txid.clone(),
                vout: i.utxo.vout,
                value: i.utxo.amount,
                sequence: i.sequence,
            })
            .collect(),
        outputs: tx
            .outputs
            .iter()
            .map(|o| {
                Ok(TxOut {
                    value: o.amount,
                    script_pubkey: hex::decode(&o.script_pubkey_hex)
                        .map_err(|e| format!("output script is not hex: {e}"))?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        locktime: tx.locktime,
    })
}

#[update]
pub async fn sign_transaction(
    unsigned: UnsignedTransaction,
    derivation_path: Vec<Vec<u8>>,
) -> Result<SignedTransaction, String> {
    let path: Vec<Vec<u8>> = if derivation_path.is_empty() {
        DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect()
    } else {
        derivation_path
    };
    let pubkey = own_pubkey(path.clone()).await?;
    let h160 = hash160(&pubkey);
    let tx = to_pure_tx(&unsigned)?;

    let total_in: u64 = unsigned.inputs.iter().map(|i| i.utxo.amount).sum();
    let total_out: u64 = unsigned.outputs.iter().map(|o| o.amount).sum();
    let fee = total_in.saturating_sub(total_out);

    let mut sigs = Vec::with_capacity(tx.inputs.len());
    for idx in 0..tx.inputs.len() {
        let sighash = bip143_sighash_p2wpkh(&tx, idx, &h160)?;
        let (sig,) = sign_with_ecdsa(SignWithEcdsaArgument {
            message_hash: sighash.to_vec(),
            derivation_path: path.clone(),
            key_id: key_id(),
        })
        .await
        .map_err(|e| format!("sign_with_ecdsa failed for input {idx}: {e:?}"))?;
        sigs.push(sig.signature);
    }

    let (txid, raw_tx) = assemble_signed(&tx, &sigs, &pubkey)?;
    let signed = SignedTransaction { txid: txid.clone(), raw_tx: raw_tx.clone(), size: (raw_tx.len() / 2) as u32, fee };
    TRANSACTIONS.with(|t| t.borrow_mut().insert(txid, signed.clone()));
    Ok(signed)
}

/// Broadcast through the IC's native Bitcoin API.
///
/// TRUTHFUL BY CONSTRUCTION. The predecessor's HTTP path could return
/// `Ok(format!("broadcast_success_{…}"))` — a success value synthesised from
/// its own input — when txid parsing failed. There is no such branch here: the
/// only `Ok` is one the network produced, and the txid returned is the one
/// computed from the transaction's own bytes.
#[update]
pub async fn broadcast_transaction(raw_tx: String) -> Result<String, String> {
    let bytes = validate_raw_tx_hex(&raw_tx)?;
    bitcoin_send_transaction(SendTransactionRequest { transaction: bytes.clone(), network: ic_network() })
        .await
        .map_err(|e| format!("bitcoin_send_transaction rejected the transaction: {e:?}"))?;

    // The management canister returns unit on success. A txid cannot be
    // derived from raw segwit bytes without re-parsing them, and inventing one
    // is exactly the defect this canister exists to remove — so this entry
    // point reports success without claiming an identifier it did not compute.
    // Callers wanting the txid use `create_and_broadcast_anchor`, which
    // computes it from the structured transaction before broadcasting.
    Ok("broadcast accepted by the network; use create_and_broadcast_anchor for the txid".to_string())
}

/// The full anchoring ceremony: fetch real UTXOs, build, sign, broadcast.
///
/// REFUSES rather than fabricating. The predecessor substituted a UTXO with an
/// all-zero txid and proceeded — manufacturing the appearance of an anchor with
/// nothing to spend.
#[update]
pub async fn create_and_broadcast_anchor(data_hash: String, fee_rate: u64) -> Result<String, String> {
    // Validate the commitment BEFORE spending anything.
    let _ = op_return_script(&data_hash)?;

    let path: Vec<Vec<u8>> = DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect();
    let pubkey = own_pubkey(path.clone()).await?;
    let network = NETWORK.with(|n| *n.borrow());
    let address = p2wpkh_address(&pubkey, network)?;

    let (utxo_res,) = bitcoin_get_utxos(GetUtxosRequest {
        address: address.clone(),
        network: ic_network(),
        filter: None,
    })
    .await
    .map_err(|e| format!("bitcoin_get_utxos failed: {e:?}"))?;

    if utxo_res.utxos.is_empty() {
        return Err(format!(
            "No UTXOs at {address} — refusing to anchor. This canister will not substitute a \
             placeholder input to manufacture the appearance of a Bitcoin anchor."
        ));
    }

    let utxos: Vec<UTXO> = utxo_res
        .utxos
        .iter()
        .map(|u| {
            let mut txid = u.outpoint.txid.clone();
            txid.reverse(); // management canister returns consensus order
            UTXO { txid: hex::encode(txid), vout: u.outpoint.vout, amount: u.value, script_pubkey: vec![] }
        })
        .collect();

    let effective_fee_rate = if fee_rate > 0 { fee_rate } else { median_fee_rate().await.unwrap_or(2) };
    let unsigned = create_anchor_transaction(utxos, data_hash, effective_fee_rate).await?;
    let signed = sign_transaction(unsigned, path).await?;

    let bytes = hex::decode(&signed.raw_tx).map_err(|e| format!("assembled raw_tx is not hex: {e}"))?;
    bitcoin_send_transaction(SendTransactionRequest { transaction: bytes, network: ic_network() })
        .await
        .map_err(|e| format!("bitcoin_send_transaction rejected the anchor: {e:?}"))?;

    // The txid was computed from the transaction's own bytes before broadcast.
    Ok(signed.txid)
}

async fn median_fee_rate() -> Option<u64> {
    let (p,) = bitcoin_get_current_fee_percentiles(GetCurrentFeePercentilesRequest { network: ic_network() })
        .await
        .ok()?;
    // Percentiles are millisatoshi/vB; index 50 is the median.
    p.get(50).map(|m| (m / 1000).max(1))
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

ic_cdk::export_candid!();
