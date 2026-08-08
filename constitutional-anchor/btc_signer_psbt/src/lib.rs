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

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
// CURRENT, NON-DEPRECATED TRANSPORT (operator amendment A1 + transport ruling).
// `ic_cdk::api::management_canister::bitcoin` is the deprecated facade; these
// are the maintained crates. Isolating the signer in its own workspace is what
// made this reachable without uplifting the frozen proof_of_state canister.
use ic_cdk_bitcoin_canister::{
    bitcoin_get_current_fee_percentiles, bitcoin_get_utxos, bitcoin_send_transaction,
    GetCurrentFeePercentilesRequest, GetUtxosRequest, NetworkInRequest as BitcoinNetwork, SendTransactionRequest,
};
use ic_cdk_management_canister::{ecdsa_public_key, sign_with_ecdsa};
use ic_management_canister_types::{EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs, SignWithEcdsaArgs};
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

/// Deployment-time configuration. Every field is REQUIRED — there are no
/// defaults, because the previous build's implicit `Testnet` + `test_key_1`
/// would have signed mainnet value with a key the subnet can rotate.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    /// "mainnet" | "testnet"
    pub network: String,
    /// "key_1" (production) | "test_key_1" | "dfx_test_key"
    pub ecdsa_key_name: String,
    /// The proof_of_state canister permitted to request anchoring.
    pub authorized_pos_principal: Principal,
}

thread_local! {
    static ADDRESSES: std::cell::RefCell<HashMap<String, BitcoinAddress>> = std::cell::RefCell::new(HashMap::new());
    static TRANSACTIONS: std::cell::RefCell<HashMap<String, SignedTransaction>> = std::cell::RefCell::new(HashMap::new());
    /// NOT initialised to a usable default. Until `init` runs, every anchoring
    /// request is denied — an uninitialised signer must be inert, not permissive.
    static CONFIG: std::cell::RefCell<Option<AnchorConfig>> = std::cell::RefCell::new(None);
    /// P0.3 (operator ruling, 2026-08-08) — the durable, root-indexed anchor-
    /// attempt state machine. `BTreeMap`, not `HashMap`: `decide_anchor_attempt`
    /// takes `&BTreeMap` so it stays pure and host-testable without pulling
    /// ic-cdk's hasher into btc_anchor_core, and iteration order is
    /// deterministic, which matters for the cross-root exclusivity scan.
    ///
    /// Persisted across upgrades in `pre_upgrade`/`post_upgrade` below — this
    /// is the "evidence needed for recovery" the ruling requires to survive
    /// one, not merely heap state that an upgrade would silently discard.
    static ANCHOR_ATTEMPTS: std::cell::RefCell<std::collections::BTreeMap<String, AnchorAttemptState>> =
        std::cell::RefCell::new(std::collections::BTreeMap::new());
}

const DERIVATION_PATH_DEFAULT: &[&[u8]] = &[b"constitutional-anchor-v2"];

fn parse_network(s: &str) -> Result<BtcNetwork, String> {
    match s.to_ascii_lowercase().as_str() {
        "mainnet" => Ok(BtcNetwork::Mainnet),
        "testnet" => Ok(BtcNetwork::Testnet),
        other => Err(format!("unknown network {other:?}; expected \"mainnet\" or \"testnet\"")),
    }
}

fn config() -> Result<AnchorConfig, String> {
    CONFIG.with(|c| {
        c.borrow().clone().ok_or_else(|| {
            "canister is not configured — it was deployed without init arguments and denies all \
             anchoring requests"
                .to_string()
        })
    })
}

fn apply_config(arg: InitArg) {
    let cfg = AnchorConfig {
        network: parse_network(&arg.network).unwrap_or_else(|e| ic_cdk::trap(&e)),
        ecdsa_key_name: arg.ecdsa_key_name,
        authorized_pos_principal: Some(arg.authorized_pos_principal.to_text()),
    };
    // TRAP rather than start misconfigured. A signer that comes up with a
    // rejected configuration is worse than one that fails to come up at all.
    if let Err(e) = validate_anchor_config(&cfg) {
        ic_cdk::trap(&format!("refusing to initialise: {e}"));
    }
    CONFIG.with(|c| *c.borrow_mut() = Some(cfg));
}

#[init]
fn init(arg: InitArg) {
    apply_config(arg);
}

