//! Phase P acceptance tests — the Proof of State v2 contract.
//!
//! ─── PROVENANCE OF THE ORACLE VECTORS ───────────────────────────────────────
//!
//! The exact hash values below were computed independently, in Python
//! (`hashlib.sha256`), from the normative spec itself — NOT derived from
//! this crate's own code. A test that only checks a function against
//! itself (e.g. "root of a 1-leaf tree equals whatever `leaf_hash_hex`
//! returns") cannot catch a consistently-wrong implementation; an external
//! oracle can. This mirrors Phase B's own
//! `oracle_rust_bitcoin_agrees_our_address_matches_our_change_script`.
//!
//! H1..H5 are the hex strings `"11"*32`, `"22"*32`, `"33"*32`, `"44"*32`,
//! `"55"*32`. L1..L5 are `leaf(Hi) = SHA256(0x00 || Hi)`. `pXY` is
//! `parent(LX, LY) = SHA256(0x01 || LX || LY)`.

use super::*;

const H1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const H2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const H3: &str = "3333333333333333333333333333333333333333333333333333333333333333";

const L1: &str = "4635e1fa62a599a7880a8d14a56f720a1d40f6e5448ab5a5e39bedc8bd87fa8e";
const L2: &str = "bc6f27de60abf5319d16ff4c98fe3c42022c84f6a7a2b207c8df19b0ec3d8d58";
const L3: &str = "5e5caeafc27155c368b6f201107d6f8b270747ce636ac5174a56c6e12ef89ad1";
const L4: &str = "a3d6d11f618ad57d28b109ac4c9ab4e76d0a5f6f73447e9bf3f83ee66037e6c4";
const L5: &str = "a23e5f60b577afd1d5d31a3efa2c95b1586648dbb4f0aa254d3de36cf3966d85";

const P12: &str = "cc15b132263fd4fd2748c0e7cb9e1c4ad0afe70fcf9382ee644c4da8af0286a5";
const P1234: &str = "0bdd9ab2021b08e98613d9495870a4e3445ddc83e83bcff6a6d6f1ccd5120105";

const ROOT_1: &str = L1;
const ROOT_2: &str = P12;
const ROOT_3: &str = "9bee4401962e94b921336a7910a5a9718836ffcbc545dde0a3f34d858beb5752";
const ROOT_4: &str = "0bdd9ab2021b08e98613d9495870a4e3445ddc83e83bcff6a6d6f1ccd5120105";
const ROOT_5: &str = "f9eb7b7e828c224c498136965f40db8848b83409caa6124306414523fa84d568";

fn h(n: u8) -> String {
    hex::encode([n; 32])
}

// ─── H -> LEAF -> PARENT (independent oracle) ──────────────────────────────

#[test]
fn p_leaf_hash_matches_independent_oracle() {
    assert_eq!(
        leaf_hash_hex(H1).expect("H1 is valid"),
        L1,
        "leaf = SHA256(0x00 || H) — a leaf computed WITHOUT the 0x00 domain-separation prefix \
         would produce a different value and could collide with an internal-node hash elsewhere \
         in the tree, which is exactly the second-preimage risk the prefix exists to remove"
    );
    assert_eq!(leaf_hash_hex(H2).expect("H2 is valid"), L2);
}

#[test]
fn p_parent_hash_matches_independent_oracle() {
    assert_eq!(
        parent_hash_hex(L1, L2).expect("both sides valid"),
        P12,
        "parent = SHA256(0x01 || left || right) — computed WITHOUT the 0x01 prefix, this would \
         collide with a leaf hash of the same 64 raw bytes treated as one H"
    );
}

#[test]
fn p_leaf_hash_normalizes_case_before_hashing() {
    // Uppercase input must canonicalise to the SAME leaf as lowercase — the
    // hash is over raw BYTES, never over the hex text's casing.
    assert_eq!(leaf_hash_hex(&H1.to_uppercase()).expect("uppercase hex is still valid hex"), L1);
}

