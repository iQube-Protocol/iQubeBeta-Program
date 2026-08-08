//! Phase B acceptance tests — the Bitcoin anchoring contract.
//!
//! ─── PROVENANCE OF THESE ASSERTIONS ─────────────────────────────────────────
//!
//! Written RED FIRST against `db6e5628` (commit "Phase B acceptance tests
//! (RED)"), where all seven failed. They are now GREEN against Constitutional
//! Anchor v2. Each test still names the defect it was written to catch, so the
//! contract remains readable as a record of what went wrong.
//!
//! WHERE AN ASSERTION CHANGED SHAPE, IT GOT STRICTER — never looser:
//!
//!   * BT-1 asked "does the root hex appear somewhere in an output's `address`
//!     string?". It now requires the output's `script_pubkey` to be EXACTLY
//!     `6a20 ‖ root_bytes`. The old form would have accepted the root sitting
//!     in a label; the new one accepts only a real commitment script.
//!   * BT-2/BT-3 previously called `sign_transaction`, which cannot run on the
//!     host, and panicked with a message DEMANDING a pure entry point
//!     ("an anchor format that can only be exercised on-chain cannot be
//!     regression-tested"). v2 provides one, so they now exercise the real
//!     serialiser and check the bytes rather than reporting an environment
//!     limit.
//!
//! ─── WHAT THE CENSUS ESTABLISHED, AND WHY IT MATTERS HERE ───────────────────
//!
//! `proof_of_state::anchor()` hard-coded `btc_canister_id =
//! "uxrrr-q7777-77774-qaaaq-cai"`. A read-only lineage census (2026-08-08)
//! resolved that principal against IC mainnet: `canister_not_found`. It is the
//! LOCAL dfx id from `.dfx/local/canister_ids.json` @ cebf998, written into
//! mainnet env config at a88bc3a under the header "Bitcoin Signer - LIVE
//! MAINNET". No IC-mainnet BTC signer has ever existed.
//!
//! So the `mock_btc_txid_*` on all 76 "anchored" batches is the ERROR BRANCH
//! firing since deployment, not a lazy placeholder. The defects below were real
//! and are fixed — but they were not, on their own, why nothing reached
//! Bitcoin. "v2" therefore names an architecture generation, not a redeploy.
//!
//! LINEAGE CORRECTION: earlier revisions claimed this canister "never had a
//! working test gate" and that the anchor path "cannot have worked at any point
//! in its history". Both overstated the evidence and are withdrawn — 3ee3cb0
//! (2025-09-14) records all four canisters deployed and live-function tested.
//! What was established is that HEAD did not build.
//!
//! Normative byte encoding: §A3 of AigentZBeta's
//! `codexes/packs/agentiq/updates/2026-08-08_canister-repair-plan.md`.

use super::*;

/// A representative 32-byte Merkle root, hex-encoded as `proof_of_state` hands
/// it over. 64 chars ⇒ 32 bytes.
const ROOT_HEX: &str = "9f2c4e6a8b0d1f3557799bbddff11335577799bbddff1133557799bbddff1133";

/// `OP_RETURN` (0x6a) then `OP_PUSHBYTES_32` (0x20).
const OP_RETURN_PUSH32: [u8; 2] = [0x6a, 0x20];

/// A real compressed secp256k1 public key (33 bytes, 0x02 prefix).
const PUBKEY_HEX: &str = "02b4632d08485ff1df2db55b9dafd23347d1c47a457072a1e87be26896549a8737";

fn pubkey() -> Vec<u8> {
    hex::decode(PUBKEY_HEX).unwrap()
}

