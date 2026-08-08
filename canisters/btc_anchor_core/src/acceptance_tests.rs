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
    let caller_idx = body.find("ic_cdk::caller()").expect("the entry point must capture the caller");
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