#[test]
fn p_h_must_be_exactly_32_raw_bytes_from_64_char_hex() {
    assert!(normalize_h_hex("deadbeef").is_err(), "32 hex chars (16 bytes) must be refused");
    assert!(normalize_h_hex(&"aa".repeat(31)).is_err(), "31 bytes must be refused");
    assert!(normalize_h_hex(&"aa".repeat(33)).is_err(), "33 bytes must be refused");
    assert!(normalize_h_hex("not-hex-at-all-".repeat(5).as_str()).is_err(), "non-hex must be refused");
    assert_eq!(normalize_h_hex(H1).expect("H1 is exactly 32 bytes"), H1, "a valid 64-char hex round-trips exactly");
}

// ─── MERKLE TREE — odd nodes PROMOTED, never duplicated ────────────────────
//
// `p_three_leaf_tree_promotes_the_odd_node_never_duplicates` is the direct
// analogue of Bitcoin's own CVE-2012-2459: an implementation that
// DUPLICATES the odd node (`parent(L3, L3)` instead of promoting `L3`
// unchanged) computes `parent(P12, parent(L3, L3))`, a DIFFERENT root from
// `ROOT_3`. This test's independent oracle can only be satisfied by
// promotion.

#[test]
fn p_single_leaf_tree_root_is_the_leaf_itself() {
    let tree = build_merkle_tree(vec![L1.to_string()]).expect("one leaf is a valid tree");
    assert_eq!(tree.root_hex(), ROOT_1, "a 1-leaf tree's root is that leaf, unchanged");
}

#[test]
fn p_two_leaf_tree_root_matches_oracle() {
    let tree = build_merkle_tree(vec![L1.to_string(), L2.to_string()]).expect("two leaves");
    assert_eq!(tree.root_hex(), ROOT_2);
}

#[test]
fn p_three_leaf_tree_promotes_the_odd_node_never_duplicates() {
    let tree = build_merkle_tree(vec![L1.to_string(), L2.to_string(), L3.to_string()]).expect("three leaves");
    assert_eq!(
        tree.root_hex(),
        ROOT_3,
        "root_3 = parent(parent(L1,L2), L3) — L3 PROMOTED unchanged. A root of \
         parent(parent(L1,L2), parent(L3,L3)) (L3 DUPLICATED instead) is Bitcoin's own historical \
         Merkle defect, CVE-2012-2459, and would fail this exact assertion"
    );
}

#[test]
fn p_four_leaf_tree_root_matches_oracle() {
    let tree =
        build_merkle_tree(vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string()]).expect("4 leaves");
    assert_eq!(tree.root_hex(), ROOT_4, "an even leaf count needs no promotion at any level");
}

#[test]
fn p_five_leaf_tree_promotes_at_two_different_levels() {
    let tree = build_merkle_tree(vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string(), L5.to_string()])
        .expect("5 leaves");
    assert_eq!(
        tree.root_hex(),
        ROOT_5,
        "root_5 = parent(parent(parent(L1,L2),parent(L3,L4)), L5) — L5 is promoted at the LEAF \
         layer (5 is odd) AND again at the next layer (3 elements is odd), proving promotion is \
         applied independently at every layer, not only the bottom one"
    );
}

#[test]
fn p_build_merkle_tree_refuses_zero_leaves() {
    let err = build_merkle_tree(vec![]).err().expect("zero leaves must be refused, not silently accepted");
    assert!(err.contains("zero leaves"), "refusal must name the reason: {err}");
}

// ─── INCLUSION PROOF — every receipt stores a VERIFIABLE inclusion path ────