fn sample_tx() -> Tx {
    let h160 = hash160(&pubkey());
    Tx {
        version: 2,
        inputs: vec![TxIn {
            prev_txid_hex: "1f8b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f9".to_string(),
            vout: 0,
            value: 100_000,
            sequence: 0xffff_fffd,
        }],
        outputs: vec![
            TxOut { value: 0, script_pubkey: op_return_script(ROOT_HEX).unwrap() },
            TxOut { value: 98_000, script_pubkey: p2wpkh_script(&h160) },
        ],
        locktime: 0,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// BT-1 — the root must actually reach the transaction
// ───────────────────────────────────────────────────────────────────────────

/// WAS RED because `create_anchor_transaction` computed
/// `let _op_return_script = format!("6a20{}", data_hash);` — underscore-
/// prefixed, never read, discarded at end of scope — so the root never entered
/// any output.
#[test]
fn bt1_anchor_output_carries_the_root() {
    let script = op_return_script(ROOT_HEX).expect("root must encode");
    assert_eq!(&script[..2], &OP_RETURN_PUSH32, "script must open OP_RETURN OP_PUSHBYTES_32");
    assert_eq!(script.len(), 34, "OP_RETURN + push opcode + 32 bytes");
    assert_eq!(
        hex::encode(&script[2..]),
        ROOT_HEX,
        "the pushed bytes must BE the root — decoded to 32 raw bytes, never the ASCII hex, which \
         would be 64 bytes and a different commitment (§A3)"
    );

    // And the root must survive into a serialised transaction.
    let raw = serialize_tx_no_witness(&sample_tx()).expect("tx must serialise");
    assert!(
        raw.windows(34).any(|w| w == script.as_slice()),
        "the commitment script does not appear in the serialised transaction bytes"
    );
}

/// WAS RED because outputs carried the literal strings "OP_RETURN" and
/// "change_address" in their `address` field — labels no serialiser could turn
/// into transaction bytes.
#[test]
fn bt1b_outputs_are_not_placeholder_strings() {
    for o in &sample_tx().outputs {
        let rendered = hex::encode(&o.script_pubkey);
        assert!(
            rendered != hex::encode(b"OP_RETURN") && rendered != hex::encode(b"change_address"),
            "output script is a placeholder label, not consensus bytes"
        );
        assert!(!o.script_pubkey.is_empty(), "every output must carry a real script_pubkey");
    }
    // Output 0 is the commitment; output 1 is spendable change (P2WPKH).
    let outs = sample_tx().outputs;
    assert_eq!(outs[0].script_pubkey[0], 0x6a, "output 0 must be the OP_RETURN commitment");
    assert_eq!(outs[1].script_pubkey[0], 0x00, "output 1 must be a witness-v0 program");
    assert_eq!(outs[1].script_pubkey.len(), 22, "P2WPKH scriptPubKey is 22 bytes");
}

// ───────────────────────────────────────────────────────────────────────────
// BT-2 / BT-3 — a signed transaction must BE a transaction
// ───────────────────────────────────────────────────────────────────────────

/// WAS RED because `raw_tx = format!("signed_tx_{}", txid)` — a label with a
/// prefix, not even valid hex.
#[test]
fn bt3_raw_tx_is_hex_decodable_transaction_bytes() {
    let sig = vec![0x42u8; 64]; // a well-formed compact signature for shape purposes
    let (_txid, raw_tx) = assemble_signed(&sample_tx(), &[sig], &pubkey()).expect("must assemble");

    assert!(!raw_tx.starts_with("signed_tx_"), "raw_tx is a label, not bytes");
    let bytes = hex::decode(&raw_tx).expect("raw_tx must be hex-decodable transaction bytes");

    // Structural checks: segwit marker+flag, and a plausible size.
    assert_eq!(&bytes[4..6], &[0x00, 0x01], "segwit marker and flag must be present");
    assert!(bytes.len() > 100, "a one-input two-output segwit tx is well over 100 bytes");
    // The witness must carry [DER signature, pubkey] — not an empty stack.
    assert!(
        bytes.windows(33).any(|w| w == pubkey().as_slice()),
        "the witness must include the compressed public key"
    );
}

/// WAS RED because `txid = hex::encode(&signature[..32])` — the first 32 bytes
/// of the ECDSA signature, which identifies no transaction on any network.
#[test]
fn bt2_txid_is_double_sha256_of_serialised_tx() {
    let tx = sample_tx();
    let sig = vec![0x42u8; 64];
    let (txid, _raw) = assemble_signed(&tx, &[sig], &pubkey()).expect("must assemble");

    // Recompute independently from the witness-free serialisation.
    let no_witness = serialize_tx_no_witness(&tx).unwrap();
    let expected = {
        let mut h = sha256d(&no_witness);
        h.reverse(); // Bitcoin displays txids in reverse byte order
        hex::encode(h)
    };
    assert_eq!(txid, expected, "txid must be sha256d of the witness-free serialisation");
    assert!(txid.len() == 64 && txid.chars().all(|c| c.is_ascii_hexdigit()), "txid must be 64 hex chars");

    // SEGWIT INVARIANT: the txid must not depend on the witness. If it did,
    // a third party could malleate our anchor's identity after broadcast.
    let (txid_other, _) = assemble_signed(&tx, &[vec![0x07u8; 64]], &pubkey()).unwrap();
    assert_eq!(txid, txid_other, "a different witness must not change the txid");
}

// ───────────────────────────────────────────────────────────────────────────
// BT-4 — addresses must be real addresses
// ───────────────────────────────────────────────────────────────────────────

/// WAS RED because `format!("tb1q{}", hex::encode(&public_key[..20]))` is not
/// bech32: no checksum, no 5-bit squashing, and it encodes the raw pubkey
/// prefix rather than `hash160(compressed_pubkey)`. Funds sent there would be
/// unspendable.
///
/// DECODED, NOT GUESSED. An earlier version of this test checked the charset of
/// the substring after the final '1' and PASSED against the broken address —
/// the verdict depended on where that '1' fell in the fixture's hex, not on
/// validity. Decoding cannot pass by accident.
#[test]
fn bt4_derived_address_is_valid_bech32_of_hash160() {
    let addr = p2wpkh_address(&pubkey(), BtcNetwork::Testnet).expect("address must derive");

    let (hrp, data, variant) = bech32::decode(&addr).expect("address must be decodable bech32");
    assert_eq!(hrp, "tb", "testnet P2WPKH uses the 'tb' human-readable part");
    assert_eq!(variant, bech32::Variant::Bech32, "witness v0 uses Bech32, not Bech32m");
    assert_eq!(data[0].to_u8(), 0, "witness version must be 0");

    let program = Vec::<u8>::from_base32(&data[1..]).expect("witness program must decode");
    assert_eq!(
        program,
        hash160(&pubkey()).to_vec(),
        "the witness program must be hash160(compressed_pubkey), not the raw pubkey prefix"
    );

    // Network selection must be explicit, not a build flag.
    let main = p2wpkh_address(&pubkey(), BtcNetwork::Mainnet).unwrap();
    assert!(main.starts_with("bc1"), "mainnet addresses use the 'bc' hrp");
    assert_ne!(main, addr, "mainnet and testnet addresses must differ");

    // An uncompressed key hashes differently and must be refused, not silently
    // used — it would yield an address we cannot spend from.
    assert!(p2wpkh_address(&[0x04u8; 65], BtcNetwork::Testnet).is_err());
}

use bech32::FromBase32;

// ───────────────────────────────────────────────────────────────────────────
// BT-5 / BT-6 — refuse rather than fabricate
// ───────────────────────────────────────────────────────────────────────────

/// WAS RED because `create_and_broadcast_anchor` substituted a UTXO with an
/// all-zero txid and proceeded, manufacturing the appearance of an anchor.
///
/// v2 refuses at the first opportunity. Exercised here through
/// `create_anchor_transaction`, which is the point where "nothing to spend"
/// becomes knowable without a replica; `create_and_broadcast_anchor` applies
/// the same refusal after `bitcoin_get_utxos` returns empty.
#[test]
fn bt5_refuses_to_anchor_without_real_utxos() {
    let err = plan_anchor(&[], ROOT_HEX, 10)
        .err()
        .expect("must refuse with no UTXOs rather than substituting a placeholder");
    assert!(err.contains("No UTXOs"), "refusal must name the reason: {err}");
    assert!(err.contains("manufacture"), "refusal should say what fabricating would cost");

    let input = |v: u64| AnchorInput {
        txid_hex: "1f8b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f9".to_string(),
        vout: 0,
        value: v,
    };
    // Cannot cover the fee — refuse rather than wrap or emit negative change.
    assert!(plan_anchor(&[input(10)], ROOT_HEX, 1000).is_err());
    // Change below dust — refuse rather than create an unspendable output.
    let plan_dusty = plan_anchor(&[input(11 + 68 + 43 + 31 + 100)], ROOT_HEX, 1);
    assert!(plan_dusty.is_err(), "must refuse when change would be dust");
    // A genuinely fundable anchor plans successfully.
    let plan = plan_anchor(&[input(100_000)], ROOT_HEX, 2).expect("should plan with real funds");
    assert_eq!(plan.op_return.len(), 34);
    assert!(plan.change_value > P2WPKH_DUST_SAT);
    assert_eq!(plan.fee, 2 * plan.estimated_vsize);
    // An invalid root is refused BEFORE any money arithmetic.
    assert!(plan_anchor(&[input(100_000)], "not-a-root", 2).is_err());
}

/// WAS RED because the HTTP broadcast path could return
/// `Ok(format!("broadcast_success_{…}"))` — a success value synthesised from
/// its own input — after failing to parse a txid.
///
/// Per amendment A1 that HTTP transport is REMOVED, not repaired; v2 uses
/// `bitcoin_send_transaction`. This pins the property that must hold: a failure
/// is an `Err`, never a manufactured `Ok`.
#[test]
fn bt6_broadcast_failure_is_never_a_synthesised_success() {
    match validate_raw_tx_hex("not-a-transaction") {
        Err(e) => assert!(e.contains("not hex"), "the refusal must name the real reason: {e}"),
        Ok(_) => panic!(
            "validation accepted input that is not a transaction. A synthesised success makes \
             every downstream anchoring claim untrustworthy."
        ),
    }
    assert!(validate_raw_tx_hex("").is_err(), "empty input must be refused");
    // The predecessor's own output shape must be refused by name.
    let e = validate_raw_tx_hex("signed_tx_deadbeef").unwrap_err();
    assert!(e.contains("signed_tx_"), "the historical label must be refused explicitly: {e}");
    // Valid hex that is too short is still not a transaction.
    assert!(validate_raw_tx_hex("deadbeef").is_err());
    // A real assembled transaction passes.
    let (_t, raw) = assemble_signed(&sample_tx(), &[vec![0x42u8; 64]], &pubkey()).unwrap();
    assert!(validate_raw_tx_hex(&raw).is_ok(), "a genuine serialised transaction must validate");
}

// ───────────────────────────────────────────────────────────────────────────
// The enduring invariant (operator directive, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────

/// Production canister principals must never be hard-coded into dependent
/// canister source. This is the defect that made every anchor fail silently:
/// `proof_of_state` embedded a local dfx principal as its signer, so the call
/// could not succeed in the environment it was deployed to, and the caller's
/// error branch reported that failure as an anchor.
#[test]
fn no_hardcoded_canister_principals_in_anchor_v2() {
    // Scans BOTH crates, and the canister crate is the one that matters: it is
    // the dependent canister whose predecessor embedded a peer's principal.
    let src = concat!(
        include_str!("lib.rs"),
        "\n",
        include_str!("../../btc_signer_psbt/src/lib.rs"),
    );
    // Strip the doc comments, which legitimately quote the offending literal
    // while explaining why it must never appear as code.
    let code: String = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("//!")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let principal_literal = regex_lite_find_principal(&code);
    assert!(
        principal_literal.is_none(),
        "a canister principal is hard-coded in this canister's executable source: {principal_literal:?}. \
         Supply cross-canister callees through governed configuration instead."
    );
}

/// Minimal scan for an IC principal literal (`xxxxx-...-cai`) inside a string
/// literal. Deliberately dependency-free — a canary that needs a crate to run
/// is a canary that gets removed when the crate is inconvenient.
fn regex_lite_find_principal(code: &str) -> Option<String> {
    for raw in code.split('"').skip(1).step_by(2) {
        let s = raw.trim();
        if s.ends_with("-cai") && s.matches('-').count() >= 4 && s.len() >= 25 {
            return Some(s.to_string());
        }
    }
    None
}

// ───────────────────────────────────────────────────────────────────────────
// P0.1 — the signing surface is closed (independent review, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────
//
// WAS RED: `sign_transaction`, `broadcast_transaction` and
// `create_anchor_transaction` were public `#[update]` methods with no caller
// check. Any principal on the IC could make this canister sign with its
// threshold key, spend its UTXOs, or broadcast arbitrary bytes.

fn cfg_with(principal: Option<&str>) -> AnchorConfig {
    AnchorConfig {
        network: BtcNetwork::Testnet,
        ecdsa_key_name: "test_key_1".to_string(),
        authorized_pos_principal: principal.map(|s| s.to_string()),
    }
}

const POS_PRINCIPAL: &str = "n2hhv-aaaaa-aaaas-qccza-cai";

#[test]
fn p01_unauthorized_principal_cannot_cause_signing_or_spending() {
    let cfg = cfg_with(Some(POS_PRINCIPAL));
    // A stranger — the shape of the attack this closes.
    let err = authorize_anchor_caller("sp5ye-2qaaa-aaaao-qkqla-cai", &cfg).unwrap_err();
    assert!(err.contains("not the authorized"), "refusal must name the mismatch: {err}");
    assert!(err.contains("refusing to sign"), "refusal must state what was refused: {err}");
}

#[test]
fn p01_anonymous_principal_is_never_authorized() {
    assert!(authorize_anchor_caller("2vxsx-fae", &cfg_with(Some(POS_PRINCIPAL))).is_err());
    // Even if someone configures it, the config itself is rejected.
    assert!(validate_anchor_config(&cfg_with(Some("2vxsx-fae"))).is_err());
}

#[test]
fn p01_unconfigured_canister_denies_everyone_rather_than_allowing_everyone() {
    // Fail-closed. The opposite default would make a misconfigured deployment
    // sign for whoever asked first.
    let err = authorize_anchor_caller(POS_PRINCIPAL, &cfg_with(None)).unwrap_err();
    assert!(err.contains("denies all anchoring requests"), "{err}");
}

#[test]
fn p01_the_configured_pos_principal_is_authorized() {
    assert!(authorize_anchor_caller(POS_PRINCIPAL, &cfg_with(Some(POS_PRINCIPAL))).is_ok());
}

#[test]
fn p01_mainnet_may_not_be_configured_with_a_test_ecdsa_key() {
    let mut cfg = cfg_with(Some(POS_PRINCIPAL));
    cfg.network = BtcNetwork::Mainnet;
    cfg.ecdsa_key_name = "test_key_1".to_string();
    let err = validate_anchor_config(&cfg).unwrap_err();
    assert!(err.contains("MAINNET"), "{err}");
    assert!(err.contains("unrecoverable"), "the refusal must say what is at stake: {err}");

    cfg.ecdsa_key_name = "key_1".to_string();
    assert!(validate_anchor_config(&cfg).is_ok(), "a production key on mainnet is permitted");

    // An empty key name has no safe default.
    cfg.ecdsa_key_name = String::new();
    assert!(validate_anchor_config(&cfg).is_err());
}

// ───────────────────────────────────────────────────────────────────────────
// INDEPENDENT ORACLE (independent review, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────
//
// Every test above validates our serialiser against our own understanding of
// the format. That is circular: a consistent misreading of the spec would pass
// all of them. `rust-bitcoin` is an independent implementation maintained by
// people who never saw this code, so making IT parse and agree is evidence of
// a different kind.
//
// This is the same discipline as CAP-1 one layer down: the question is not
// "does our code accept our output" but "does something we did not write
// recognise it".

#[test]
fn oracle_rust_bitcoin_parses_our_transaction_and_agrees_on_the_txid() {
    use bitcoin::consensus::Decodable;

    let tx = sample_tx();
    let (our_txid, raw_hex) = assemble_signed(&tx, &[vec![0x42u8; 64]], &pubkey()).unwrap();
    let raw = hex::decode(&raw_hex).unwrap();

    let parsed = bitcoin::Transaction::consensus_decode(&mut raw.as_slice())
        .expect("rust-bitcoin must be able to parse our serialised transaction");

    // Structure agrees.
    assert_eq!(parsed.input.len(), tx.inputs.len(), "input count");
    assert_eq!(parsed.output.len(), tx.outputs.len(), "output count");
    assert_eq!(parsed.version.0, 2, "version");
    assert_eq!(parsed.lock_time.to_consensus_u32(), 0, "locktime");

    // THE TXID. Computed by an implementation that shares none of our code.
    assert_eq!(
        parsed.compute_txid().to_string(),
        our_txid,
        "rust-bitcoin derives a different txid from the same bytes — our txid derivation is wrong"
    );

    // THE COMMITMENT. rust-bitcoin must independently classify output 0 as a
    // valid OP_RETURN carrying exactly our 32-byte root.
    let commitment = &parsed.output[0];
    assert!(commitment.script_pubkey.is_op_return(), "output 0 must be recognised as OP_RETURN");
    assert_eq!(commitment.value.to_sat(), 0, "an OP_RETURN output carries no value");
    let pushed: Vec<u8> = commitment.script_pubkey.as_bytes()[2..].to_vec();
    assert_eq!(hex::encode(&pushed), ROOT_HEX, "the pushed bytes must be the root");

    // THE CHANGE OUTPUT must be a valid witness program rust-bitcoin can spend.
    assert!(
        parsed.output[1].script_pubkey.is_p2wpkh(),
        "output 1 must be recognised as P2WPKH — otherwise the change is unspendable"
    );

    // SEGWIT: the witness must be present and carry two items.
    assert_eq!(parsed.input[0].witness.len(), 2, "witness is [signature, pubkey]");
}

#[test]
fn oracle_rust_bitcoin_agrees_our_address_matches_our_change_script() {
    use std::str::FromStr;

    let addr_str = p2wpkh_address(&pubkey(), BtcNetwork::Testnet).unwrap();
    let addr = bitcoin::Address::from_str(&addr_str)
        .expect("rust-bitcoin must parse our address")
        .require_network(bitcoin::Network::Testnet)
        .expect("address must belong to the network we said it did");

    // The address and the script we put in the transaction must denote the
    // same output — if they diverge, change goes somewhere we cannot spend.
    let ours = p2wpkh_script(&hash160(&pubkey()));
    assert_eq!(
        addr.script_pubkey().as_bytes(),
        ours.as_slice(),
        "our address and our change scriptPubKey disagree"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// P0.1 — structural guarantees about the canister's exported surface
// ───────────────────────────────────────────────────────────────────────────

fn canister_src_without_comments() -> String {
    include_str!("../../btc_signer_psbt/src/lib.rs")
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("//!")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The signing, spending and broadcasting primitives must NOT be callable from
/// outside. They were `#[update]` in the first Phase B build, which let any
/// principal on the IC drive the threshold key.
#[test]
fn p01_signing_primitives_are_not_exported() {
    let src = canister_src_without_comments();
    for dangerous in ["sign_transaction", "broadcast_transaction", "create_anchor_transaction", "get_btc_address"] {
        // Find the function definition and check what precedes it.
        let needle = format!("async fn {dangerous}(");
        let idx = src.find(&needle).unwrap_or_else(|| panic!("{dangerous} not found in canister source"));
        let preceding = &src[idx.saturating_sub(120)..idx];
        assert!(
            !preceding.contains("#[update]") && !preceding.contains("#[query]"),
            "{dangerous} is an exported canister method. It signs, spends or broadcasts, so \
             exporting it hands those capabilities to every principal on the IC."
        );
    }
}

/// Exactly one update method may exist, and it must be the authorized anchor
/// entry point. A second update is a second door.
#[test]
fn p01_exactly_one_update_method_is_exported() {
    let src = canister_src_without_comments();
    let updates = src.matches("#[update]").count();
    assert_eq!(
        updates, 1,
        "expected exactly one exported update method (create_and_broadcast_anchor); found {updates}"
    );
    let idx = src.find("#[update]").unwrap();
    assert!(
        src[idx..idx + 200].contains("create_and_broadcast_anchor"),
        "the single update method must be the authorized anchor entry point"
    );
}

/// THE CALLER MUST BE CAPTURED BEFORE THE FIRST AWAIT.
///
/// `ic_cdk::caller()` returns whoever is replying at that point in execution.
/// After an inter-canister await it is the MANAGEMENT CANISTER, not the
/// originator — so an authorization check placed after any await silently
/// checks the wrong principal. This canary pins the ordering that makes the
/// check meaningful.
#[test]
fn p01_caller_is_captured_before_the_first_await() {
    let src = canister_src_without_comments();
    let fn_idx = src.find("pub async fn create_and_broadcast_anchor").expect("entry point must exist");
    let body = &src[fn_idx..];
    // Matches BOTH `ic_cdk::caller()` (CDK 0.13) and `ic_cdk::api::msg_caller()`
    // (CDK 0.20+). The 0.20 migration renamed it, and this canary caught the
    // stale needle immediately — which is the behaviour wanted: the property
    // must be re-proven after an API change, not assumed to have survived it.
    let caller_idx = body
        .find("msg_caller()")
        .or_else(|| body.find("ic_cdk::caller()"))
        .expect(
            "the entry point must capture the caller. If the CDK renamed this API again, verify \
             the ordering property still holds before widening this search.",
        );
    let authorize_idx = body.find("authorize_anchor_caller(").expect("the entry point must authorize");
    let first_await = body.find(".await").expect("the entry point performs inter-canister calls");

    assert!(
        caller_idx < first_await,
        "ic_cdk::caller() is read AFTER an await, where it returns the management canister rather \
         than the originator — the authorization check would compare the wrong principal"
    );
    assert!(
        authorize_idx < first_await,
        "authorization happens after an await; by then any work the caller was not entitled to may \
         already have begun"
    );
}

/// The canister must not come up usable without explicit configuration, and
/// must not carry a compiled-in ECDSA key name.
#[test]
fn p01_no_implicit_network_or_key_defaults() {
    let src = canister_src_without_comments();
    assert!(
        !src.contains(r#"const KEY_NAME: &str = "test_key_1""#),
        "the ECDSA key name is compiled in; it must be a deployment decision"
    );
    assert!(
        src.contains("static CONFIG") && src.contains("RefCell::new(None)"),
        "configuration must start as None so an unconfigured canister denies rather than defaults"
    );
    assert!(src.contains("validate_anchor_config"), "init must validate its configuration");
}

// ───────────────────────────────────────────────────────────────────────────
// TRANSPORT ISOLATION (operator transport ruling, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────

/// The Constitutional Anchor must stay on the modern, non-deprecated Bitcoin
/// API, and must NOT be pulled back into the legacy workspace.
///
/// Isolation was necessary, not stylistic: `ic-cdk-executor` declares a Cargo
/// `links` key, and Cargo allows only one package with a given `links` value in
/// a dependency graph. Two ic-cdk majors therefore cannot cohabit, so the only
/// way to modernise the signer without uplifting the frozen `proof_of_state`
/// canister is a separate workspace.
#[test]
fn transport_uses_the_current_bitcoin_api_not_the_deprecated_facade() {
    let src = canister_src_without_comments();
    assert!(
        src.contains("ic_cdk_bitcoin_canister"),
        "the signer must use the maintained ic-cdk-bitcoin-canister crate"
    );
    assert!(
        !src.contains("api::management_canister::bitcoin"),
        "the deprecated ic_cdk::api::management_canister::bitcoin facade is back; it is the \
         transport amendment A1 replaced"
    );
    // No HTTP outcall may reappear: an outcall must reach byte-identical
    // responses across replicas to pass consensus, so a block explorer is
    // structurally consensus-hostile AND makes a third party the arbiter of
    // whether a constitutional anchor exists.
    assert!(
        !src.contains("http_request") && !src.contains("sendrawtransaction"),
        "an HTTPS-outcall broadcast path has reappeared"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// P0.3 — spend serialisation / idempotency (operator ruling, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────
//
// WAS RED against the pre-serialisation baseline, where `create_and_broadcast_
// anchor` had no attempt map at all: every one of these properties was simply
// absent, because there was no `decide_anchor_attempt` to call and no state to
// consult. A concurrent request for a different root proceeded straight into
// `bitcoin_get_utxos`/sign/broadcast with no idea another spend might already
// be in flight; a retry against an already-signed root re-fetched UTXOs and
// built a second, distinct transaction; a retry against an already-broadcast
// root re-signed and re-sent.

use std::collections::BTreeMap;

fn signed_state(txid: &str, raw_tx: &str) -> AnchorAttemptState {
    AnchorAttemptState::Signed {
        txid: txid.to_string(),
        raw_tx: raw_tx.to_string(),
        inputs: vec![AnchorInput { txid_hex: "aa".repeat(32), vout: 0, value: 100_000 }],
    }
}

fn reserved_state(attempt_id: &str, reserved_at_ns: u64) -> AnchorAttemptState {
    AnchorAttemptState::Reserved { attempt_id: attempt_id.to_string(), reserved_at_ns }
}

fn broadcast_state(txid: &str, inputs: Vec<AnchorInput>) -> AnchorAttemptState {
    AnchorAttemptState::Broadcast { txid: txid.to_string(), inputs }
}

fn one_input(txid_hex: &str, vout: u32, value: u64) -> AnchorInput {
    AnchorInput { txid_hex: txid_hex.to_string(), vout, value }
}

/// Two concurrent anchor attempts for DIFFERENT roots cannot both proceed —
/// the second must be refused, naming which root is active, whether the first
/// is merely `Reserved` or has already progressed to `Signed`.
#[test]
fn p03_concurrent_reservation_for_a_different_root_is_rejected() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), reserved_state("attempt-a", 0));

    assert_eq!(
        decide_anchor_attempt("root-b", &attempts),
        AnchorDecision::InProgress { active_root: "root-a".to_string() },
        "a second root must never be allowed to reserve while another root's ceremony is merely Reserved"
    );

    // The same exclusion holds once the active root has progressed to Signed
    // — an in-flight signed transaction is just as exclusive as a bare
    // reservation, because it still has not been broadcast (and might yet
    // need to be rebroadcast, spending the same inputs again).
    attempts.insert("root-a".to_string(), signed_state("txid-a", "aa"));
    assert_eq!(
        decide_anchor_attempt("root-b", &attempts),
        AnchorDecision::InProgress { active_root: "root-a".to_string() },
        "an in-flight Signed attempt is exclusive too, not only a bare Reserved one"
    );

    // Once root-a reaches Broadcast (terminal, successful), it is no longer
    // active, and root-b may reserve freely.
    attempts.insert("root-a".to_string(), broadcast_state("txid-a", vec![one_input(&"aa".repeat(32), 0, 100_000)]));
    assert_eq!(
        decide_anchor_attempt("root-b", &attempts),
        AnchorDecision::Reserve,
        "a root that has reached Broadcast is no longer active and must not block a different root"
    );
}

/// A retry against a root already at `Broadcast` must return the existing
/// txid and never be silently replayed as a new spend, signature, or
/// broadcast.
#[test]
fn p03_broadcast_is_never_silently_replayed() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), broadcast_state("abc123", vec![one_input(&"bb".repeat(32), 0, 100_000)]));

    let decision = decide_anchor_attempt("root-a", &attempts);
    assert_eq!(
        decision,
        AnchorDecision::ReturnBroadcast("abc123".to_string()),
        "a retry of an already-broadcast root must return the existing txid, never attempt another spend"
    );
    assert_ne!(decision, AnchorDecision::Reserve, "Broadcast must never be re-derived as a fresh reservation");
}

/// A retry against a root already at `Signed` must rebroadcast the EXACT
/// existing transaction — never fetch new UTXOs or construct a second spend
/// for the same root. This is the recovery path for "signing succeeded, then
/// the canister's execution was interrupted (upgrade, trap, out-of-cycles)
/// before the broadcast call resolved."
#[test]
fn p03_same_root_signed_resumes_by_rebroadcasting_never_builds_a_second_spend() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), signed_state("txid-a", "deadbeef"));

    let decision = decide_anchor_attempt("root-a", &attempts);
    assert_eq!(
        decision,
        AnchorDecision::Rebroadcast {
            txid: "txid-a".to_string(),
            raw_tx: "deadbeef".to_string(),
            inputs: vec![one_input(&"aa".repeat(32), 0, 100_000)],
        },
        "recovery from Signed must resume with the SAME raw_tx AND the SAME inputs (P0.4: needed so \
         the caller can retain them on the Broadcast it writes after rebroadcasting), never Reserve \
         — Reserve would let the canister fetch a fresh UTXO set and build a distinct, competing \
         spend for the same root"
    );
    // The one property that matters most, stated as its own assertion: this
    // must never be Reserve, under any circumstance, for a Signed root.
    assert_ne!(
        decision,
        AnchorDecision::Reserve,
        "no duplicate transaction may ever be generated for a root that already has a signed one"
    );
}

