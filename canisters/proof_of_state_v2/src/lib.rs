//! Proof of State v2 — Phase P.
//!
//! ─── WHAT THIS CANISTER IS FOR ──────────────────────────────────────────────
//!
//! It ingests constitutional receipt hashes (H), batches them into a Merkle
//! tree, and anchors each batch's root into Bitcoin via Constitutional
//! Anchor v2 (`btc_signer_psbt`) — reached ONLY as a governed `Principal`
//! over an inter-canister call, never as a Rust dependency (see
//! `Cargo.toml` for why that keeps this canister on ic-cdk 0.13).
//!
//! ─── `n2hhv` IS UNTOUCHED ────────────────────────────────────────────────────
//!
//! `canisters/proof_of_state` (the deployed legacy canister, principal
//! `n2hhv...`) is preserved exactly as-is. This is a NEW canister, not a
//! migration of that one — no code here replaces it, and no code there was
//! modified to build this.
//!
//! ─── WHAT CHANGED FROM THE LEGACY canisters/proof_of_state ──────────────────
//!
//! | legacy defect | v2 |
//! |---|---|
//! | `batch()` hashed the concatenation of receipt IDs — not a real Merkle tree | `pos_core::build_batch`: H -> leaf -> tree -> root, odd nodes promoted never duplicated |
//! | no inclusion proof was ever computed or stored | every batched receipt gets a `Vec<ProofStep>`, checkable via `verify_receipt` |
//! | `issue_receipt` minted a NEW id every call, even for the same input | `issue_receipt(H)` is idempotent by H (`pos_core::decide_issue_receipt`) |
//! | `anchor()` hard-coded `btc_canister_id = "uxrrr-q7777-77774-qaaaq-cai"` | `anchor_signer_principal` — governed `InitArg`, required, never a literal |
//! | on failure, fell back to `Ok(format!("mock_btc_txid_{}", …))` | every failure is an `Err`; `request_anchor` returns only a txid the signer itself returned |
//! | `btc_block_height: Some(800000)` — a literal, never updated from reality | no field is ever set from a literal; `block_height`/`confirmations` are `record_confirmation` PARAMETERS, supplied by reconciliation evidence |
//! | `get_anchor_status()` returned `"confirmed"` merely because `btc_anchor_txid.is_some()` | `BatchAnchorState::AnchorRequested` (broadcast) is a DISTINCT, non-terminal state from `BatchAnchorState::Anchored` — see `record_confirmation` |
//! | heap-only `thread_local!` state — an upgrade silently discarded every receipt and batch | `pre_upgrade`/`post_upgrade` persist config, receipts, pending order, and batches via `ic_cdk::storage::stable_save`/`stable_restore` |
//!
//! ─── THE NEXT GATE ──────────────────────────────────────────────────────────
//!
//! Phase P closes with these tests GREEN. No canister is deployed, no
//! migration is applied, and no historical repair runs from this change.
//! `record_confirmation` accepts confirmation evidence as parameters from an
//! authorized reconciler; WIRING that evidence to a live query of Bitcoin's
//! own confirmation depth is the live testnet CAP-1 Constitutional Anchor
//! Proof — the next gate, not this one.

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use pos_core::{
    build_batch, decide_anchor_request, decide_confirmation, decide_issue_receipt, normalize_h_hex,
    verify_inclusion, BatchAnchorState, ReceiptV2,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Deployment-time configuration. Every field is REQUIRED — there are no
/// defaults, mirroring `btc_signer_psbt::InitArg`'s own discipline: the
/// legacy canister's implicit `btc_canister_id` literal is exactly what
/// this type exists to make impossible to reintroduce.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    /// Constitutional Anchor v2 (`btc_signer_psbt`). The ONLY canister
    /// `request_anchor` calls. Never a literal in source — see the module
    /// docs' defect table.
    pub anchor_signer_principal: Principal,
    /// Who may submit confirmation/reconciliation evidence via
    /// `record_confirmation`. Deliberately distinct from
    /// `anchor_signer_principal`: this principal is what independently
    /// establishes ANCHORED, never mere BROADCAST.
    pub authorized_reconciler_principal: Principal,
    /// Confirmation depth required before Broadcast may become Anchored.
    /// Must be at least 1 — zero confirmations is Broadcast, not Anchored.
    pub min_confirmations: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PosConfig {
    anchor_signer_principal: Principal,
    authorized_reconciler_principal: Principal,
    min_confirmations: u32,
}

fn validate_pos_config(cfg: &PosConfig) -> Result<(), String> {
    if cfg.anchor_signer_principal == Principal::anonymous() {
        return Err("anchor_signer_principal may not be the anonymous principal".to_string());
    }
    if cfg.authorized_reconciler_principal == Principal::anonymous() {
        return Err("authorized_reconciler_principal may not be the anonymous principal".to_string());
    }
    if cfg.min_confirmations == 0 {
        return Err(
            "min_confirmations must be at least 1 — zero confirmations is Broadcast, not Anchored"
                .to_string(),
        );
    }
    Ok(())
}

/// One batch: its members, in the order they were issued, and its
/// relationship to Bitcoin. `anchor_state` is `pos_core::BatchAnchorState`
/// directly — never a bare `Option<String>` txid, which is exactly the
/// shape that let the legacy canister conflate broadcast with anchored.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct BatchV2 {
    pub root_hex: String,
    pub h_hexes: Vec<String>,
    pub created_at_ns: u64,
    pub anchor_state: BatchAnchorState,
}