#[test]
fn p_inclusion_proof_for_a_normal_leaf_matches_the_hand_computed_steps_and_replays_to_the_root() {
    let leaves = vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string(), L5.to_string()];
    let tree = build_merkle_tree(leaves).expect("5 leaves");

    // Leaf index 2 = H3/L3. Hand-computed steps: sibling L4 (Right) at the
    // leaf layer, sibling P12 (Left) at the next, sibling L5 (Right) at the
    // top — see the module docs' oracle derivation.
    let proof = inclusion_proof(&tree, 2).expect("index 2 is in range");
    assert_eq!(
        proof,
        vec![
            ProofStep::Sibling { hash_hex: L4.to_string(), side: Side::Right },
            ProofStep::Sibling { hash_hex: P12.to_string(), side: Side::Left },
            ProofStep::Sibling { hash_hex: L5.to_string(), side: Side::Right },
        ]
    );
    assert_eq!(
        verify_inclusion(L3, &proof, ROOT_5),
        Ok(true),
        "replaying the proof from L3 must reach ROOT_5 exactly"
    );
}

#[test]
fn p_inclusion_proof_for_a_promoted_leaf_matches_the_hand_computed_steps_and_replays_to_the_root() {
    let leaves = vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string(), L5.to_string()];
    let tree = build_merkle_tree(leaves).expect("5 leaves");

    // Leaf index 4 = H5/L5, promoted at BOTH the leaf layer and the next —
    // its proof must record TWO `Promoted` steps before its one real
    // sibling combination, never silently skip them or assume every level
    // has a sibling.
    let proof = inclusion_proof(&tree, 4).expect("index 4 is in range");
    assert_eq!(
        proof,
        vec![ProofStep::Promoted, ProofStep::Promoted, ProofStep::Sibling { hash_hex: P1234.to_string(), side: Side::Left }]
    );
    assert_eq!(verify_inclusion(L5, &proof, ROOT_5), Ok(true));
}

#[test]
fn p_a_tampered_sibling_hash_fails_verification_without_erroring() {
    let leaves = vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string()];
    let tree = build_merkle_tree(leaves).expect("4 leaves");
    let mut proof = inclusion_proof(&tree, 0).expect("index 0 is in range");
    // Flip the FIRST sibling hash to an unrelated, still well-formed 32-byte
    // hex value — the proof remains syntactically valid, so this must be
    // caught by RECOMPUTATION, not input validation.
    if let ProofStep::Sibling { hash_hex, .. } = &mut proof[0] {
        *hash_hex = h(0xff);
    }
    assert_eq!(
        verify_inclusion(L1, &proof, ROOT_4),
        Ok(false),
        "a tampered sibling must make verification fail, not merely error — the whole point of a \
         Merkle proof is that a wrong sibling cannot be papered over"
    );
}

#[test]
fn p_a_proof_for_the_wrong_leaf_fails_verification() {
    let leaves = vec![L1.to_string(), L2.to_string(), L3.to_string(), L4.to_string()];
    let tree = build_merkle_tree(leaves).expect("4 leaves");
    let proof_for_l1 = inclusion_proof(&tree, 0).expect("index 0 is in range");
    assert_eq!(
        verify_inclusion(L2, &proof_for_l1, ROOT_4),
        Ok(false),
        "L1's own proof must not also validate L2 — a proof is bound to the specific leaf it was \
         computed for"
    );
}

#[test]
fn p_inclusion_proof_refuses_an_out_of_range_leaf_index() {
    let tree = build_merkle_tree(vec![L1.to_string(), L2.to_string()]).expect("2 leaves");
    assert!(inclusion_proof(&tree, 2).is_err(), "index 2 is out of range for a 2-leaf tree");
}

// ─── THE FULL PIPELINE — H -> leaf -> Merkle tree -> inclusion proof -> root ─

#[test]
fn p_build_batch_runs_the_whole_pipeline_and_every_member_verifies() {
    let h_hexes = vec![H1.to_string(), H2.to_string(), H3.to_string()];
    let result = build_batch(&h_hexes).expect("3 H values batch successfully");
    assert_eq!(result.root_hex, ROOT_3, "the batch's root must be the SAME root the raw-leaf oracle computed");
    assert_eq!(result.proofs.len(), 3);

    for (h_hex, proof) in h_hexes.iter().zip(result.proofs.iter()) {
        let leaf_hex = leaf_hash_hex(h_hex).expect("valid H");
        assert_eq!(
            verify_inclusion(&leaf_hex, proof, &result.root_hex),
            Ok(true),
            "every batched H's own stored proof must independently verify against the batch's root"
        );
    }
}