/// An unsuccessful PRE-broadcast attempt — no UTXOs, signing itself failed,
/// anything that ends before a signed transaction exists — must record a
/// truthful `Failed` state and permit a controlled retry. Nothing was ever
/// spent under `Failed` (or a bare, stalled `Reserved`), so starting over is
/// safe, unlike retrying from `Signed`.
#[test]
fn p03_a_failure_before_signing_permits_a_controlled_retry() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert(
        "root-a".to_string(),
        AnchorAttemptState::Failed { reason: "No UTXOs at tb1q... — refusing to anchor".to_string() },
    );
    assert_eq!(
        decide_anchor_attempt("root-a", &attempts),
        AnchorDecision::Reserve,
        "a root whose ceremony failed before a signed transaction existed must be retryable"
    );

    // A bare Reserved with no further progress is NOT the same case as
    // Failed (P0.3.1, independent review, 2026-08-08 — corrects the
    // assumption this test originally made here). Ordinary retry traffic
    // cannot tell "interrupted between reserving and signing, genuinely
    // stranded" apart from "still legitimately in flight, just slow" — so
    // decide_anchor_attempt must refuse a same-root retry against Reserved
    // exactly like a different root's active reservation. See
    // p03_1_same_root_reserved_refuses_a_concurrent_request_for_that_root
    // below for the dedicated test, and decide_stale_recovery for the
    // separate, explicit path that DOES eventually free a stranded root.
    attempts.insert("root-a".to_string(), reserved_state("attempt-a", 0));
    assert_eq!(
        decide_anchor_attempt("root-a", &attempts),
        AnchorDecision::InProgress { active_root: "root-a".to_string() },
        "a stalled Reserved attempt must never be silently re-derived as Reserve by ordinary retry — \
         that is exactly the reentrancy race that let two ceremonies build competing spends for one root"
    );

    // And a failed attempt for ONE root must never block a DIFFERENT root —
    // Failed releases exclusivity, unlike Reserved/Signed. Root-a is cleared
    // to Broadcast first (not left Reserved from above) so THIS assertion
    // tests Failed's non-exclusivity specifically, not a leftover active
    // reservation from the previous checks.
    attempts.insert("root-a".to_string(), broadcast_state("txid-a", vec![one_input(&"aa".repeat(32), 0, 100_000)]));
    attempts.insert("root-b".to_string(), AnchorAttemptState::Failed { reason: "transport error".to_string() });
    assert_eq!(
        decide_anchor_attempt("root-c", &attempts),
        AnchorDecision::Reserve,
        "root-b's Failed state must not block root-c from reserving"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// P0.3.1 — close the same-root reentrancy race (independent review, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────
//
// WAS RED against P0.3: `decide_anchor_attempt` let a SAME-ROOT `Reserved`
// fall through to the cross-root check, which returned `Reserve` whenever no
// OTHER root was active. Because `create_and_broadcast_anchor` inserts
// `Reserved` synchronously before its first network await, a second request
// for the SAME root — arriving while the first ceremony is still suspended
// on that await — would see `Reserved`, get `Reserve` back, and start a
// SECOND UTXO-fetch/signing ceremony racing the first to spend the same
// inputs. That breaks "one active Bitcoin spend at a time" exactly as
// thoroughly as the cross-root race P0.3 closed.

/// Call A reserves root H; a concurrent call B for the SAME root H must be
/// refused with `ANCHOR_IN_PROGRESS`, identically to how a different root's
/// active reservation is refused — never re-derived as `Reserve`.
#[test]
fn p03_1_same_root_reserved_refuses_a_concurrent_request_for_that_root() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    // Call A's reservation, taken synchronously before its first await.
    attempts.insert("root-h".to_string(), reserved_state("attempt-a", 1_000));

    // Call B, for the SAME root, while A's ceremony is still suspended.
    let decision = decide_anchor_attempt("root-h", &attempts);
    assert_eq!(
        decision,
        AnchorDecision::InProgress { active_root: "root-h".to_string() },
        "a same-root Reserved attempt must refuse a concurrent request for that SAME root, exactly \
         like a different root's active reservation — falling through to Reserve here is the race \
         that let two ceremonies build competing spends for one root"
    );
    assert_ne!(
        decision,
        AnchorDecision::Reserve,
        "a live Reserved attempt must never be re-derived as a fresh reservation by ordinary retry \
         traffic — only the separate, explicit stale-recovery path (decide_stale_recovery) may clear it"
    );
}

