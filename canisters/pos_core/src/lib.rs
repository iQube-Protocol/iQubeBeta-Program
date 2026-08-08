//! Proof of State v2 — pure Merkle-tree construction and anchor-state
//! semantics.
//!
//! ─── WHY THIS MODULE IS PURE ────────────────────────────────────────────────
//!
//! Every function here is deterministic and free of `ic_cdk`, mirroring
//! `btc_anchor_core` (Phase B) for the identical reason: `proof_of_state_v2`
//! is a `cdylib` and cannot be linked into a host test binary, so the
//! H -> leaf -> Merkle tree -> inclusion proof -> root pipeline — and the
//! rule that a broadcast anchor is not yet an anchored one — must live here
//! or Phase P's acceptance contract cannot be regression-tested at all.
//!
//! ─── NORMATIVE ENCODING (Phase P contract) ──────────────────────────────────
//!
//!   * H = exactly 32 raw bytes decoded from its 64-char SHA-256 hex.
//!   * leaf = `SHA256(0x00 || H)`.
//!   * parent = `SHA256(0x01 || left || right)`.
//!   * An odd node is PROMOTED to the next layer unchanged, never duplicated.
//!     Duplicating it is Bitcoin's own historical Merkle-tree defect
//!     (CVE-2012-2459): a duplicated node makes two DIFFERENT sets of leaves
//!     hash to the SAME root, because nothing distinguishes "one leaf,
//!     promoted" from "two identical leaves, paired". Promotion removes the
//!     ambiguity structurally rather than by convention.
//!   * The `0x00`/`0x01` domain-separation prefixes on leaf/parent hashing
//!     exist so a leaf hash can never be replayed as an internal node's hash
//!     (or vice versa) — without them, an attacker could potentially forge
//!     an inclusion proof by relabelling which layer a given hash "belongs"
//!     to. This is the second-preimage attack a bare, unprefixed
//!     Merkle tree is classically vulnerable to.
//!
//! All hex throughout this module's public API is lowercase, and every hash
//! is exactly 32 raw bytes (64 hex chars) — mirroring `btc_anchor_core`'s own
//! hex-string-first convention (`AnchorInput.txid_hex`, `normalize_root_hex`)
//! rather than exposing raw `[u8; 32]` arrays across the public surface.

use candid::{CandidType, Deserialize};
use sha2::{Digest, Sha256};

/// Canonicalise a 64-hex-char value to lowercase, validating it decodes to
/// exactly 32 bytes. Used for H (the input receipt hash) — NOT for a
/// Merkle root or leaf hash, which have their own call sites below, but the
/// validation rule is identical: this crate's one 32-byte-hex canonicaliser.
pub fn normalize_h_hex(h_hex: &str) -> Result<String, String> {
    decode_32(h_hex, "H").map(|b| hex::encode(b))
}

fn decode_32(hex_str: &str, what: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex_str).map_err(|e| format!("{what} is not hex: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!(
            "{what} must be exactly 32 raw bytes (64 hex chars), got {} bytes",
            bytes.len()
        ));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// `SHA256(SHA256(x))`-free — this is Merkle hashing, not Bitcoin's own