#[test]
fn p_build_batch_refuses_zero_receipts() {
    assert!(build_batch(&[]).is_err(), "an empty batch must be refused, not silently produce an empty root");
}

// ─── issue_receipt(H) IS IDEMPOTENT BY H ────────────────────────────────────

#[test]
fn p_issue_receipt_is_idempotent_by_h() {
    let first = decide_issue_receipt(H1, 1_000, None).expect("first issuance succeeds");
    // A SECOND call for the SAME H, at a LATER time, with the FIRST
    // receipt passed as `existing` (exactly what the canister's own H-keyed
    // lookup would supply) — must return the FIRST receipt UNCHANGED, not a
    // new one stamped with the later time.
    let second = decide_issue_receipt(H1, 99_999, Some(&first)).expect("idempotent replay succeeds");
    assert_eq!(
        second, first,
        "issue_receipt(H) must be idempotent by H — a second call for the same H must return the \
         SAME receipt (same issued_at_ns), never mint a second, unrelated receipt the way the \
         legacy canister's format!(\"receipt_{{}}\", ic_cdk::api::time()) did"
    );
    assert_eq!(second.issued_at_ns, 1_000, "the ORIGINAL issuance time must survive, not the retry's");
}

#[test]
fn p_issue_receipt_canonicalises_h_before_using_it_as_the_idempotency_key() {
    let lower = decide_issue_receipt(H1, 1_000, None).expect("lowercase H");
    let upper = decide_issue_receipt(&H1.to_uppercase(), 2_000, None).expect("uppercase H, no existing passed");
    assert_eq!(
        lower.h_hex, upper.h_hex,
        "the SAME H spelled in a different case must canonicalise to the SAME key — a canister \
         that looked up RECEIPTS by the raw, un-normalised string would treat these as two \
         different H values and lose idempotency"
    );
}

#[test]
fn p_issue_receipt_refuses_malformed_h() {
    assert!(decide_issue_receipt("not-valid-hex", 1_000, None).is_err());
}

// ─── ANCHOR STATE — Broadcast is not Anchored ──────────────────────────────

#[test]
fn p_decide_anchor_request_moves_unanchored_to_anchor_requested() {
    assert_eq!(
        decide_anchor_request(&BatchAnchorState::Unanchored, "txid-a".to_string()),
        Ok(BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() })
    );
}

#[test]
fn p_decide_anchor_request_same_txid_retry_is_idempotent() {
    let requested = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    assert_eq!(decide_anchor_request(&requested, "txid-a".to_string()), Ok(requested));
}

#[test]
fn p_decide_anchor_request_refuses_a_mismatched_txid_for_an_already_requested_root() {
    let requested = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    let err = decide_anchor_request(&requested, "txid-DIFFERENT".to_string())
        .err()
        .expect("Constitutional Anchor v2's own per-root idempotency makes a mismatch a bug upstream");
    assert!(err.contains("idempotency"), "refusal must name why a mismatch is refused: {err}");
}

#[test]
fn p_decide_anchor_request_never_produces_anchored() {
    // THE core invariant, exercised from every reachable starting state:
    // requesting (or retrying a request for) an anchor can NEVER by itself
    // reach Anchored. Broadcast is not Anchored.
    let starting_states = vec![
        BatchAnchorState::Unanchored,
        BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() },
        BatchAnchorState::Anchored { txid: "txid-a".to_string(), block_height: 900_000, confirmations: 6 },
    ];
    for state in starting_states {
        if let Ok(result) = decide_anchor_request(&state, "txid-a".to_string()) {
            assert!(
                !matches!(result, BatchAnchorState::Anchored { .. }) || matches!(state, BatchAnchorState::Anchored { .. }),
                "decide_anchor_request must never PRODUCE Anchored from a non-Anchored starting \
                 state — only decide_confirmation may reach Anchored, from independent evidence"
            );
        }
    }
}