/// A DIFFERENT root's concurrent request remains refused after this fix too
/// — the same-root case above is an ADDITION to `decide_anchor_attempt`'s
/// exclusivity, not a replacement of the cross-root check. This mirrors
/// `p03_concurrent_reservation_for_a_different_root_is_rejected` deliberately,
/// pinned again here so the two guarantees are visibly tested side by side.
#[test]
fn p03_1_different_root_concurrent_request_remains_refused() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), reserved_state("attempt-a", 0));

    assert_eq!(
        decide_anchor_attempt("root-b", &attempts),
        AnchorDecision::InProgress { active_root: "root-a".to_string() },
        "a different root must still be refused while root-a is Reserved"
    );
}

/// Stale-reservation recovery is a SEPARATE, EXPLICIT act — never something
/// ordinary retry traffic performs (see the two tests above) — and it must
/// never let the ceremony it superseded go on to write `Signed` (or
/// `Failed`) as if nothing had happened. This is the "verify the stored
/// reservation is still that same attempt" half of the fix: once recovery
/// supersedes an old `attempt_id`, that old ceremony — however far along its
/// own awaits it already was — must find, on resuming, that it no longer
/// holds the reservation it started with.
#[test]
fn p03_1_stale_reservation_recovery_cannot_let_the_superseded_attempt_later_sign() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    let old_attempt_id = "attempt-old".to_string();
    attempts.insert("root-h".to_string(), reserved_state(&old_attempt_id, 0));

    // Too young to recover: refused by design — recovery must not race an
    // in-flight ceremony that is merely slow.
    assert!(
        decide_stale_recovery("root-h", 1_000, 5_000, &attempts).is_err(),
        "recovery must refuse a reservation that has not yet crossed the staleness threshold"
    );

    // Old enough now: recovery is permitted, and names the attempt it is
    // about to supersede.
    let recovered_attempt_id = decide_stale_recovery("root-h", 10_000, 5_000, &attempts)
        .expect("a reservation past the staleness threshold must be recoverable");
    assert_eq!(recovered_attempt_id, old_attempt_id);

    // Apply the recovery exactly as the canister would: release the root by
    // moving it to Failed — nothing was ever spent under Reserved, so this
    // is as safe as any other Failed transition.
    attempts.insert(
        "root-h".to_string(),
        AnchorAttemptState::Failed { reason: "recovered stale reservation".to_string() },
    );

    // The OLD ceremony's own await(s) now resolve, and it reaches the point
    // where it would persist Signed. It must find it no longer holds the
    // reservation it started with.
    assert!(
        !reservation_matches("root-h", &old_attempt_id, &attempts),
        "a ceremony whose reservation was recovered out from under it must detect the mismatch \
         before persisting Signed — recovery must not leave the superseded attempt free to write \
         over whatever state comes after it"
    );

    // A fresh ceremony (a NEW attempt_id) that reserves after recovery, by
    // contrast, is legitimately holding the root and must see a match.
    let new_attempt_id = "attempt-new".to_string();
    attempts.insert("root-h".to_string(), reserved_state(&new_attempt_id, 10_000));
    assert!(
        reservation_matches("root-h", &new_attempt_id, &attempts),
        "the legitimate, current reservation must still be recognised as matching its own attempt_id"
    );
    assert!(
        !reservation_matches("root-h", &old_attempt_id, &attempts),
        "the superseded attempt_id must never match again, even after a fresh reservation is taken"
    );
}