/// double-SHA256. A single SHA256 with a domain-separation prefix is the
/// normative encoding here; do not import Bitcoin's `sha256d` convention by
/// habit.
fn sha256(data: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// `leaf = SHA256(0x00 || H)`. Takes H as hex, normalising it first — the
/// canonical entry point every H must pass through before it becomes a leaf.
pub fn leaf_hash_hex(h_hex: &str) -> Result<String, String> {
    let h = decode_32(h_hex, "H")?;
    let mut preimage = Vec::with_capacity(33);
    preimage.push(0x00);
    preimage.extend_from_slice(&h);
    Ok(hex::encode(sha256(&preimage)))
}

/// `parent = SHA256(0x01 || left || right)`. `left`/`right` are already-
/// hashed 32-byte values (leaf or parent hashes) — never raw H.
pub fn parent_hash_hex(left_hex: &str, right_hex: &str) -> Result<String, String> {
    let left = decode_32(left_hex, "left")?;
    let right = decode_32(right_hex, "right")?;
    let mut preimage = Vec::with_capacity(65);
    preimage.push(0x01);
    preimage.extend_from_slice(&left);
    preimage.extend_from_slice(&right);
    Ok(hex::encode(sha256(&preimage)))
}

/// A built Merkle tree, bottom layer first (leaves) up to a single-element
/// top layer (the root). Not `CandidType` — ephemeral, built fresh each
/// batch; only `root_hex` and each leaf's `inclusion_proof` are persisted.
pub struct MerkleTree {
    layers: Vec<Vec<[u8; 32]>>,
}

impl MerkleTree {
    pub fn root_hex(&self) -> String {
        // `build_merkle_tree` never returns with an empty top layer — see
        // its own refusal on zero leaves — so this always has exactly one
        // element.
        hex::encode(self.layers.last().expect("a built tree always has at least one layer")[0])
    }

    pub fn leaf_count(&self) -> usize {
        self.layers[0].len()
    }
}

/// Build a Merkle tree from an ORDERED list of leaf hashes (already
/// `leaf_hash_hex`-ed — never raw H). Odd nodes are PROMOTED to the next
/// layer unchanged, never duplicated (see the module docs on
/// CVE-2012-2459).
pub fn build_merkle_tree(leaf_hexes: Vec<String>) -> Result<MerkleTree, String> {
    if leaf_hexes.is_empty() {
        return Err("cannot build a Merkle tree with zero leaves".to_string());
    }
    let leaves: Vec<[u8; 32]> =
        leaf_hexes.iter().map(|h| decode_32(h, "leaf")).collect::<Result<_, _>>()?;

    let mut layers = vec![leaves];
    while layers.last().expect("layers is never empty").len() > 1 {
        let current = layers.last().expect("layers is never empty");
        let mut next = Vec::with_capacity(current.len().div_ceil(2));
        let mut i = 0;
        while i < current.len() {
            if i + 1 < current.len() {
                let left_hex = hex::encode(current[i]);
                let right_hex = hex::encode(current[i + 1]);
                let parent = decode_32(
                    &parent_hash_hex(&left_hex, &right_hex).expect("both sides are already valid 32-byte hex"),
                    "parent",
                )
                .expect("parent_hash_hex always returns valid 32-byte hex");
                next.push(parent);
            } else {
                // Odd node: PROMOTED unchanged, never duplicated.
                next.push(current[i]);
            }
            i += 2;
        }
        layers.push(next);
    }
    Ok(MerkleTree { layers })
}

/// Which side of the current node the sibling sits on, in a `ProofStep`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum Side {
    /// The sibling is to the left: `parent = parent_hash(sibling, current)`.
    Left,
    /// The sibling is to the right: `parent = parent_hash(current, sibling)`.
    Right,
}

/// One step of an inclusion proof, replayed bottom-up from a leaf to the
/// root. `CandidType, Deserialize` so a full proof can be stored on a
/// receipt and survive both a Candid query response and a stable-memory
/// upgrade round trip.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum ProofStep {
    /// Combine the current hash with `hash_hex` on the given `side`.
    Sibling { hash_hex: String, side: Side },
    /// This node had no sibling at this level (an odd-length layer) and was
    /// PROMOTED unchanged — nothing to combine; the current hash passes
    /// through to the next level as-is.
    Promoted,
}

/// Compute the inclusion proof for the leaf at `leaf_index`, walking every
/// layer of `tree` from the leaves up to (but not including) the root
/// layer.
pub fn inclusion_proof(tree: &MerkleTree, leaf_index: usize) -> Result<Vec<ProofStep>, String> {
    if leaf_index >= tree.leaf_count() {
        return Err(format!(
            "leaf index {leaf_index} is out of range for a tree with {} leaves",
            tree.leaf_count()
        ));
    }
    let mut steps = Vec::new();
    let mut idx = leaf_index;
    for layer in &tree.layers[..tree.layers.len() - 1] {
        if idx % 2 == 0 {
            if idx + 1 < layer.len() {
                steps.push(ProofStep::Sibling { hash_hex: hex::encode(layer[idx + 1]), side: Side::Right });
            } else {
                // Last index of an odd-length layer: no sibling, promoted.
                steps.push(ProofStep::Promoted);
            }
        } else {
            steps.push(ProofStep::Sibling { hash_hex: hex::encode(layer[idx - 1]), side: Side::Left });
        }
        idx /= 2;
    }
    Ok(steps)
}