thread_local! {
    /// Keyed by canonical H hex — this IS the idempotency index
    /// `issue_receipt` consults before ever constructing a new receipt.
    static RECEIPTS: RefCell<BTreeMap<String, ReceiptV2>> = RefCell::new(BTreeMap::new());
    /// Canonical H hexes awaiting the next batch, in issuance order.
    static PENDING: RefCell<Vec<String>> = RefCell::new(Vec::new());
    /// Keyed by root hex.
    static BATCHES: RefCell<BTreeMap<String, BatchV2>> = RefCell::new(BTreeMap::new());
    /// NOT initialised to a usable default — until `init` runs, every
    /// operation that needs configuration is denied.
    static CONFIG: RefCell<Option<PosConfig>> = RefCell::new(None);
}

fn config() -> Result<PosConfig, String> {
    CONFIG.with(|c| {
        c.borrow().clone().ok_or_else(|| {
            "canister is not configured — it was deployed without init arguments".to_string()
        })
    })
}

fn apply_config(arg: InitArg) {
    let cfg = PosConfig {
        anchor_signer_principal: arg.anchor_signer_principal,
        authorized_reconciler_principal: arg.authorized_reconciler_principal,
        min_confirmations: arg.min_confirmations,
    };
    // TRAP rather than start misconfigured — a PoS canister that comes up
    // denying anchor authority to itself is worse than one that fails to
    // come up at all.
    if let Err(e) = validate_pos_config(&cfg) {
        ic_cdk::trap(&format!("refusing to initialise: {e}"));
    }
    CONFIG.with(|c| *c.borrow_mut() = Some(cfg));
}

#[init]
fn init(arg: InitArg) {
    apply_config(arg);
}