#[test]
fn p_decide_confirmation_refuses_when_no_anchor_was_ever_requested() {
    let err = decide_confirmation(&BatchAnchorState::Unanchored, "txid-a", 900_000, 6, 6)
        .err()
        .expect("Unanchored -> Anchored directly must be refused");
    assert!(
        err.contains("nothing to confirm"),
        "refusal must name that there is nothing to confirm: {err}"
    );
}

#[test]
fn p_decide_confirmation_refuses_a_mismatched_txid() {
    let requested = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    let err = decide_confirmation(&requested, "txid-DIFFERENT", 900_000, 6, 6)
        .err()
        .expect("evidence for a transaction this root never requested must be refused");
    assert!(err.contains("never requested"), "refusal must name the mismatch: {err}");
}

#[test]
fn p_decide_confirmation_refuses_insufficient_confirmations() {
    // THIS is "Broadcast is not Anchored" made operational: AnchorRequested
    // plus ONE confirmation is still refused when the required depth is 6.
    let requested = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    let err = decide_confirmation(&requested, "txid-a", 900_000, 1, 6)
        .err()
        .expect("1 confirmation must not satisfy a 6-confirmation requirement");
    assert!(err.contains("Broadcast is not Anchored"), "refusal must name the invariant by name: {err}");
}

#[test]
fn p_decide_confirmation_accepts_sufficient_matching_confirmations() {
    let requested = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    assert_eq!(
        decide_confirmation(&requested, "txid-a", 900_000, 6, 6),
        Ok(BatchAnchorState::Anchored { txid: "txid-a".to_string(), block_height: 900_000, confirmations: 6 })
    );
}

#[test]
fn p_broadcast_state_alone_is_never_equal_to_anchored() {
    let broadcast = BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() };
    let anchored = BatchAnchorState::Anchored { txid: "txid-a".to_string(), block_height: 900_000, confirmations: 6 };
    assert_ne!(
        broadcast, anchored,
        "AnchorRequested (broadcast) and Anchored are DIFFERENT states even for the identical txid \
         — the semantic invariant this phase adds, stated as its own standalone assertion"
    );
}

#[test]
fn p_anchor_state_and_proof_steps_survive_a_stable_memory_round_trip() {
    use candid::{Decode, Encode};

    let states = vec![
        BatchAnchorState::Unanchored,
        BatchAnchorState::AnchorRequested { txid: "txid-a".to_string() },
        BatchAnchorState::Anchored { txid: "txid-a".to_string(), block_height: 900_000, confirmations: 6 },
    ];
    for state in states {
        let bytes = Encode!(&state).expect("BatchAnchorState must Candid-encode");
        let round_tripped: BatchAnchorState = Decode!(&bytes, BatchAnchorState).expect("must decode");
        assert_eq!(round_tripped, state);
    }

    let receipt = ReceiptV2 {
        h_hex: H1.to_string(),
        leaf_hex: L1.to_string(),
        issued_at_ns: 1_000,
        batch_root_hex: Some(ROOT_1.to_string()),
        inclusion_proof: vec![ProofStep::Sibling { hash_hex: L2.to_string(), side: Side::Right }, ProofStep::Promoted],
    };
    let bytes = Encode!(&receipt).expect("ReceiptV2 must Candid-encode");
    let round_tripped: ReceiptV2 = Decode!(&bytes, ReceiptV2).expect("must decode");
    assert_eq!(round_tripped, receipt, "a receipt's inclusion proof — including a Promoted step — must round-trip");
}

// ─── pos_core MUST NEVER TAKE A CDK DEPENDENCY ──────────────────────────────

#[test]
fn core_stays_cdk_free() {
    let manifest = include_str!("../Cargo.toml");
    let mut in_deps = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if t.starts_with('[') {
            in_deps = t == "[dependencies]";
            continue;
        }
        if in_deps {
            let name = t.split(['=', ' ']).next().unwrap_or("");
            assert!(
                !name.eq_ignore_ascii_case("ic-cdk") && !name.to_ascii_lowercase().starts_with("ic-cdk-"),
                "pos_core must stay CDK-free — found dependency declaration: {t}"
            );
        }
    }
}