/// Replay `proof` starting from `leaf_hex` and check the result equals
/// `root_hex`. This is the "every receipt stores a VERIFIABLE inclusion
/// path" half of the contract made checkable: a receipt whose proof does
/// not replay to its batch's own recorded root is not actually included in
/// that batch, whatever the stored data claims.
pub fn verify_inclusion(leaf_hex: &str, proof: &[ProofStep], root_hex: &str) -> Result<bool, String> {
    let mut cur = decode_32(leaf_hex, "leaf")?;
    for step in proof {
        cur = match step {
            ProofStep::Sibling { hash_hex, side: Side::Left } => decode_32(
                &parent_hash_hex(hash_hex, &hex::encode(cur))?,
                "parent",
            )?,
            ProofStep::Sibling { hash_hex, side: Side::Right } => decode_32(
                &parent_hash_hex(&hex::encode(cur), hash_hex)?,
                "parent",
            )?,
            ProofStep::Promoted => cur,
        };
    }
    let root = decode_32(root_hex, "root")?;
    Ok(cur == root)
}

/// The full pipeline's output for one batch: the root, and — in the SAME
/// order as the input `h_hexes` — each member's inclusion proof.
pub struct BatchResult {
    pub root_hex: String,
    pub proofs: Vec<Vec<ProofStep>>,
}

/// H -> leaf -> Merkle tree -> inclusion proof -> root, for an ORDERED list
/// of H values. This is the one function that runs the whole pipeline; the
/// canister calls it once per batch rather than re-deriving any step.
pub fn build_batch(h_hexes: &[String]) -> Result<BatchResult, String> {
    if h_hexes.is_empty() {
        return Err("cannot batch zero receipts".to_string());
    }
    let leaf_hexes: Vec<String> = h_hexes.iter().map(|h| leaf_hash_hex(h)).collect::<Result<_, _>>()?;
    let tree = build_merkle_tree(leaf_hexes)?;
    let root_hex = tree.root_hex();
    let proofs: Vec<Vec<ProofStep>> =
        (0..h_hexes.len()).map(|i| inclusion_proof(&tree, i)).collect::<Result<_, _>>()?;
    Ok(BatchResult { root_hex, proofs })
}

// ─── RECEIPTS — issue_receipt(H) is idempotent by H ────────────────────────

/// One issued receipt. `CandidType, Deserialize` so it survives both a
/// Candid query response and the canister's stable-memory upgrade round
/// trip (Phase P: "the legacy canister's heap-only state problem must not
/// be reproduced in v2").
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct ReceiptV2 {
    /// Canonical (lowercase, validated) 64-char hex of H.
    pub h_hex: String,
    /// `leaf_hash_hex(h_hex)` — computed once, at issuance.
    pub leaf_hex: String,
    pub issued_at_ns: u64,
    /// `None` until this receipt's H is included in a batch.
    pub batch_root_hex: Option<String>,
    /// Empty until batched; thereafter the verifiable inclusion path this
    /// receipt's leaf takes to `batch_root_hex`.
    pub inclusion_proof: Vec<ProofStep>,
}

/// Decide the outcome of `issue_receipt(H)`. IDEMPOTENT BY H: if `existing`
/// is `Some`, that exact receipt is returned UNCHANGED — never a new
/// `issued_at_ns`, never a fresh entry alongside it. The canister looks up
/// `existing` from its own H-keyed store and passes it in; this function
/// owns the decision, the canister only applies it (the same division of
/// labour as `decide_anchor_attempt` in Phase B).
pub fn decide_issue_receipt(
    h_hex: &str,
    now_ns: u64,
    existing: Option<&ReceiptV2>,
) -> Result<ReceiptV2, String> {
    let canonical = normalize_h_hex(h_hex)?;
    if let Some(existing) = existing {
        return Ok(existing.clone());
    }
    let leaf_hex = leaf_hash_hex(&canonical)?;
    Ok(ReceiptV2 {
        h_hex: canonical,
        leaf_hex,
        issued_at_ns: now_ns,
        batch_root_hex: None,
        inclusion_proof: Vec::new(),
    })
}

// ─── ANCHOR STATE — Broadcast is not Anchored ──────────────────────────────
//
// Phase P adds one semantic invariant beyond Phase B's own contract:
// `Broadcast` (Constitutional Anchor v2's own terminal state, meaning the
// network ACCEPTED the transaction) is NOT `Anchored` from Proof of State's
// point of view. A broadcast transaction may be dropped, replaced, or
// simply never confirm. Confirmation/reconciliation must INDEPENDENTLY
// establish anchoring — never by trusting the anchor-request call's own
// return value, and never through the same code path that made the
// request.