#[pre_upgrade]
fn pre_upgrade() {
    let cfg = CONFIG.with(|c| c.borrow().clone());
    // P0.3: the anchor-attempt map persists alongside config, as a plain
    // Vec of pairs — the same explicit-shape discipline the InitArg
    // conversion above already follows, rather than trusting an unchecked
    // direct BTreeMap serialisation.
    let attempts: Vec<(String, AnchorAttemptState)> =
        ANCHOR_ATTEMPTS.with(|a| a.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    ic_cdk::storage::stable_save((
        cfg.map(|c| InitArg {
            network: match c.network { BtcNetwork::Mainnet => "mainnet".into(), BtcNetwork::Testnet => "testnet".into() },
            ecdsa_key_name: c.ecdsa_key_name,
            authorized_pos_principal: Principal::from_text(
                c.authorized_pos_principal.unwrap_or_default(),
            )
            .unwrap_or(Principal::anonymous()),
        }),
        attempts,
    ))
    .expect("config and anchor-attempt state must survive upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    // No prior deployment of this canister has ever existed (2026-08-08
    // lineage census) — there is no pre-P0.3 stable-memory layout to remain
    // compatible with, so the tuple shape below is free to include the
    // attempt map from this canister's first real upgrade onward.
    if let Ok((cfg_arg, attempts)) =
        ic_cdk::storage::stable_restore::<(Option<InitArg>, Vec<(String, AnchorAttemptState)>)>()
    {
        if let Some(arg) = cfg_arg {
            apply_config(arg);
        }
        ANCHOR_ATTEMPTS.with(|a| *a.borrow_mut() = attempts.into_iter().collect());
    }
}

/// Read the configuration. No secrets — the principal and key NAME are public
/// facts about the deployment, and publishing them is what makes the
/// authorization boundary auditable from outside.
#[query]
pub fn get_config() -> Option<(String, String, String)> {
    CONFIG.with(|c| {
        c.borrow().as_ref().map(|cfg| {
            (
                match cfg.network { BtcNetwork::Mainnet => "mainnet".to_string(), BtcNetwork::Testnet => "testnet".to_string() },
                cfg.ecdsa_key_name.clone(),
                cfg.authorized_pos_principal.clone().unwrap_or_else(|| "(unset)".to_string()),
            )
        })
    })
}

fn ic_network() -> Result<BitcoinNetwork, String> {
    Ok(match config()?.network {
        BtcNetwork::Mainnet => BitcoinNetwork::Mainnet,
        BtcNetwork::Testnet => BitcoinNetwork::Testnet,
    })
}

fn key_id() -> Result<EcdsaKeyId, String> {
    Ok(EcdsaKeyId { curve: EcdsaCurve::Secp256k1, name: config()?.ecdsa_key_name })
}

/// Fetch this canister's own compressed secp256k1 public key.
async fn own_pubkey(derivation_path: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    let res = ecdsa_public_key(&EcdsaPublicKeyArgs {
        canister_id: None,
        derivation_path,
        key_id: key_id()?,
    })
    .await
    .map_err(|e| format!("ecdsa_public_key failed: {e:?}"))?;
    Ok(res.public_key)
}

async fn get_btc_address(derivation_path: Vec<Vec<u8>>) -> Result<BitcoinAddress, String> {
    let path = if derivation_path.is_empty() {
        DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect()
    } else {
        derivation_path
    };
    let public_key = own_pubkey(path.clone()).await?;
    let network = config()?.network;
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
async fn create_anchor_transaction(
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
    let network = config()?.network;

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

async fn sign_transaction(
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
        let sig = sign_with_ecdsa(&SignWithEcdsaArgs {
            message_hash: sighash.to_vec(),
            derivation_path: path.clone(),
            key_id: key_id()?,
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
async fn broadcast_transaction(raw_tx: String) -> Result<String, String> {
    let bytes = validate_raw_tx_hex(&raw_tx)?;
    bitcoin_send_transaction(&SendTransactionRequest { transaction: bytes.clone(), network: ic_network()? })
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

/// Fetch real UTXOs, build, and sign — the part of the ceremony that happens
/// BEFORE a decision has been persisted about the outcome. Returns the signed
/// transaction's parts; the caller is responsible for persisting `Signed`
/// (before broadcasting) and `Broadcast`/`Failed` afterward — this function
/// touches no anchor-attempt state itself, so the SAME persistence rule
/// applies regardless of which caller invokes it.
///
/// REFUSES rather than fabricating. The predecessor substituted a UTXO with an
/// all-zero txid and proceeded — manufacturing the appearance of an anchor
/// with nothing to spend.
async fn run_fresh_anchor_ceremony(
    root: String,
    path: Vec<Vec<u8>>,
    fee_rate: u64,
) -> Result<(String, String, Vec<AnchorInput>), String> {
    let pubkey = own_pubkey(path.clone()).await?;
    let network = config()?.network;
    let address = p2wpkh_address(&pubkey, network)?;

    let utxo_res = bitcoin_get_utxos(&GetUtxosRequest {
        address: address.clone(),
        network: ic_network()?,
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
            // `Txid` is a typed newtype over 32 bytes in CONSENSUS order;
            // Bitcoin displays txids reversed, and `prev_txid_le` in
            // btc_anchor_core reverses again on serialisation. Getting this
            // wrong would spend a different outpoint than intended.
            let mut txid = u.outpoint.txid.as_ref().to_vec();
            txid.reverse();
            UTXO { txid: hex::encode(txid), vout: u.outpoint.vout, amount: u.value, script_pubkey: vec![] }
        })
        .collect();
    let anchor_inputs: Vec<AnchorInput> = utxos
        .iter()
        .map(|u| AnchorInput { txid_hex: u.txid.clone(), vout: u.vout, value: u.amount })
        .collect();

    let effective_fee_rate = if fee_rate > 0 {
        fee_rate
    } else {
        median_fee_rate().await.ok_or_else(|| {
            "FEE_RATE_UNAVAILABLE: no fee_rate was supplied and Bitcoin fee-percentile discovery \
             failed — refusing to substitute a guessed rate"
                .to_string()
        })?
    };

    let unsigned = create_anchor_transaction(utxos, root, effective_fee_rate).await?;
    let signed = sign_transaction(unsigned, path).await?;
    Ok((signed.txid, signed.raw_tx, anchor_inputs))
}

/// THE ONLY AUTHORIZED ENTRY POINT (P0.1, independent review 2026-08-08).
///
/// `create_anchor_transaction`, `sign_transaction` and `broadcast_transaction`
/// are private. They were public `#[update]` methods, which meant ANY principal
/// on the IC could make this canister sign with its threshold key, spend its
/// UTXOs, or broadcast arbitrary bytes. A signer with an open signing surface
/// is not a signer; it is a signing oracle for whoever asks.
///
/// ── THE CALLER IS CAPTURED BEFORE THE FIRST AWAIT ──────────────────────────
///
/// This is not stylistic. On the IC, `ic_cdk::caller()` returns whoever is
/// replying AT THAT POINT IN EXECUTION. After an inter-canister `await` — and
/// this function awaits `ecdsa_public_key`, `bitcoin_get_utxos`,
/// `sign_with_ecdsa` and `bitcoin_send_transaction` — it returns the MANAGEMENT
/// CANISTER, not the originator. An authorization check placed after any await
/// would therefore compare the management canister against the configured
/// proof_of_state principal, fail, and be "fixed" by whoever debugged it into
/// something that passes for everyone. Capturing first makes that mistake
/// impossible to make quietly.
///
/// ── SPEND SERIALISATION / IDEMPOTENCY (P0.3, operator ruling, 2026-08-08) ──
///
/// Immediately after authorization and BEFORE the first NETWORK await, this
/// atomically decides and — if the decision is `Reserve` — records the
/// reservation, in one synchronous `RefCell` borrow. IC message execution is
/// atomic up to its first await, so no concurrent call can observe or act on
/// this root between the decision and the insert: there is no window for a
/// second caller to slip in.
///
/// `decide_anchor_attempt` (btc_anchor_core, pure, host-tested) owns every
/// rule about what happens next: return the existing txid for an
/// already-broadcast root, rebroadcast the exact existing transaction for an
/// already-signed root without rebuilding it, refuse a different root while
/// one is active, or clear to reserve. This function only APPLIES the
/// decision — the rule that is tested is the rule that runs.
///
/// `Signed` is persisted BEFORE `bitcoin_send_transaction` is invoked for a
/// fresh ceremony, so a canister trap, upgrade, or ambiguous resumption
/// between signing and broadcast leaves durable evidence of exactly which
/// transaction to rebroadcast — never a reason to sign a second one.
#[update]
pub async fn create_and_broadcast_anchor(data_hash: String, fee_rate: u64) -> Result<String, String> {
    // ── FIRST STATEMENT. NOTHING MAY PRECEDE THIS. ──
    let caller = ic_cdk::api::msg_caller().to_text();
    let cfg = config()?;
    authorize_anchor_caller(&caller, &cfg)?;

    // Validate the commitment AND canonicalise it — the attempt map is keyed
    // on this canonical form so two spellings of the same root are never
    // treated as two different anchor attempts.
    let root = normalize_root_hex(&data_hash)?;

    // ── ATOMIC DECISION + RESERVATION. STILL BEFORE THE FIRST AWAIT. ──
    let decision = ANCHOR_ATTEMPTS.with(|a| {
        let mut attempts = a.borrow_mut();
        let decision = decide_anchor_attempt(&root, &attempts);
        if matches!(decision, AnchorDecision::Reserve) {
            attempts.insert(root.clone(), AnchorAttemptState::Reserved);
        }
        decision
    });

    match decision {
        AnchorDecision::ReturnBroadcast(txid) => return Ok(txid),
        AnchorDecision::InProgress { active_root } => {
            return Err(format!(
                "ANCHOR_IN_PROGRESS: root {active_root} already has an active anchor ceremony \
                 (reserved or signed, not yet broadcast) — refusing to start a concurrent Bitcoin \
                 spend for root {root}"
            ));
        }
        AnchorDecision::Rebroadcast { txid, raw_tx } => {
            // Same root, already signed. REBROADCAST THE EXACT SAME BYTES —
            // never refetch UTXOs or build a second spend for a root that
            // already has a valid signed transaction.
            let bytes = hex::decode(&raw_tx)
                .map_err(|e| format!("stored raw_tx for root {root} is not hex: {e}"))?;
            bitcoin_send_transaction(&SendTransactionRequest { transaction: bytes, network: ic_network()? })
                .await
                .map_err(|e| {
                    // Stays Signed — untouched here. A valid signed
                    // transaction still exists, and the next retry must
                    // rebroadcast it via this same branch, never rebuild.
                    format!("rebroadcast of the existing signed transaction for root {root} failed: {e:?}")
                })?;
            ANCHOR_ATTEMPTS.with(|a| {
                a.borrow_mut().insert(root.clone(), AnchorAttemptState::Broadcast { txid: txid.clone() });
            });
            return Ok(txid);
        }
        AnchorDecision::Reserve => {
            // Reserved above; the fresh ceremony continues below.
        }
    }

    let path: Vec<Vec<u8>> = DERIVATION_PATH_DEFAULT.iter().map(|p| p.to_vec()).collect();
    let ceremony_result = run_fresh_anchor_ceremony(root.clone(), path, fee_rate).await;

    let (txid, raw_tx, inputs) = match ceremony_result {
        Ok(v) => v,
        Err(e) => {
            // Nothing was signed. Record a truthful failed state — Failed is
            // not Reserved/Signed, so this releases the root's exclusivity
            // and permits a controlled retry; nothing was ever spent.
            ANCHOR_ATTEMPTS.with(|a| {
                a.borrow_mut().insert(root.clone(), AnchorAttemptState::Failed { reason: e.clone() });
            });
            return Err(e);
        }
    };

    // ── PERSIST Signed BEFORE invoking bitcoin_send_transaction. ──
    ANCHOR_ATTEMPTS.with(|a| {
        a.borrow_mut()
            .insert(root.clone(), AnchorAttemptState::Signed { txid: txid.clone(), raw_tx: raw_tx.clone(), inputs });
    });

    let bytes = hex::decode(&raw_tx).map_err(|e| format!("assembled raw_tx is not hex: {e}"))?;
    bitcoin_send_transaction(&SendTransactionRequest { transaction: bytes, network: ic_network()? })
        .await
        .map_err(|e| {
            // Stays Signed, deliberately untouched — see the Rebroadcast
            // branch above, which is exactly what the next retry will hit.
            format!("bitcoin_send_transaction rejected the anchor: {e:?}")
        })?;

    ANCHOR_ATTEMPTS.with(|a| {
        a.borrow_mut().insert(root.clone(), AnchorAttemptState::Broadcast { txid: txid.clone() });
    });

    Ok(txid)
}

/// Discover the median fee rate, in sat/vB. `None` means discovery failed —
/// the network call errored, or the canister has no configured network to
/// ask about — and the caller must treat that as `FEE_RATE_UNAVAILABLE`,
/// never silently substitute a guessed rate (P0.3, operator ruling,
/// 2026-08-08: "remove `median_fee_rate().await.unwrap_or(2)`... return
/// FEE_RATE_UNAVAILABLE").
async fn median_fee_rate() -> Option<u64> {
    let network = ic_network().ok()?;
    let p = bitcoin_get_current_fee_percentiles(&GetCurrentFeePercentilesRequest { network })
        .await
        .ok()?;
    // Percentiles are millisatoshi/vB; index 50 is the median. Rounded UP to
    // sat/vB, never down — msat_per_vb_to_sat_per_vb_ceil (btc_anchor_core,
    // pure and host-tested) is the single place that conversion happens.
    p.get(50).map(|m| msat_per_vb_to_sat_per_vb_ceil(u64::from(*m)))
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
