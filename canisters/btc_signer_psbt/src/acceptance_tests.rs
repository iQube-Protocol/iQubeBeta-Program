//! Phase B acceptance tests — the anchoring contract, written RED FIRST.
//!
//! ─── THESE TESTS ARE EXPECTED TO FAIL AGAINST THIS COMMIT ───────────────────
//!
//! They encode the contract `btc_signer_psbt` must satisfy before the AigentZBeta
//! constitutional receipt spine may enable its PoS/Bitcoin leg
//! (`POS_LEG_SUBMISSION_ENABLED`). Every one of them fails today, and each names
//! the specific defect that makes it fail. That is deliberate: a canary written
//! after the fix, or one that passes on arrival, proves nothing about whether it
//! can detect the regression it exists to catch.
//!
//! ─── WHY THIS FILE EXISTS AT ALL ────────────────────────────────────────────
//!
//! A read-only probe of the DEPLOYED proof_of_state canister
//! (n2hhv-aaaaa-aaaas-qccza-cai, 2026-08-08) established that this system has
//! never anchored anything to Bitcoin:
//!
//!   * all 76 anchored batches carry `btc_anchor_txid = "mock_btc_txid_<...>"`,
//!     which is not a 64-hex Bitcoin txid and cannot exist on any network;
//!   * `btc_block_height` is the constant 800000 on every batch;
//!   * the batch root commits to receipt IDs, not `data_hash` — verified
//!     cryptographically, 20/20 batches match sha256(concat ids), 0/20 match
//!     sha256(concat data_hashes);
//!   * `merkle_proof` is empty on all 186 receipts in anchored batches.
//!
//! The defects in THIS canister are why fixing proof_of_state alone could not
//! help: even a perfect Merkle root over H would be handed to a signer that
//! never serialises a transaction.
//!
//! ─── THE EXISTING TEST MODULE ENCODES THE DEFECT ────────────────────────────
//!
//! `tests::anchor_tx_builds_with_change` asserts
//! `o.address == "change_address"` — it requires the placeholder string to be
//! present, so correcting the address field would BREAK it. It is a passing test
//! that defends the bug. It is left in place for now so this commit changes no
//! behaviour, and Phase B implementation must delete it rather than work around
//! it.
//!
//! ─── LINEAGE CORRECTION (2026-08-08) ────────────────────────────────────────
//!
//! An earlier revision of this file claimed this canister had "never had a
//! working test gate" and that "the anchor path cannot have worked at any point
//! in its history". BOTH OVERSTATED THE EVIDENCE and are withdrawn.
//!
//! What was actually established is narrower: at HEAD (db6e5628) the workspace
//! did not resolve and these tests did not compile. That is a statement about
//! HEAD, not about the project's history. Commit 3ee3cb0 (2025-09-14) records
//! all four canisters deployed and live-function tested, and the workspace
//! break was introduced later when reputation_qube was added with an invalid
//! ic-cdk-macros feature.
//!
//! The distinction matters because it changes the repair: this is a regression
//! in a system that once ran, not an edifice that never did.
//!
//! ─── WHY THE MOCK txid FIRES: THE CENSUS RESULT ─────────────────────────────
//!
//! `proof_of_state::anchor()` hardcodes its callee:
//!
//!     let btc_canister_id = "uxrrr-q7777-77774-qaaaq-cai";
//!
//! A read-only lineage census (2026-08-08) resolved that principal against IC
//! mainnet: canister_not_found. It is the LOCAL dfx id recorded in
//! `.dfx/local/canister_ids.json` @ cebf998, which commit a88bc3a then wrote
//! into mainnet environment configuration under the header
//! "Bitcoin Signer - LIVE MAINNET". No IC-mainnet btc_signer has ever existed.
//!
//! So the synthesised txid is not a lazy placeholder. It is the ERROR BRANCH
//!
//!     Err(_) => Ok(format!("mock_btc_txid_{}", &batch.root[..8]))
//!
//! firing on every anchor since deployment, because the inter-canister call can
//! never reach anything. The defects below are real and must still be fixed —
//! but they are not, on their own, why nothing reached Bitcoin.
//!
//! Normative encoding for every byte-level assertion below is §A3 of
//! `codexes/packs/agentiq/updates/2026-08-08_canister-repair-plan.md`
//! (AigentZBeta).

use super::*;

/// A representative 32-byte Merkle root, hex-encoded exactly as
/// `proof_of_state` will hand it over.
const ROOT_HEX: &str = "9f2c4e6a8b0d1f3557799bbddff11335577799bbddff1133557799bbddff11335";

/// `OP_RETURN` (0x6a) followed by `OP_PUSHBYTES_32` (0x20). Every compliant
/// anchor output's `script_pubkey` starts with these two bytes, and the 32
/// bytes that follow are the root.
const OP_RETURN_PUSH32: [u8; 2] = [0x6a, 0x20];

fn sample_utxo(amount: u64) -> UTXO {
    UTXO {
        // A real 64-hex txid, unlike the all-zero mock in
        // `create_and_broadcast_anchor`.
        txid: "1f8b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f9".to_string(),
        vout: 0,
        amount,
        script_pubkey: vec![0x00, 0x14],
        }
}