/// `AnchorAttemptState` — including the evidence needed for recovery, the
/// exact signed `raw_tx` and the inputs it spends, AND (P0.4) the inputs a
/// `Broadcast` attempt spent — must survive the SAME Candid encode/decode
/// round trip that `ic_cdk::storage::stable_save`/`stable_restore` performs
/// across a canister upgrade. This is the pure, host-testable half of
/// "anchor-attempt state and the evidence needed for recovery must survive
/// canister upgrades" — the stable-memory call itself needs a replica, but
/// the encoding it depends on does not.
#[test]
fn p03_anchor_attempt_state_survives_a_stable_memory_round_trip() {
    use candid::{Decode, Encode};

    let states = vec![
        reserved_state("attempt-a", 1_700_000_000_000_000_000),
        signed_state("txid-a", "deadbeef"),
        broadcast_state("txid-b", vec![one_input(&"cc".repeat(32), 1, 250_000)]),
        AnchorAttemptState::Failed { reason: "bitcoin_get_utxos failed: transport error".to_string() },
    ];

    for state in states {
        let bytes = Encode!(&state).expect("AnchorAttemptState must Candid-encode");
        let round_tripped: AnchorAttemptState =
            Decode!(&bytes, AnchorAttemptState).expect("AnchorAttemptState must Candid-decode");
        assert_eq!(
            round_tripped, state,
            "anchor-attempt state must round-trip losslessly through the encoding \
             ic_cdk::storage::stable_save/stable_restore uses across a canister upgrade"
        );
    }

    // The whole map, keyed by root — the actual shape persisted — round-trips
    // too, not only a single entry in isolation.
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), reserved_state("attempt-a", 1_700_000_000_000_000_000));
    attempts.insert("root-b".to_string(), signed_state("txid-b", "beefdead"));
    let as_vec: Vec<(String, AnchorAttemptState)> = attempts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let bytes = Encode!(&as_vec).expect("the persisted Vec<(String, AnchorAttemptState)> shape must encode");
    let round_tripped: Vec<(String, AnchorAttemptState)> =
        Decode!(&bytes, Vec<(String, AnchorAttemptState)>).expect("must decode");
    assert_eq!(round_tripped, as_vec);
}