// ─── STRUCTURAL GUARANTEES ABOUT proof_of_state_v2's OWN WIRING ────────────
//
// These mirror `btc_anchor_core`'s `canister_src_without_comments()`
// canaries: properties that cannot be host-tested by calling the canister
// (it is a `cdylib`, and the properties concern IC-only mechanisms —
// upgrade persistence, caller authorization) are instead pinned by finding
// markers in the comment-stripped source, so they are re-checked on every
// future edit rather than trusted to have survived one.

fn canister_src_without_comments() -> String {
    include_str!("../../proof_of_state_v2/src/lib.rs")
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("//!")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// THE LEGACY DEFECT THIS FORBIDS: `canisters/proof_of_state::anchor()`
/// hard-coded `let btc_canister_id = "uxrrr-q7777-77774-qaaaq-cai";` — a
/// LOCAL dfx id that resolves `canister_not_found` on the IC, which is why
/// every "anchored" batch actually recorded a mock txid. The signer
/// principal MUST come from governed configuration, never source code.
#[test]
fn p_no_hardcoded_canister_principal_in_proof_of_state_v2() {
    let src = canister_src_without_comments();
    assert!(
        !src.contains("uxrrr-q7777-77774-qaaaq-cai"),
        "the legacy local-dfx canister id must never reappear"
    );
    // No OTHER literal canister-id-shaped string may be passed to
    // `ic_cdk::call` either — the signer principal must flow from `cfg.
    // anchor_signer_principal`, read out of CONFIG.
    let call_idx = src.find("ic_cdk::call(").expect("request_anchor must call out to the signer");
    let arg_window = &src[call_idx..(call_idx + 200).min(src.len())];
    assert!(
        arg_window.contains("cfg.anchor_signer_principal"),
        "the inter-canister call's target must be cfg.anchor_signer_principal, read from governed \
         config — not a literal Principal"
    );
}

/// THE LEGACY DEFECT THIS FORBIDS: `anchor()`'s failure branch returned
/// `Ok(format!("mock_btc_txid_{}", &batch.root[..8]))` — a SUCCESS
/// synthesised from the function's own input, indistinguishable from a real
/// anchor to any caller that didn't read the source.
#[test]
fn p_no_synthetic_txid_fallback_in_proof_of_state_v2() {
    let src = canister_src_without_comments();
    assert!(!src.contains("mock_btc_txid"), "no mock txid literal/prefix may reappear");
    assert!(
        !src.contains(r#"Ok(format!("btc_anchor_"#),
        "no synthesised-success txid may be constructed from the function's own input"
    );
}

/// THE LEGACY DEFECT THIS FORBIDS: `batch.btc_block_height = Some(800000);`
/// — a literal, never updated from reality, standing in for confirmation
/// evidence that was never actually gathered.
#[test]
fn p_no_hardcoded_block_height_in_proof_of_state_v2() {
    let src = canister_src_without_comments();
    assert!(!src.contains("800000"), "no hard-coded block height literal may reappear");
    let sig_idx = src.find("pub fn record_confirmation(").expect("record_confirmation must exist");
    let sig_window = &src[sig_idx..(sig_idx + 300).min(src.len())];
    assert!(
        sig_window.contains("block_height: u64") && sig_window.contains("confirmations: u32"),
        "block_height and confirmations must be PARAMETERS supplied by confirmation evidence, \
         never fields assigned from a literal"
    );
}

/// `issue_receipt` MUST look up the existing receipt (its idempotency key)
/// BEFORE deciding the outcome — never construct a new receipt first and
/// discover the collision after.
#[test]
fn p_issue_receipt_looks_up_existing_before_deciding_in_proof_of_state_v2() {
    let src = canister_src_without_comments();
    let fn_idx = src.find("pub fn issue_receipt(").expect("issue_receipt must exist");
    let body = &src[fn_idx..];
    let lookup_idx = body.find("RECEIPTS.with(|r| r.borrow().get(").expect("must look up the existing receipt");
    let decide_idx = body.find("decide_issue_receipt(").expect("must call the pure idempotency decision");
    assert!(
        lookup_idx < decide_idx,
        "the H-keyed lookup must happen BEFORE decide_issue_receipt is called, so the idempotency \
         check sees the TRUE prior state rather than being told about it after the fact"
    );
}

/// `request_anchor` (broadcast) and `record_confirmation` (anchored) MUST
/// be separate entry points, and neither may reach into the other's
/// decision function — the legacy `get_anchor_status()` conflated the two
/// concepts inside ONE code path (`btc_anchor_txid.is_some()` alone meant
/// "confirmed").
#[test]
fn p_request_anchor_and_record_confirmation_are_separate_and_do_not_cross_call() {
    let src = canister_src_without_comments();
    let request_idx = src.find("pub async fn request_anchor(").expect("request_anchor must exist");
    let confirm_idx = src.find("pub fn record_confirmation(").expect("record_confirmation must exist");
    assert_ne!(request_idx, confirm_idx, "the two entry points must be distinct functions");

    let (first_idx, second_idx) = if request_idx < confirm_idx { (request_idx, confirm_idx) } else { (confirm_idx, request_idx) };
    let first_body = &src[first_idx..second_idx];
    let second_body = &src[second_idx..];

    let request_body = if request_idx < confirm_idx { first_body } else { second_body };
    let confirm_body = if confirm_idx < request_idx { first_body } else { second_body };

    assert!(
        !request_body.contains("decide_confirmation("),
        "request_anchor must never call decide_confirmation — reaching Anchored belongs solely to \
         record_confirmation"
    );
    assert!(
        !confirm_body.contains("decide_anchor_request("),
        "record_confirmation must never call decide_anchor_request — requesting a broadcast \
         belongs solely to request_anchor"
    );
}

/// `record_confirmation` MUST authorize the caller BEFORE it reads or
/// writes any batch state — the same "caller captured/checked first"
/// discipline `btc_signer_psbt::authorize_anchor_caller` follows.
#[test]
fn p_record_confirmation_authorizes_before_touching_batch_state() {
    let src = canister_src_without_comments();
    let fn_idx = src.find("pub fn record_confirmation(").expect("record_confirmation must exist");
    let body = &src[fn_idx..];
    let auth_idx = body
        .find("cfg.authorized_reconciler_principal")
        .expect("must check the caller against the authorized reconciler");
    let batches_idx = body.find("BATCHES.with(").expect("must read the batch at some point");
    assert!(
        auth_idx < batches_idx,
        "the authorization check must happen BEFORE any batch state is read or written"
    );
}

/// THE LEGACY DEFECT THIS FORBIDS: `canisters/proof_of_state` has no
/// `#[pre_upgrade]`/`#[post_upgrade]` at all — every receipt and batch it
/// ever recorded lives only in a `thread_local!`, discarded silently on
/// upgrade. `proof_of_state_v2` must persist ALL FOUR pieces of state.
#[test]
fn p_stable_persistence_covers_every_piece_of_state_in_proof_of_state_v2() {
    let src = canister_src_without_comments();
    let pre_idx = src.find("#[pre_upgrade]").expect("pre_upgrade must exist");
    let post_idx = src.find("#[post_upgrade]").expect("post_upgrade must exist");
    let pre_body = &src[pre_idx..post_idx];
    let post_body = &src[post_idx..];

    assert!(pre_body.contains("ic_cdk::storage::stable_save"), "pre_upgrade must call stable_save");
    assert!(post_body.contains("ic_cdk::storage::stable_restore"), "post_upgrade must call stable_restore");

    for marker in ["RECEIPTS", "PENDING", "BATCHES", "CONFIG"] {
        assert!(
            pre_body.contains(marker),
            "pre_upgrade must persist {marker} — heap-only state that an upgrade silently \
             discards is exactly the legacy canister's defect"
        );
        assert!(post_body.contains(marker), "post_upgrade must restore {marker}");
    }
}