/// ── STABLE-STATE PERSISTENCE (Phase P: no heap-only state) ─────────────────
///
/// The legacy `canisters/proof_of_state` has no `#[pre_upgrade]`/
/// `#[post_upgrade]` at all: every receipt and batch it ever recorded lives
/// only in a `thread_local!` `HashMap`/`Vec`, which an upgrade discards
/// silently. This canister persists config, every receipt, the pending
/// order, and every batch — mirroring the exact mechanism
/// `btc_signer_psbt` already uses for its own anchor-attempt state.
#[pre_upgrade]
fn pre_upgrade() {
    let cfg = CONFIG.with(|c| c.borrow().clone());
    let init_arg = cfg.map(|c| InitArg {
        anchor_signer_principal: c.anchor_signer_principal,
        authorized_reconciler_principal: c.authorized_reconciler_principal,
        min_confirmations: c.min_confirmations,
    });
    let receipts: Vec<(String, ReceiptV2)> =
        RECEIPTS.with(|r| r.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    let pending: Vec<String> = PENDING.with(|p| p.borrow().clone());
    let batches: Vec<(String, BatchV2)> =
        BATCHES.with(|b| b.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    ic_cdk::storage::stable_save((init_arg, receipts, pending, batches))
        .expect("config, receipts, pending order, and batches must survive upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    if let Ok((init_arg, receipts, pending, batches)) = ic_cdk::storage::stable_restore::<(
        Option<InitArg>,
        Vec<(String, ReceiptV2)>,
        Vec<String>,
        Vec<(String, BatchV2)>,
    )>() {
        if let Some(arg) = init_arg {
            apply_config(arg);
        }
        RECEIPTS.with(|r| *r.borrow_mut() = receipts.into_iter().collect());
        PENDING.with(|p| *p.borrow_mut() = pending);
        BATCHES.with(|b| *b.borrow_mut() = batches.into_iter().collect());
    }
}

/// Read the configuration. No secrets — the principals and the
/// confirmation threshold are public facts about the deployment.
#[query]
pub fn get_config() -> Option<(String, String, u32)> {
    CONFIG.with(|c| {
        c.borrow().as_ref().map(|cfg| {
            (
                cfg.anchor_signer_principal.to_text(),
                cfg.authorized_reconciler_principal.to_text(),
                cfg.min_confirmations,
            )
        })
    })
}

/// `issue_receipt(H)` — IDEMPOTENT BY H. The legacy canister minted
/// `format!("receipt_{}", ic_cdk::api::time())` on every call, so the same
/// H issued twice produced two distinct, unrelated receipts. Here the
/// canonical-H-keyed lookup happens BEFORE any new receipt is constructed,
/// and `pos_core::decide_issue_receipt` owns the actual idempotency rule —
/// this function only applies it.
#[update]
pub fn issue_receipt(h_hex: String) -> Result<ReceiptV2, String> {
    let canonical = normalize_h_hex(&h_hex)?;
    let now_ns = ic_cdk::api::time();
    let existing = RECEIPTS.with(|r| r.borrow().get(&canonical).cloned());
    let receipt = decide_issue_receipt(&h_hex, now_ns, existing.as_ref())?;
    let is_new = existing.is_none();
    RECEIPTS.with(|r| {
        r.borrow_mut().insert(receipt.h_hex.clone(), receipt.clone());
    });
    if is_new {
        PENDING.with(|p| p.borrow_mut().push(receipt.h_hex.clone()));
    }
    Ok(receipt)
}

/// Drain every pending receipt into one Merkle batch: H -> leaf -> tree ->
/// inclusion proof -> root (`pos_core::build_batch`, the whole pipeline in
/// one call). Every involved receipt is updated with its `batch_root_hex`
/// and its own verifiable `inclusion_proof` — never left as an empty,
/// unverifiable placeholder.
#[update]
pub fn batch_now() -> Result<BatchV2, String> {
    let h_hexes = PENDING.with(|p| {
        let mut p = p.borrow_mut();
        let drained = p.clone();
        p.clear();
        drained
    });
    if h_hexes.is_empty() {
        return Err("no pending receipts to batch".to_string());
    }
    let result = build_batch(&h_hexes).map_err(|e| {
        // Nothing must be left partially drained on refusal — restore the
        // pending queue exactly as it was.
        PENDING.with(|p| *p.borrow_mut() = h_hexes.clone());
        e
    })?;
    let created_at_ns = ic_cdk::api::time();
    let batch = BatchV2 {
        root_hex: result.root_hex.clone(),
        h_hexes: h_hexes.clone(),
        created_at_ns,
        anchor_state: BatchAnchorState::Unanchored,
    };
    BATCHES.with(|b| {
        b.borrow_mut().insert(batch.root_hex.clone(), batch.clone());
    });
    RECEIPTS.with(|r| {
        let mut receipts = r.borrow_mut();
        for (h_hex, proof) in h_hexes.iter().zip(result.proofs.into_iter()) {
            if let Some(receipt) = receipts.get_mut(h_hex) {
                receipt.batch_root_hex = Some(result.root_hex.clone());
                receipt.inclusion_proof = proof;
            }
        }
    });
    Ok(batch)
}

/// Request an anchor for `root_hex` from Constitutional Anchor v2.
///
/// THE SIGNER IS REACHED ONLY THROUGH GOVERNED CONFIG. `cfg.
/// anchor_signer_principal` is read from `CONFIG` — never a literal, unlike
/// the legacy `anchor()`'s `btc_canister_id = "uxrrr-q7777-77774-qaaaq-cai"`.
///
/// NO SYNTHETIC TXID. The legacy `anchor()` fell back to
/// `Ok(format!("mock_btc_txid_{}", …))` on ANY call failure. This function
/// returns a txid ONLY when Constitutional Anchor v2 itself returns one —
/// every other outcome, whether the signer refused or the call itself
/// failed, is a truthful `Err` naming which.
///
/// THIS NEVER WRITES `Anchored`. `decide_anchor_request` (pos_core) can
/// only ever produce `Unanchored -> AnchorRequested` (or reconcile a retry
/// against an existing `AnchorRequested`/`Anchored` of the SAME txid) — see
/// its own docs for why. Reaching `Anchored` requires `record_confirmation`,
/// a wholly separate call.
#[update]
pub async fn request_anchor(root_hex: String) -> Result<String, String> {
    let cfg = config()?;
    let batch = BATCHES
        .with(|b| b.borrow().get(&root_hex).cloned())
        .ok_or_else(|| format!("no batch with root {root_hex} is on record"))?;

    let call_result: ic_cdk::api::call::CallResult<(Result<String, String>,)> = ic_cdk::call(
        cfg.anchor_signer_principal,
        "create_and_broadcast_anchor",
        (root_hex.clone(), 0u64),
    )
    .await;

    let txid = match call_result {
        Ok((Ok(txid),)) => txid,
        Ok((Err(e),)) => {
            return Err(format!("Constitutional Anchor v2 refused to anchor root {root_hex}: {e}"))
        }
        Err((code, msg)) => {
            return Err(format!(
                "inter-canister call to Constitutional Anchor v2 failed: {code:?} {msg}"
            ))
        }
    };

    let next_state = decide_anchor_request(&batch.anchor_state, txid.clone())?;
    BATCHES.with(|b| {
        if let Some(batch) = b.borrow_mut().get_mut(&root_hex) {
            batch.anchor_state = next_state;
        }
    });
    Ok(txid)
}

/// Record confirmation/reconciliation evidence for `root_hex` — the ONLY
/// path to `BatchAnchorState::Anchored`.
///
/// GATED TO THE AUTHORIZED RECONCILER, checked BEFORE any state is
/// consulted or written — the same discipline `btc_signer_psbt::
/// authorize_anchor_caller` uses. `block_height`/`confirmations` are
/// PARAMETERS supplied by the caller's own evidence — never a literal in
/// this function, unlike the legacy `btc_block_height: Some(800000)`.
///
/// Wiring this evidence to a LIVE query of Bitcoin's own confirmation depth
/// (rather than trusting whatever the caller supplies) is the live testnet
/// CAP-1 Constitutional Anchor Proof — the next gate, not this phase.
#[update]
pub fn record_confirmation(
    root_hex: String,
    observed_txid: String,
    block_height: u64,
    confirmations: u32,
) -> Result<BatchAnchorState, String> {
    let caller = ic_cdk::caller().to_text();
    let cfg = config()?;
    if caller != cfg.authorized_reconciler_principal.to_text() {
        return Err(format!(
            "caller {caller} is not the authorized reconciler ({}) — refusing to record \
             confirmation evidence",
            cfg.authorized_reconciler_principal.to_text()
        ));
    }

    let batch = BATCHES
        .with(|b| b.borrow().get(&root_hex).cloned())
        .ok_or_else(|| format!("no batch with root {root_hex} is on record"))?;

    let next_state = decide_confirmation(
        &batch.anchor_state,
        &observed_txid,
        block_height,
        confirmations,
        cfg.min_confirmations,
    )?;

    BATCHES.with(|b| {
        if let Some(batch) = b.borrow_mut().get_mut(&root_hex) {
            batch.anchor_state = next_state.clone();
        }
    });
    Ok(next_state)
}

/// Replay a receipt's own stored inclusion proof against its batch's own
/// recorded root. This is "every receipt stores a VERIFIABLE inclusion
/// path" made checkable, not merely asserted.
#[query]
pub fn verify_receipt(h_hex: String) -> Result<bool, String> {
    let canonical = normalize_h_hex(&h_hex)?;
    let receipt = RECEIPTS
        .with(|r| r.borrow().get(&canonical).cloned())
        .ok_or_else(|| format!("no receipt for H {canonical}"))?;
    let root_hex = receipt
        .batch_root_hex
        .clone()
        .ok_or_else(|| "receipt has not yet been included in a batch".to_string())?;
    verify_inclusion(&receipt.leaf_hex, &receipt.inclusion_proof, &root_hex)
}

#[query]
pub fn get_receipt(h_hex: String) -> Option<ReceiptV2> {
    let canonical = normalize_h_hex(&h_hex).ok()?;
    RECEIPTS.with(|r| r.borrow().get(&canonical).cloned())
}

#[query]
pub fn get_batch(root_hex: String) -> Option<BatchV2> {
    BATCHES.with(|b| b.borrow().get(&root_hex).cloned())
}

#[query]
pub fn get_pending_count() -> u64 {
    PENDING.with(|p| p.borrow().len() as u64)
}

ic_cdk::export_candid!();