// ───────────────────────────────────────────────────────────────────────────
// P0.3 — fee-rate honesty (operator ruling, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────

/// WAS RED-in-spirit against `median_fee_rate().await.unwrap_or(2)`: a failed
/// fee lookup silently became "2 sat/vB", a fee this function never computed
/// and the network never reported. The removal itself lives in
/// `btc_signer_psbt` (an `unwrap_or(2)` cannot exist in a pure, host-testable
/// crate); what belongs here is the ceiling-not-floor conversion the
/// replacement calls.
#[test]
fn p03_msat_per_vb_rounds_up_never_down() {
    // 1999 msat/vB floors to 1 sat/vB but must ceil to 2 — flooring would
    // understate the fee the network actually wants.
    assert_eq!(msat_per_vb_to_sat_per_vb_ceil(1999), 2);
    // An exact multiple of 1000 needs no rounding.
    assert_eq!(msat_per_vb_to_sat_per_vb_ceil(2000), 2);
    // One msat/vB over an exact multiple still rounds up to the next sat/vB.
    assert_eq!(msat_per_vb_to_sat_per_vb_ceil(2001), 3);
    // Zero input is guarded to the minimum valid fee rate, never zero —
    // plan_anchor refuses a zero fee_rate outright, so this guard's job is to
    // never hand it one.
    assert_eq!(msat_per_vb_to_sat_per_vb_ceil(0), 1);
}