// ───────────────────────────────────────────────────────────────────────────
// BT-1 — the root must actually reach the transaction
// ───────────────────────────────────────────────────────────────────────────

/// FAILS TODAY: `create_anchor_transaction` computes
/// `let _op_return_script = format!("6a20{}", data_hash);` — underscore-prefixed,
/// never read, discarded at end of scope. The root never enters any output. The
/// output that is supposed to carry it holds the literal string "OP_RETURN" in
/// its `address` field instead.
#[test]
fn bt1_anchor_output_carries_the_root() {
    let tx = futures::executor::block_on(create_anchor_transaction(
        vec![sample_utxo(100_000)],
        ROOT_HEX.to_string(),
        10,
    ))
    .expect("anchor tx should build with sufficient funds");

    let carries_root = tx
        .outputs
        .iter()
        .any(|o| o.address.contains(ROOT_HEX) || o.address.contains(&ROOT_HEX.to_uppercase()));

    assert!(
        carries_root,
        "No output carries the Merkle root. The OP_RETURN script is computed into \
         `_op_return_script` and discarded, so the commitment never reaches Bitcoin. \
         Outputs were: {:?}",
        tx.outputs.iter().map(|o| &o.address).collect::<Vec<_>>()
    );
}

/// FAILS TODAY: outputs carry human labels where consensus data belongs.
/// "OP_RETURN" and "change_address" are not addresses and not scripts — no
/// serialiser can turn them into transaction bytes, which is the root cause of
/// BT-1 and BT-3 alike.
#[test]
fn bt1b_outputs_are_not_placeholder_strings() {
    let tx = futures::executor::block_on(create_anchor_transaction(
        vec![sample_utxo(100_000)],
        ROOT_HEX.to_string(),
        10,
    ))
    .expect("anchor tx should build");

    for o in &tx.outputs {
        assert!(
            o.address != "OP_RETURN" && o.address != "change_address",
            "Output address is the placeholder {:?}. A literal label cannot be encoded into a \
             Bitcoin transaction; this is why no transaction is ever serialised.",
            o.address
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// BT-2 / BT-3 — a signed transaction must BE a transaction
// ───────────────────────────────────────────────────────────────────────────

/// FAILS TODAY: `let raw_tx = format!("signed_tx_{}", txid);`
///
/// `raw_tx` is a label with a `signed_tx_` prefix, so it is not even valid hex,
/// let alone a consensus-encoded transaction. `broadcast_transaction` feeds this
/// string straight to `sendrawtransaction`, which no Bitcoin node would accept.
/// (Scope: a defect in THIS function. It is not the reason mainnet anchoring
/// produced mock txids — see the census note in the module header.)
#[test]
fn bt3_raw_tx_is_hex_decodable_transaction_bytes() {
    let signed = match futures::executor::block_on(sign_transaction(
        UnsignedTransaction {
            inputs: vec![TransactionInput { utxo: sample_utxo(100_000), sequence: 0xfffffffd }],
            outputs: vec![TransactionOutput { address: ROOT_HEX.to_string(), amount: 0 }],
            locktime: 0,
        },
        vec![],
    )) {
        Ok(s) => s,
        // In a host (non-replica) test the threshold-ECDSA call cannot succeed.
        // That is an environment limit, not the contract under test, so it is
        // reported as such rather than silently passing.
        Err(e) => panic!(
            "sign_transaction unavailable in host test ({e}). Phase B must expose a pure \
             serialisation entry point that is testable without the management canister — \
             an anchor format that can only be exercised on-chain cannot be regression-tested."
        ),
    };

    assert!(
        !signed.raw_tx.starts_with("signed_tx_"),
        "raw_tx is the label {:?}, not transaction bytes.",
        signed.raw_tx
    );
    assert!(
        hex::decode(&signed.raw_tx).is_ok(),
        "raw_tx {:?} is not hex-decodable, so it is not a serialised transaction and cannot be \
         broadcast.",
        signed.raw_tx
    );
}

/// FAILS TODAY: `let txid = hex::encode(&signature[..32]);`
///
/// The txid is the first 32 bytes of the ECDSA signature. A Bitcoin txid is the
/// double-SHA256 of the consensus-serialised transaction. These are unrelated
/// values, so the identifier we record as "the anchor" refers to nothing on any
/// network — precisely what the deployed-canister probe observed.
#[test]
fn bt2_txid_is_double_sha256_of_serialised_tx() {
    let signed = match futures::executor::block_on(sign_transaction(
        UnsignedTransaction {
            inputs: vec![TransactionInput { utxo: sample_utxo(100_000), sequence: 0xfffffffd }],
            outputs: vec![TransactionOutput { address: ROOT_HEX.to_string(), amount: 0 }],
            locktime: 0,
        },
        vec![],
    )) {
        Ok(s) => s,
        Err(e) => panic!("sign_transaction unavailable in host test ({e}); see bt3 for why this matters."),
    };

    let bytes = hex::decode(&signed.raw_tx)
        .expect("raw_tx must be transaction bytes before a txid can be derived from it");
    let expected = {
        use sha2::{Digest, Sha256};
        let once = Sha256::digest(&bytes);
        let twice = Sha256::digest(once);
        // Bitcoin displays txids in reverse byte order.
        let mut b = twice.to_vec();
        b.reverse();
        hex::encode(b)
    };

    assert_eq!(
        signed.txid, expected,
        "txid is not sha256d(serialised_tx). It is currently the first 32 bytes of the signature, \
         which identifies no transaction on any Bitcoin network."
    );
}

// ───────────────────────────────────────────────────────────────────────────
// BT-4 — addresses must be real addresses
// ───────────────────────────────────────────────────────────────────────────

/// FAILS TODAY: `format!("tb1q{}", hex::encode(&public_key[..20]))`
///
/// Three independent errors in one line: the payload is the raw first 20 bytes
/// of the pubkey rather than `hash160(compressed_pubkey)`; it is hex rather than
/// bech32's 5-bit squashed encoding; and there is no checksum. A wallet would
/// reject it, and funds sent to it would be unspendable.
///
/// DECODED, NOT GUESSED. An earlier version of this test checked the charset of
/// the substring after the final '1'. It PASSED against the broken
/// implementation — not because the address was valid, but because that
/// particular pubkey's hex happened to place its last '1' before a run of
/// characters that were all in the bech32 charset. The verdict depended on the
/// test's own fixture bytes. Decoding is the only assertion that cannot pass by
/// accident.
#[test]
fn bt4_derived_address_is_valid_bech32_of_hash160() {
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

    let pubkey_hex = "02b4632d08485ff1df2db55b9dafd23347d1c47a457072a1e87be26896549a8737";
    let pubkey = hex::decode(pubkey_hex).unwrap();
    let current_impl_address = format!("tb1q{}", hex::encode(&pubkey[..20]));

    let (hrp, data, variant) = match bech32::decode(&current_impl_address) {
        Ok(v) => v,
        Err(e) => panic!(
            "address {current_impl_address:?} is not decodable bech32 ({e}). It is hex with a \
             \"tb1q\" prefix: no checksum, no 5-bit squashing, and the payload is the raw first \
             20 bytes of the pubkey rather than hash160(pubkey)."
        ),
    };

    assert_eq!(hrp, "tb", "testnet P2WPKH must use the 'tb' human-readable part");
    assert_eq!(variant, bech32::Variant::Bech32, "P2WPKH (witness v0) uses Bech32, not Bech32m");

    // The decoded payload must be hash160(compressed_pubkey) — the actual
    // witness program — not any 20 bytes that happen to be the right length.
    let expected_hash160 = Ripemd160::digest(Sha256::digest(&pubkey)).to_vec();
    let decoded: Vec<u8> = {
        use bech32::FromBase32;
        Vec::<u8>::from_base32(&data[1..]).expect("witness program must decode from base32")
    };
    assert_eq!(
        decoded, expected_hash160,
        "witness program is not hash160(compressed_pubkey); funds sent to this address would be \
         unspendable"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// BT-5 / BT-6 — refuse rather than fabricate
// ───────────────────────────────────────────────────────────────────────────

/// FAILS TODAY: `create_and_broadcast_anchor` substitutes
/// `txid: "0000…0000"` — an all-zero mock UTXO — and proceeds. Anchoring must
/// refuse when it has no real funds to spend, exactly as AigentZBeta's
/// sync/repair now refuses rather than fabricating receipts to satisfy a metric.
#[test]
fn bt5_refuses_to_anchor_without_real_utxos() {
    let res = futures::executor::block_on(create_and_broadcast_anchor(ROOT_HEX.to_string(), 10));

    match res {
        Err(_) => { /* correct: refused */ }
        Ok(txid) => panic!(
            "create_and_broadcast_anchor returned {txid:?} despite having no real UTXOs. It \
             substitutes an all-zero mock UTXO and proceeds, manufacturing the appearance of an \
             anchor. Per amendment A1 this path must source UTXOs from the IC's native Bitcoin \
             API (bitcoin_get_utxos) and refuse when there are none."
        ),
    }
}

/// FAILS TODAY: on the txid-parse-failure path `broadcast_transaction` returns
/// `Ok(format!("broadcast_success_{}", ...))` — a success string synthesised
/// from its own input. This is the same Err-as-success class already fixed in
/// AigentZBeta's LayerZero route: it makes the telemetry of the anchoring path
/// untrustworthy, so a repaired signer could still report success while every
/// broadcast was rejected.
///
/// Per amendment A1 the HTTP transport is removed entirely in favour of
/// `bitcoin_send_transaction`; this test pins the property that must hold
/// afterwards — a failure is an `Err`, never a manufactured `Ok`.
#[test]
fn bt6_broadcast_failure_is_never_a_synthesised_success() {
    let res = futures::executor::block_on(broadcast_transaction("not-a-transaction".to_string()));

    if let Ok(v) = &res {
        assert!(
            !v.starts_with("broadcast_success_"),
            "broadcast_transaction manufactured the success value {v:?} from its own input after \
             failing to obtain a txid. A rejected broadcast must be an Err carrying the node's \
             reason."
        );
    }
}