/// One batch root's relationship to Bitcoin. Persisted alongside the batch.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum BatchAnchorState {
    /// No anchor request has been made yet for this root.
    Unanchored,
    /// Constitutional Anchor v2 returned `txid` for this root — the
    /// transaction has been BROADCAST. THIS IS NOT ANCHORED. `Anchored` is
    /// reachable ONLY through `decide_confirmation`, from independently
    /// supplied confirmation evidence — never from this state alone, and
    /// never from `decide_anchor_request`'s own success.
    AnchorRequested { txid: String },
    /// Confirmation/reconciliation has INDEPENDENTLY established that
    /// `txid` reached at least the required confirmation depth at
    /// `block_height`. This is the only state under which a receipt's
    /// inclusion proof may be treated as CONSTITUTIONALLY anchored.
    Anchored { txid: String, block_height: u64, confirmations: u32 },
}

/// Decide the outcome of requesting (or retrying a request for) an anchor
/// for `current`'s root, given the `txid` Constitutional Anchor v2 just
/// returned. Constitutional Anchor v2 is itself idempotent per root (P0.3):
/// the SAME root always returns the SAME txid, whether fresh, rebroadcast,
/// or already-broadcast. A different txid for a request already recorded
/// under a different one means something upstream is broken, and this
/// function refuses rather than silently overwriting the record.
///
/// NEVER returns `Anchored` — that transition belongs to
/// `decide_confirmation` alone, from evidence this function never sees.
pub fn decide_anchor_request(current: &BatchAnchorState, txid: String) -> Result<BatchAnchorState, String> {
    match current {
        BatchAnchorState::Unanchored => Ok(BatchAnchorState::AnchorRequested { txid }),
        BatchAnchorState::AnchorRequested { txid: existing } => {
            if *existing == txid {
                Ok(BatchAnchorState::AnchorRequested { txid })
            } else {
                Err(format!(
                    "Constitutional Anchor v2 returned txid {txid} for a root already \
                     AnchorRequested under txid {existing} — Anchor v2's own idempotency \
                     guarantees the same root always returns the same txid, so a mismatch means \
                     something upstream is broken; refusing to overwrite the recorded txid"
                ))
            }
        }
        BatchAnchorState::Anchored { txid: existing, .. } => {
            if *existing == txid {
                Ok(current.clone())
            } else {
                Err(format!(
                    "root is already Anchored under txid {existing}; refusing to move backwards \
                     toward AnchorRequested for a different txid {txid}"
                ))
            }
        }
    }
}

/// Decide the outcome of confirmation/reconciliation evidence for
/// `current`'s root. This is the ONLY path to `Anchored`, and it is
/// deliberately independent of `decide_anchor_request`:
///
///   * the evidence's `observed_txid` must match the txid THIS root's own
///     anchor request actually recorded — confirmation evidence for a
///     transaction this root never requested must never anchor it;
///   * `confirmations` must meet or exceed `min_confirmations` — a merely
///     broadcast (zero-confirmation) transaction is refused by construction,
///     which is the "Broadcast is not Anchored" invariant made operational.
///
/// Calling this with no prior anchor request (`Unanchored`) is refused:
/// there is nothing to confirm, and skipping straight to `Anchored` without
/// ever having been `AnchorRequested` is exactly the conflation this
/// function exists to forbid.
pub fn decide_confirmation(
    current: &BatchAnchorState,
    observed_txid: &str,
    block_height: u64,
    confirmations: u32,
    min_confirmations: u32,
) -> Result<BatchAnchorState, String> {
    let requested_txid = match current {
        BatchAnchorState::Unanchored => {
            return Err(
                "no anchor has been requested for this root — there is nothing to confirm; \
                 skipping straight from Unanchored to Anchored is exactly the Broadcast-is-not- \
                 Anchored conflation this function exists to forbid"
                    .to_string(),
            )
        }
        BatchAnchorState::AnchorRequested { txid } => txid,
        BatchAnchorState::Anchored { txid, .. } => txid,
    };
    if requested_txid != observed_txid {
        return Err(format!(
            "confirmation evidence names txid {observed_txid}, but this root's own anchor \
             request recorded txid {requested_txid} — refusing to anchor a root on evidence for \
             a transaction it never requested"
        ));
    }
    if confirmations < min_confirmations {
        return Err(format!(
            "txid {observed_txid} has only {confirmations} confirmation(s), below the required \
             {min_confirmations} — Broadcast is not Anchored; refusing to advance until \
             confirmation depth is independently established"
        ));
    }
    Ok(BatchAnchorState::Anchored { txid: observed_txid.to_string(), block_height, confirmations })
}

/// Phase P acceptance contract.
#[cfg(test)]
#[path = "acceptance_tests.rs"]
mod acceptance_tests;