// ───────────────────────────────────────────────────────────────────────────
// P0.4 — durable spent-input exclusion (operator ruling, 2026-08-08)
// ───────────────────────────────────────────────────────────────────────────
//
// WAS RED against the pre-P0.4 baseline, where `Broadcast` carried only a
// `txid` and discarded `inputs` entirely — there was no way for a fresh
// ceremony to know which outpoints a broadcast-but-unconfirmed anchor was
// still using, so `spent_outpoints`/`select_unspent_inputs` (below) simply
// could not exist: they read the `inputs` field this change adds.

/// A `Broadcast` root's inputs must be excluded from what a DIFFERENT root's
/// fresh ceremony may spend — the transaction is out on the network and may
/// yet confirm; a second root spending the same outpoint would either fail
/// or race it.
#[test]
fn p04_broadcast_roots_inputs_are_unavailable_to_a_different_root() {
    let spent_input = one_input(&"aa".repeat(32), 0, 100_000);
    let free_input = one_input(&"bb".repeat(32), 1, 50_000);

    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), broadcast_state("txid-a", vec![spent_input.clone()]));

    let available = select_unspent_inputs(vec![spent_input.clone(), free_input.clone()], &attempts)
        .expect("the free input alone must still be fundable");
    assert_eq!(
        available,
        vec![free_input.clone()],
        "root-a's Broadcast input must be excluded from root-b's candidate set, leaving only the \
         unrelated free UTXO"
    );
    assert!(
        !available.contains(&spent_input),
        "a Bitcoin outpoint committed to root-a's Broadcast attempt must never become selectable \
         for a different root merely because root-a reached Broadcast"
    );
}

/// The same exclusion holds for `Signed` — a valid transaction spending
/// these inputs already exists and may still be (re)broadcast at any time,
/// so it is exactly as unavailable to a different root as a `Broadcast` one.
#[test]
fn p04_signed_roots_inputs_are_unavailable_to_a_different_root() {
    let spent_input = one_input(&"aa".repeat(32), 0, 100_000);
    let free_input = one_input(&"cc".repeat(32), 2, 75_000);

    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), signed_state("txid-a", "deadbeef"));
    // signed_state's own input is one_input("aa"*32, 0, 100_000) — reuse the
    // same outpoint here so this test's `spent_input` genuinely collides
    // with it rather than coincidentally matching by construction.
    assert_eq!(spent_input, one_input(&"aa".repeat(32), 0, 100_000));

    let available = select_unspent_inputs(vec![spent_input.clone(), free_input.clone()], &attempts)
        .expect("the free input alone must still be fundable");
    assert_eq!(
        available,
        vec![free_input],
        "root-a's Signed input must be excluded from root-b's candidate set — a Signed transaction \
         may still be rebroadcast, so its inputs are exactly as spoken-for as a Broadcast one's"
    );
}

/// Free, unrelated UTXOs — never mentioned by any Signed/Broadcast attempt —
/// must remain selectable regardless of how many OTHER attempts exist.
#[test]
fn p04_unrelated_free_utxos_remain_selectable() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert(
        "root-a".to_string(),
        broadcast_state("txid-a", vec![one_input(&"aa".repeat(32), 0, 100_000)]),
    );
    attempts.insert("root-b".to_string(), signed_state("txid-b", "beefdead"));

    let free_1 = one_input(&"dd".repeat(32), 0, 10_000);
    let free_2 = one_input(&"ee".repeat(32), 3, 20_000);
    let available = select_unspent_inputs(vec![free_1.clone(), free_2.clone()], &attempts)
        .expect("UTXOs unrelated to any Signed/Broadcast attempt must remain fundable");
    assert_eq!(available, vec![free_1, free_2], "no unrelated input may be excluded");
}

/// If EVERY candidate UTXO is already committed to a Signed/Broadcast
/// attempt, the ceremony must refuse truthfully — never silently fall back
/// to a spent input, and never return an empty `Ok` that would be
/// indistinguishable from "there was nothing to exclude in the first place".
#[test]
fn p04_refuses_truthfully_when_all_candidate_utxos_are_already_reserved() {
    let only_input = one_input(&"aa".repeat(32), 0, 100_000);
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), broadcast_state("txid-a", vec![only_input.clone()]));

    let err = select_unspent_inputs(vec![only_input], &attempts)
        .err()
        .expect("refusing is the point — falling back to a spent input would risk a double-spend");
    assert!(
        err.contains("ALL_UTXOS_ALREADY_RESERVED"),
        "the refusal must name itself distinctly from a plain 'no UTXOs' error, so an operator can \
         tell 'wait for confirmation' apart from 'fund a new address': {err}"
    );
}

/// P0.4 must not regress the P0.3/P0.3.1 guarantees it sits beside: a
/// same-root `Signed` retry still rebroadcasts the identical stored bytes
/// (and, now, the identical stored inputs) rather than rebuilding anything.
#[test]
fn p04_same_root_signed_retry_still_rebroadcasts_identical_bytes() {
    let mut attempts: BTreeMap<String, AnchorAttemptState> = BTreeMap::new();
    attempts.insert("root-a".to_string(), signed_state("txid-a", "deadbeef"));

    assert_eq!(
        decide_anchor_attempt("root-a", &attempts),
        AnchorDecision::Rebroadcast {
            txid: "txid-a".to_string(),
            raw_tx: "deadbeef".to_string(),
            inputs: vec![one_input(&"aa".repeat(32), 0, 100_000)],
        },
        "a same-root Signed retry must still rebroadcast the EXACT stored raw_tx and inputs, \
         unaffected by P0.4's addition of inputs to Broadcast"
    );
}

/// State round-trip (P0.3's guarantee) must extend to `Broadcast`'s newly
/// retained `inputs` — this is re-asserted here, alongside the exclusion
/// tests, so P0.4's own core data addition is pinned by name in this
/// section rather than only incidentally by the P0.3 round-trip test above.
#[test]
fn p04_state_round_trip_preserves_broadcast_inputs() {
    use candid::{Decode, Encode};

    let inputs = vec![one_input(&"aa".repeat(32), 0, 100_000), one_input(&"bb".repeat(32), 2, 50_000)];
    let state = broadcast_state("txid-a", inputs.clone());

    let bytes = Encode!(&state).expect("Broadcast{inputs} must Candid-encode");
    let round_tripped: AnchorAttemptState =
        Decode!(&bytes, AnchorAttemptState).expect("Broadcast{inputs} must Candid-decode");
    assert_eq!(round_tripped, state, "Broadcast's retained inputs must round-trip losslessly");
    match round_tripped {
        AnchorAttemptState::Broadcast { inputs: round_tripped_inputs, .. } => {
            assert_eq!(round_tripped_inputs, inputs, "the exact spent outpoints must survive the round trip")
        }
        other => panic!("expected Broadcast, got {other:?}"),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// P0.3 — structural guarantee about the canister's own wiring
// ───────────────────────────────────────────────────────────────────────────

/// SIGNED MUST BE DURABLE BEFORE THE NETWORK IS ASKED TO BROADCAST IT.
///
/// `decide_anchor_attempt` can only make retrying from `Signed` safe if
/// `Signed` was actually recorded before the send that might be interrupted.
/// This pins the ordering the same way `p01_caller_is_captured_before_the_
/// first_await` pins caller-capture: by finding both markers in the
/// comment-stripped source and asserting on their positions, so the property
/// is re-checked on every future edit rather than trusted to have survived one.
#[test]
fn p03_signed_state_is_persisted_before_the_fresh_ceremony_broadcasts() {
    let src = canister_src_without_comments();
    let fn_idx = src.find("pub async fn create_and_broadcast_anchor").expect("entry point must exist");
    let body = &src[fn_idx..];

    // Unique in the file: the ONLY place a NEW Signed value is constructed.
    // The Rebroadcast branch reads an existing txid/raw_tx out of the
    // decision — it never builds this literal.
    let signed_insert_idx = body
        .find("AnchorAttemptState::Signed { txid: txid.clone()")
        .expect("the fresh-ceremony path must persist a Signed state before broadcasting");
    // The LAST bitcoin_send_transaction call in the function is the
    // fresh-ceremony one — Rebroadcast's own call sits earlier in the file,
    // in an earlier match arm.
    let last_broadcast_idx =
        body.rfind("bitcoin_send_transaction(").expect("the entry point must broadcast");

    assert!(
        signed_insert_idx < last_broadcast_idx,
        "Signed must be persisted BEFORE bitcoin_send_transaction is invoked for a fresh ceremony — \
         otherwise a trap, upgrade, or ambiguous resumption between signing and broadcast leaves no \
         durable record of which transaction to rebroadcast, and a retry could sign a second, \
         competing spend instead"
    );
}

/// FUNDING UTXOs MUST BE REQUESTED WITH AT LEAST ONE CONFIRMATION.
///
/// (P0.4, operator ruling, 2026-08-08: "request fresh funding UTXOs with at
/// least one confirmation rather than `filter: None`.") An unconfirmed UTXO
/// can vanish from a reorg — building a spend on top of one would risk a
/// signed, possibly-broadcast anchor transaction whose input silently
/// ceases to exist. This cannot be host-tested by calling the real
/// `bitcoin_get_utxos` (that needs a replica), so it is pinned structurally,
/// the same way `p03_signed_state_is_persisted_before_the_fresh_ceremony_
/// broadcasts` pins an ordering: by finding the marker in the
/// comment-stripped source.
#[test]
fn p04_funding_utxos_are_requested_with_at_least_one_confirmation() {
    let src = canister_src_without_comments();
    assert!(
        src.contains("UtxosFilterInRequest::MinConfirmations(1)"),
        "GetUtxosRequest must request at least one confirmation via \
         UtxosFilterInRequest::MinConfirmations(1) — an unconfirmed UTXO can vanish from a reorg"
    );
    assert!(
        !src.contains("filter: None"),
        "the get_utxos request's filter must never be literally None"
    );
}

/// `btc_anchor_core` must never take a CDK dependency. Its freedom from one is
/// what keeps all 19 pure tests and the rust-bitcoin oracle runnable on the
/// host, independent of which CDK the canister links against.
#[test]
fn core_stays_cdk_free() {
    // PARSES DEPENDENCY ENTRIES, NOT PROSE. A plain substring scan matched this
    // manifest's OWN COMMENT explaining that the crate has no ic-cdk dependency
    // — the second time in this session a canary has flagged the very text
    // documenting the rule it enforces. Test the declaration, not the vocabulary.
    let manifest = include_str!("../Cargo.toml");
    let mut in_deps = false;
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if t.starts_with('[') {
            in_deps = t.starts_with("[dependencies") || t.starts_with("[dev-dependencies");
            continue;
        }
        if !in_deps {
            continue;
        }
        let name = t.split(['=', ' ']).next().unwrap_or("").trim();
        assert!(
            !name.starts_with("ic-cdk") && !name.starts_with("ic_cdk"),
            "btc_anchor_core declares the CDK dependency {name:?}. The pure Bitcoin logic must \
             remain host-testable and independent of which CDK the canister links against."
        );
    }
}
