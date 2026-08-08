//! Constitutional Anchor v2 — pure Bitcoin transaction construction.
//!
//! ─── WHY THIS MODULE IS PURE ────────────────────────────────────────────────
//!
//! Every function here is deterministic and free of `ic_cdk` calls, so the
//! anchoring format can be regression-tested on the host without a replica.
//! That is a requirement, not a convenience: the previous implementation's
//! anchor format could only be exercised on-chain, so no test could ever have
//! caught that it produced `signed_tx_<hex>` instead of a transaction. BT-3's
//! own failure message names this — *"an anchor format that can only be
//! exercised on-chain cannot be regression-tested."*
//!
//! ─── NAMING ────────────────────────────────────────────────────────────────
//!
//! "Constitutional Anchor v2" names the ARCHITECTURE GENERATION, not a
//! redeployment of a v1 that reached mainnet — the lineage census (2026-08-08)
//! established that no IC-mainnet BTC signer has ever existed. The package
//! remains `btc_signer_psbt`; this will be its first genuine mainnet
//! deployment.
//!
//! Byte encoding follows §A3 of AigentZBeta's
//! `codexes/packs/agentiq/updates/2026-08-08_canister-repair-plan.md`, which is
//! normative: a verifier written from that section alone must reproduce these
//! bytes.

use candid::{CandidType, Deserialize};
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

/// Which Bitcoin network an address/transaction targets. Carried explicitly —
/// never inferred from a build flag, because a testnet address silently used on
/// mainnet is an unrecoverable loss of funds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BtcNetwork {
    Mainnet,
    Testnet,
}

impl BtcNetwork {
    /// bech32 human-readable part. `bc` mainnet, `tb` testnet (BIP-173).
    pub fn hrp(&self) -> &'static str {
        match self {
            BtcNetwork::Mainnet => "bc",
            BtcNetwork::Testnet => "tb",
        }
    }
}

/// `SHA256(SHA256(x))` — Bitcoin's ubiquitous double hash.
pub fn sha256d(data: &[u8]) -> [u8; 32] {
    let once = Sha256::digest(data);
    let twice = Sha256::digest(once);
    let mut out = [0u8; 32];
    out.copy_from_slice(&twice);
    out
}

/// `RIPEMD160(SHA256(x))` — the witness program for P2WPKH.
pub fn hash160(data: &[u8]) -> [u8; 20] {
    let sha = Sha256::digest(data);
    let rip = Ripemd160::digest(sha);
    let mut out = [0u8; 20];
    out.copy_from_slice(&rip);
    out
}

/// Bitcoin's compact-size integer.
pub fn varint(n: u64) -> Vec<u8> {
    if n < 0xfd {
        vec![n as u8]
    } else if n <= 0xffff {
        let mut v = vec![0xfd];
        v.extend_from_slice(&(n as u16).to_le_bytes());
        v
    } else if n <= 0xffff_ffff {
        let mut v = vec![0xfe];
        v.extend_from_slice(&(n as u32).to_le_bytes());
        v
    } else {
        let mut v = vec![0xff];
        v.extend_from_slice(&n.to_le_bytes());
        v
    }
}

/// ── THE COMMITMENT OUTPUT ───────────────────────────────────────────────────
///
/// `OP_RETURN OP_PUSHBYTES_32 <root>` → `0x6a 0x20 ‖ root_bytes`.
///
/// The root arrives as 64-char hex and is **decoded to 32 raw bytes** before
/// being pushed (§A3). Pushing the ASCII hex would need 64 bytes and would
/// commit to a different value — the same class of mistake as hashing a hex
/// string instead of the bytes it denotes.
///
/// The predecessor built this exact script into `_op_return_script` and then
/// discarded it (underscore-prefixed, never read), which is why no commitment
/// ever reached Bitcoin.
pub fn op_return_script(root_hex: &str) -> Result<Vec<u8>, String> {
    // `normalize_root_hex` (P0.3) owns the "is this 32 raw bytes" validation —
    // reused here rather than duplicated, so the anchor-attempt map's notion
    // of a valid root and this function's never drift apart.
    let canonical = normalize_root_hex(root_hex)?;
    let root = hex::decode(&canonical).expect("normalize_root_hex guarantees valid, even-length hex");
    let mut script = Vec::with_capacity(34);
    script.push(0x6a); // OP_RETURN
    script.push(0x20); // OP_PUSHBYTES_32
    script.extend_from_slice(&root);
    Ok(script)
}

/// P2WPKH scriptPubKey: `OP_0 OP_PUSHBYTES_20 <hash160(pubkey)>`.
pub fn p2wpkh_script(h160: &[u8; 20]) -> Vec<u8> {
    let mut s = Vec::with_capacity(22);
    s.push(0x00); // OP_0 — witness version 0
    s.push(0x14); // push 20 bytes
    s.extend_from_slice(h160);
    s
}

/// ── ADDRESS DERIVATION ──────────────────────────────────────────────────────
///
/// Proper BIP-173 bech32 P2WPKH. Three things the predecessor got wrong, all in
/// one line (`format!("tb1q{}", hex::encode(&public_key[..20]))`):
///   1. it used the raw first 20 bytes of the pubkey, not `hash160(pubkey)`;
///   2. it emitted hex rather than bech32's 5-bit squashed encoding;
///   3. it had no checksum — so a typo could not be detected.
/// Funds sent to such an address are unspendable.
pub fn p2wpkh_address(compressed_pubkey: &[u8], network: BtcNetwork) -> Result<String, String> {
    if compressed_pubkey.len() != 33 {
        return Err(format!(
            "expected a 33-byte COMPRESSED secp256k1 pubkey, got {} bytes — an uncompressed key \
             yields a different hash160 and therefore a different address",
            compressed_pubkey.len()
        ));
    }
    let h160 = hash160(compressed_pubkey);
    let mut data = vec![bech32::u5::try_from_u8(0).unwrap()]; // witness version 0
    data.extend_from_slice(&bech32::ToBase32::to_base32(&h160.to_vec()));
    bech32::encode(network.hrp(), data, bech32::Variant::Bech32)
        .map_err(|e| format!("bech32 encoding failed: {e}"))
}

/// One transaction input, resolved against the UTXO it spends.
#[derive(Clone, Debug)]
pub struct TxIn {
    /// Big-endian display form, as Bitcoin shows txids.
    pub prev_txid_hex: String,
    pub vout: u32,
    /// Needed for the BIP-143 sighash — segwit commits to the spent amount.
    pub value: u64,
    pub sequence: u32,
}

/// One transaction output, carrying real script bytes.
#[derive(Clone, Debug)]
pub struct TxOut {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

/// A transaction prior to signing.
#[derive(Clone, Debug)]
pub struct Tx {
    pub version: i32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

fn prev_txid_le(hex_be: &str) -> Result<Vec<u8>, String> {
    let mut b = hex::decode(hex_be).map_err(|e| format!("prev txid is not hex: {e}"))?;
    if b.len() != 32 {
        return Err(format!("prev txid must be 32 bytes, got {}", b.len()));
    }
    b.reverse(); // display order is big-endian; consensus order is little-endian
    Ok(b)
}

/// Consensus serialisation WITHOUT the segwit marker/flag and witnesses. This
/// is what a txid is computed over — which is precisely why segwit txids are
/// stable under witness malleation.
pub fn serialize_tx_no_witness(tx: &Tx) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    out.extend_from_slice(&tx.version.to_le_bytes());
    out.extend_from_slice(&varint(tx.inputs.len() as u64));
    for i in &tx.inputs {
        out.extend_from_slice(&prev_txid_le(&i.prev_txid_hex)?);
        out.extend_from_slice(&i.vout.to_le_bytes());
        out.extend_from_slice(&varint(0)); // empty scriptSig — the signature lives in the witness
        out.extend_from_slice(&i.sequence.to_le_bytes());
    }
    out.extend_from_slice(&varint(tx.outputs.len() as u64));
    for o in &tx.outputs {
        out.extend_from_slice(&o.value.to_le_bytes());
        out.extend_from_slice(&varint(o.script_pubkey.len() as u64));
        out.extend_from_slice(&o.script_pubkey);
    }
    out.extend_from_slice(&tx.locktime.to_le_bytes());
    Ok(out)
}

/// Full segwit serialisation, including marker, flag and witnesses. This is the
/// form broadcast to the network.
pub fn serialize_tx_with_witness(tx: &Tx, witnesses: &[Vec<Vec<u8>>]) -> Result<Vec<u8>, String> {
    if witnesses.len() != tx.inputs.len() {
        return Err(format!(
            "witness count {} does not match input count {}",
            witnesses.len(),
            tx.inputs.len()
        ));
    }
    let mut out = Vec::new();
    out.extend_from_slice(&tx.version.to_le_bytes());
    out.push(0x00); // segwit marker
    out.push(0x01); // segwit flag
    out.extend_from_slice(&varint(tx.inputs.len() as u64));
    for i in &tx.inputs {
        out.extend_from_slice(&prev_txid_le(&i.prev_txid_hex)?);
        out.extend_from_slice(&i.vout.to_le_bytes());
        out.extend_from_slice(&varint(0));
        out.extend_from_slice(&i.sequence.to_le_bytes());
    }
    out.extend_from_slice(&varint(tx.outputs.len() as u64));
    for o in &tx.outputs {
        out.extend_from_slice(&o.value.to_le_bytes());
        out.extend_from_slice(&varint(o.script_pubkey.len() as u64));
        out.extend_from_slice(&o.script_pubkey);
    }
    for w in witnesses {
        out.extend_from_slice(&varint(w.len() as u64));
        for item in w {
            out.extend_from_slice(&varint(item.len() as u64));
            out.extend_from_slice(item);
        }
    }
    out.extend_from_slice(&tx.locktime.to_le_bytes());
    Ok(out)
}

/// The txid: `sha256d` of the witness-free serialisation, displayed in reverse
/// byte order.
///
/// The predecessor set `txid = hex::encode(&signature[..32])` — the first 32
/// bytes of the ECDSA signature. That value identifies no transaction on any
/// network, which is exactly why every recorded "anchor" was unfindable.
pub fn compute_txid(tx: &Tx) -> Result<String, String> {
    let bytes = serialize_tx_no_witness(tx)?;
    let mut h = sha256d(&bytes);
    h.reverse();
    Ok(hex::encode(h))
}

/// BIP-143 sighash for a P2WPKH input (SIGHASH_ALL).
///
/// Segwit commits to the spent amount and to every prevout/sequence/output,
/// which is what makes offline signing safe. `scriptCode` for P2WPKH is the
/// canonical P2PKH script over the same hash160 — a spec quirk, not a mistake.
pub fn bip143_sighash_p2wpkh(tx: &Tx, index: usize, h160: &[u8; 20]) -> Result<[u8; 32], String> {
    let input = tx.inputs.get(index).ok_or_else(|| format!("no input at index {index}"))?;

    let mut prevouts = Vec::new();
    let mut sequences = Vec::new();
    for i in &tx.inputs {
        prevouts.extend_from_slice(&prev_txid_le(&i.prev_txid_hex)?);
        prevouts.extend_from_slice(&i.vout.to_le_bytes());
        sequences.extend_from_slice(&i.sequence.to_le_bytes());
    }
    let mut outputs = Vec::new();
    for o in &tx.outputs {
        outputs.extend_from_slice(&o.value.to_le_bytes());
        outputs.extend_from_slice(&varint(o.script_pubkey.len() as u64));
        outputs.extend_from_slice(&o.script_pubkey);
    }

    // scriptCode = OP_DUP OP_HASH160 <20> <h160> OP_EQUALVERIFY OP_CHECKSIG
    let mut script_code = vec![0x19, 0x76, 0xa9, 0x14];
    script_code.extend_from_slice(h160);
    script_code.extend_from_slice(&[0x88, 0xac]);

    let mut pre = Vec::new();
    pre.extend_from_slice(&tx.version.to_le_bytes());
    pre.extend_from_slice(&sha256d(&prevouts));
    pre.extend_from_slice(&sha256d(&sequences));
    pre.extend_from_slice(&prev_txid_le(&input.prev_txid_hex)?);
    pre.extend_from_slice(&input.vout.to_le_bytes());
    pre.extend_from_slice(&script_code);
    pre.extend_from_slice(&input.value.to_le_bytes());
    pre.extend_from_slice(&input.sequence.to_le_bytes());
    pre.extend_from_slice(&sha256d(&outputs));
    pre.extend_from_slice(&tx.locktime.to_le_bytes());
    pre.extend_from_slice(&1u32.to_le_bytes()); // SIGHASH_ALL

    Ok(sha256d(&pre))
}

/// Encode a 64-byte compact (r‖s) signature as DER, low-S normalised, with the
/// SIGHASH_ALL byte appended — the form a witness carries.
///
/// Low-S normalisation is consensus-relevant: a high-S signature is valid
/// cryptographically but non-standard, and nodes will not relay it. A signer
/// that skips this produces transactions that verify locally and never
/// propagate — a failure that would look exactly like "broadcast did nothing".
pub fn compact_sig_to_der(sig: &[u8]) -> Result<Vec<u8>, String> {
    if sig.len() != 64 {
        return Err(format!("expected a 64-byte compact signature, got {}", sig.len()));
    }
    let r = &sig[..32];
    let s_raw = &sig[32..];

    // secp256k1 curve order n
    const N: [u8; 32] = [
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
    ];
    const HALF_N: [u8; 32] = [
        0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x5d, 0x57, 0x6e, 0x73, 0x57, 0xa4, 0x50, 0x1d, 0xdf, 0xe9, 0x2f, 0x46, 0x68, 0x1b, 0x20, 0xa0,
    ];

    let s_owned: Vec<u8> = if s_raw > &HALF_N[..] {
        // s = n - s
        let mut out = [0u8; 32];
        let mut borrow = 0i16;
        for i in (0..32).rev() {
            let d = N[i] as i16 - s_raw[i] as i16 - borrow;
            if d < 0 {
                out[i] = (d + 256) as u8;
                borrow = 1;
            } else {
                out[i] = d as u8;
                borrow = 0;
            }
        }
        out.to_vec()
    } else {
        s_raw.to_vec()
    };

    fn der_int(v: &[u8]) -> Vec<u8> {
        let mut t = v;
        while t.len() > 1 && t[0] == 0 {
            t = &t[1..];
        }
        let mut out = vec![0x02];
        if t[0] & 0x80 != 0 {
            out.push((t.len() + 1) as u8);
            out.push(0x00);
        } else {
            out.push(t.len() as u8);
        }
        out.extend_from_slice(t);
        out
    }

    let rd = der_int(r);
    let sd = der_int(&s_owned);
    let mut der = vec![0x30, (rd.len() + sd.len()) as u8];
    der.extend_from_slice(&rd);
    der.extend_from_slice(&sd);
    der.push(0x01); // SIGHASH_ALL
    Ok(der)
}

/// Assemble a fully signed transaction from its parts. Pure — this is the entry
/// point the acceptance tests exercise without a replica.
pub fn assemble_signed(
    tx: &Tx,
    compact_sigs: &[Vec<u8>],
    compressed_pubkey: &[u8],
) -> Result<(String, String), String> {
    if compact_sigs.len() != tx.inputs.len() {
        return Err(format!(
            "have {} signatures for {} inputs",
            compact_sigs.len(),
            tx.inputs.len()
        ));
    }
    let witnesses: Vec<Vec<Vec<u8>>> = compact_sigs
        .iter()
        .map(|s| compact_sig_to_der(s).map(|der| vec![der, compressed_pubkey.to_vec()]))
        .collect::<Result<_, _>>()?;
    let raw = serialize_tx_with_witness(tx, &witnesses)?;
    let txid = compute_txid(tx)?;
    Ok((txid, hex::encode(raw)))
}

// ─── ANCHOR PLANNING — refusal logic, pure and host-testable ────────────────

/// A candidate input, reduced to what planning needs.
///
/// `PartialEq, Eq` + `CandidType, Deserialize` (P0.3): this struct is embedded
/// in `AnchorAttemptState::Signed` and must both compare for equality (tests)
/// and survive a Candid encode/decode round trip (canister upgrades).
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub struct AnchorInput {
    pub txid_hex: String,
    pub vout: u32,
    pub value: u64,
}

/// The result of planning an anchor: what it will cost and what comes back.
#[derive(Clone, Debug)]
pub struct AnchorPlan {
    pub op_return: Vec<u8>,
    pub change_value: u64,
    pub estimated_vsize: u64,
    pub fee: u64,
}

/// Dust threshold for a P2WPKH output. Below this a change output is
/// unspendable in practice and relaying nodes reject it, so the value must go
/// to fee instead of creating an output nobody can ever claim.
pub const P2WPKH_DUST_SAT: u64 = 294;

/// Plan an anchor, or REFUSE.
///
/// Refusal is the point of this function. The predecessor substituted a UTXO
/// with an all-zero txid when it had nothing to spend and proceeded, which
/// manufactured the appearance of an anchor. Every "cannot" here is an `Err`
/// naming the reason — never a placeholder that lets the ceremony continue.
pub fn plan_anchor(inputs: &[AnchorInput], root_hex: &str, fee_rate: u64) -> Result<AnchorPlan, String> {
    if inputs.is_empty() {
        return Err(
            "No UTXOs provided — refusing to build an anchor with nothing to spend. A placeholder \
             input would manufacture the appearance of a Bitcoin anchor."
                .to_string(),
        );
    }
    if fee_rate == 0 {
        return Err("fee_rate must be greater than zero".to_string());
    }
    // Validate the commitment BEFORE computing anything about money.
    let op_return = op_return_script(root_hex)?;

    let total_input: u64 = inputs.iter().try_fold(0u64, |acc, i| {
        acc.checked_add(i.value).ok_or_else(|| "input values overflow u64".to_string())
    })?;

    // P2WPKH input ≈ 68 vB; OP_RETURN(34-byte script) output 43 vB; P2WPKH
    // change output 31 vB; version/counts/locktime overhead ≈ 11 vB.
    let estimated_vsize = 11 + 68 * inputs.len() as u64 + 43 + 31;
    let fee = fee_rate
        .checked_mul(estimated_vsize)
        .ok_or_else(|| "fee calculation overflows u64".to_string())?;

    if total_input <= fee {
        return Err(format!(
            "Insufficient funds: inputs total {total_input} sat, estimated fee {fee} sat \
             ({estimated_vsize} vB at {fee_rate} sat/vB)"
        ));
    }
    let change_value = total_input - fee;
    if change_value < P2WPKH_DUST_SAT {
        return Err(format!(
            "Change {change_value} sat is below the {P2WPKH_DUST_SAT} sat dust threshold — the \
             output would be unspendable and nodes would reject the transaction"
        ));
    }
    Ok(AnchorPlan { op_return, change_value, estimated_vsize, fee })
}

/// Validate that a hex string really is transaction bytes before broadcast.
///
/// The predecessor fed `"signed_tx_<hex>"` straight to a node and, on failing
/// to parse a txid back, returned `Ok(format!("broadcast_success_{…}"))` — a
/// success synthesised from its own input. Refusing here, by name, is what
/// makes the broadcast path's telemetry trustworthy.
pub fn validate_raw_tx_hex(raw: &str) -> Result<Vec<u8>, String> {
    if raw.is_empty() {
        return Err("raw_tx is empty".to_string());
    }
    if raw.starts_with("signed_tx_") {
        return Err(
            "raw_tx is a label of the form \"signed_tx_<hex>\", not a serialised transaction"
                .to_string(),
        );
    }
    let bytes = hex::decode(raw)
        .map_err(|e| format!("raw_tx is not hex, so it is not a serialised transaction: {e}"))?;
    if bytes.len() < 60 {
        return Err(format!(
            "raw_tx decodes to {} bytes — far too short to be a Bitcoin transaction",
            bytes.len()
        ));
    }
    Ok(bytes)
}

/// Phase B acceptance contract.
#[cfg(test)]
#[path = "acceptance_tests.rs"]
mod acceptance_tests;

// ─── GOVERNED CONFIGURATION AND AUTHORIZATION ───────────────────────────────
//
// P0.1/P0.2 (independent review, 2026-08-08). Two boundary defects were found
// in the first Phase B build:
//
//   * `sign_transaction`, `broadcast_transaction` and `create_anchor_transaction`
//     were unrestricted public `#[update]` methods. ANY principal could make the
//     canister sign with its threshold key, spend its UTXOs, or broadcast
//     arbitrary bytes. A signer whose signing surface is open is not a signer,
//     it is an oracle for anyone who asks.
//   * network and ECDSA key name were implicit defaults (`Testnet`,
//     `test_key_1`), so a mainnet deployment would silently sign with a test
//     key — keys that carry no security guarantee and can be rotated by the
//     subnet.
//
// Both decisions are made HERE, in pure code, so they are host-testable and so
// the rule that is tested is the rule that runs.

/// Everything about this canister's behaviour that must be a deliberate,
/// recorded deployment decision rather than a compiled-in default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchorConfig {
    pub network: BtcNetwork,
    /// `dfx_test_key` (local), `test_key_1` (testnet), `key_1` (production).
    pub ecdsa_key_name: String,
    /// Textual principal of the `proof_of_state` canister permitted to request
    /// anchoring. `None` means NOT CONFIGURED, which denies everyone — the
    /// canister is inert until governance names its caller.
    pub authorized_pos_principal: Option<String>,
}

/// The only ECDSA key names that carry a production security guarantee.
pub const PRODUCTION_ECDSA_KEYS: &[&str] = &["key_1"];

/// Authorize an anchoring request.
///
/// FAILS CLOSED in every ambiguous case. An unset principal denies rather than
/// allows, because the failure mode of the opposite default is that a
/// misconfigured deployment silently accepts anyone's request to spend its
/// funds and sign with its key.
///
/// The caller MUST be captured before the first `await`. On the IC,
/// `ic_cdk::caller()` returns whoever is replying at that point in execution —
/// after an inter-canister await that is the management canister, not the
/// originator. Authorizing on a post-await caller would check the wrong
/// principal entirely, and would pass.
pub fn authorize_anchor_caller(caller: &str, cfg: &AnchorConfig) -> Result<(), String> {
    // The anonymous principal is never a legitimate constitutional actor.
    if caller == "2vxsx-fae" {
        return Err(
            "the anonymous principal may not request anchoring: signing and spending require a \
             named, authorized caller"
                .to_string(),
        );
    }
    match cfg.authorized_pos_principal.as_deref() {
        None => Err(
            "no authorized proof_of_state principal is configured — this canister denies all \
             anchoring requests until governance names its caller. Denying is deliberate: the \
             alternative default would let a misconfigured deployment sign for anyone."
                .to_string(),
        ),
        Some(expected) if expected == caller => Ok(()),
        Some(expected) => Err(format!(
            "caller {caller} is not the authorized proof_of_state principal ({expected}); \
             refusing to sign, spend or broadcast"
        )),
    }
}

/// Reject configurations that would sign production value with a test key, or
/// name a network without naming a key at all.
pub fn validate_anchor_config(cfg: &AnchorConfig) -> Result<(), String> {
    if cfg.ecdsa_key_name.trim().is_empty() {
        return Err("ecdsa_key_name must be set explicitly; there is no safe default".to_string());
    }
    if cfg.network == BtcNetwork::Mainnet && !PRODUCTION_ECDSA_KEYS.contains(&cfg.ecdsa_key_name.as_str()) {
        return Err(format!(
            "refusing a Bitcoin MAINNET configuration with ECDSA key {:?}: only {:?} carry a \
             production guarantee. A test key can be rotated by the subnet, which would make every \
             address derived from it — and any funds at those addresses — unrecoverable.",
            cfg.ecdsa_key_name, PRODUCTION_ECDSA_KEYS
        ));
    }
    if cfg.authorized_pos_principal.as_deref() == Some("2vxsx-fae") {
        return Err("the anonymous principal may not be configured as the authorized caller".to_string());
    }
    Ok(())
}

// ─── ANCHOR ATTEMPT SERIALISATION — one active Bitcoin spend at a time ─────
//
// P0.3 (operator ruling, 2026-08-08): "Add a durable root-indexed anchor-
// attempt state machine. Before the first network await, atomically reserve
// the anchor ceremony. Permit only one active Bitcoin spend at a time... No
// boolean-only lock that can strand silently after a callback failure; state
// must identify the root and lifecycle."
//
// Lifecycle: Reserved(root) -> Signed(root, txid, raw_tx, inputs) ->
// Broadcast(root, txid), with Failed(root, reason) reachable from Reserved
// whenever the ceremony ends before a signed transaction exists.
//
// THIS MODULE OWNS ONLY THE DECISION — pure, host-testable, no storage and no
// ic_cdk. `btc_signer_psbt` owns the actual root-indexed map and its
// stable-memory persistence; it calls `decide_anchor_attempt` synchronously,
// as the FIRST thing after authorization and BEFORE its first network
// `.await`, and applies whichever decision comes back. IC message execution
// is atomic up to that first await, which is what makes a plain
// check-and-insert into a `RefCell`-guarded map an atomic reservation without
// a separate lock primitive.

/// One root's anchor attempt, at whatever point in its lifecycle it currently
/// sits. `CandidType, Deserialize` so the canister can persist the whole map
/// through `ic_cdk::storage::stable_save`/`stable_restore` — the same
/// mechanism `InitArg` already uses — and so the evidence needed for recovery
/// (the exact signed `raw_tx` and the inputs it spends) survives a canister
/// upgrade rather than only living in heap memory.
#[derive(Clone, Debug, PartialEq, Eq, CandidType, Deserialize)]
pub enum AnchorAttemptState {
    /// A ceremony has claimed this root and no other root may become active
    /// until it resolves. Nothing has been signed or spent yet.
    ///
    /// `attempt_id` identifies THIS specific ceremony's claim (P0.3.1,
    /// independent review, 2026-08-08). It is NOT safe to retry the way
    /// `Failed` is: a same-root `Reserved` entry may simply be the ORIGINAL
    /// caller's own ceremony, still suspended on a network await, so
    /// `decide_anchor_attempt` refuses a same-root request against it rather
    /// than treating it as free to reserve again (see that function's docs).
    /// Recovering a reservation that is genuinely stranded — its ceremony
    /// trapped, or was upgraded away — is a separate, explicit act gated on
    /// `reserved_at_ns` (see `decide_stale_recovery`), never something
    /// ordinary retry traffic performs. `attempt_id` is what lets a recovery
    /// supersede a *specific* stale claim, and lets a ceremony that resumes
    /// after its own awaits detect — via `reservation_matches` — that its
    /// claim was the one superseded, so it never writes `Signed` (or
    /// `Failed`) over whatever superseded it.
    Reserved { attempt_id: String, reserved_at_ns: u64 },
    /// A transaction has been built and signed for this root. `raw_tx` is the
    /// exact bytes that must be (re)broadcast — a retry from this state must
    /// NEVER fetch new UTXOs or construct a different spend, because doing so
    /// while the first signed transaction might still confirm would risk a
    /// double-spend of the same inputs.
    Signed { txid: String, raw_tx: String, inputs: Vec<AnchorInput> },
    /// The network accepted the transaction. Terminal and successful — any
    /// further request for this root returns this txid and does nothing
    /// else. The constitutional event already happened.
    ///
    /// `inputs` is retained deliberately (P0.4, operator ruling, 2026-08-08).
    /// `Broadcast` releasing exclusivity used to discard them, which let a
    /// LATER root fetch and spend an outpoint a broadcast-but-not-yet-
    /// confirmed anchor was still using — a Bitcoin outpoint belongs to
    /// whichever root's transaction is out on the network, confirmed or
    /// not, until it either confirms or is definitively replaced; it must
    /// never become selectable for a second root merely because the first
    /// reached `Broadcast`. See `spent_outpoints`/`select_unspent_inputs`,
    /// which read this field to build the durable used-outpoint exclusion
    /// set every fresh ceremony consults.
    Broadcast { txid: String, inputs: Vec<AnchorInput> },
    /// The ceremony ended before a signed transaction existed (no UTXOs,
    /// signing itself failed, a transport error before broadcast). Terminal
    /// for THIS attempt, but — unlike `Broadcast` — it releases the root: a
    /// later request for the same root may reserve again and try fresh,
    /// because nothing was ever spent under this state.
    Failed { reason: String },
}

/// What the canister may do next, having asked `decide_anchor_attempt`. Pure
/// data — applying it (making the reservation, rebroadcasting, or refusing)
/// is the canister's job, not this function's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnchorDecision {
    /// No conflicting attempt exists. The caller may insert `Reserved` for
    /// this root and proceed with a fresh ceremony.
    Reserve,
    /// This root already reached `Broadcast`. Return the existing txid;
    /// spend nothing.
    ReturnBroadcast(String),
    /// This root already reached `Signed`. Rebroadcast `raw_tx` exactly as
    /// stored; do not rebuild. `inputs` carries over from `Signed` so the
    /// caller can retain them on the `Broadcast` it writes after a
    /// successful rebroadcast (P0.4) — this path must preserve the same
    /// used-outpoint record a fresh ceremony's `Broadcast` write does.
    Rebroadcast { txid: String, raw_tx: String, inputs: Vec<AnchorInput> },
    /// A DIFFERENT root currently holds an active (`Reserved` or `Signed`)
    /// attempt. Refuse — only one Bitcoin spend ceremony may be active across
    /// all roots at a time.
    InProgress { active_root: String },
}

/// Canonicalise a 64-hex-char root to lowercase, validating it decodes to
/// exactly 32 bytes. The attempt map is keyed on this canonical form so two
/// differently-cased spellings of the same root — or a root with incidental
/// leading/trailing whitespace stripped by hex parsing — are never treated as
/// two distinct anchor attempts.
pub fn normalize_root_hex(root_hex: &str) -> Result<String, String> {
    let root = hex::decode(root_hex).map_err(|e| format!("root is not hex: {e}"))?;
    if root.len() != 32 {
        return Err(format!("root must be 32 bytes, got {}", root.len()));
    }
    Ok(hex::encode(root))
}

/// The P0.3 decision function.
///
/// SAME ROOT AS THE REQUEST:
///   * `Broadcast(txid)` -> `ReturnBroadcast`. The event already happened; a
///     retry must not spend again, silently or otherwise.
///   * `Signed{..}` -> `Rebroadcast` with the EXACT existing `raw_tx`/`txid`.
///     Never refetch UTXOs or build a second spend for a root that already
///     has a valid signed transaction — that is how a root would end up with
///     two competing spends of the same inputs.
///   * `Reserved{..}` -> `InProgress { active_root: root }` (P0.3.1,
///     independent review, 2026-08-08 — corrects the original P0.3 rule,
///     which let this fall through to the cross-root check and return
///     `Reserve` whenever no OTHER root was active). Because the reservation
///     is inserted synchronously BEFORE the first network await,
///     `create_and_broadcast_anchor` can be re-entered for the SAME root
///     while the original ceremony is still suspended on that await — a
///     naive fall-through would let a second request see `Reserved`, get
///     `Reserve` back, and start a SECOND UTXO-fetch/signing ceremony racing
///     the first to spend the same inputs. Ordinary retry traffic cannot
///     distinguish "still legitimately in flight" from "genuinely stranded",
///     so it must never be granted a fresh reservation either way. Only
///     `decide_stale_recovery`, gated on elapsed time, may clear a `Reserved`
///     entry — never this function.
///   * `Failed` or no entry at all -> falls through to the cross-root check
///     below. Nothing was ever spent under `Failed`, so resuming from
///     scratch is safe PROVIDED no other root is active.
///
/// ANY OTHER ROOT (checked only once the same-root cases above are ruled
/// out): if it holds `Reserved` or `Signed` — anything short of `Broadcast`
/// or `Failed` — the requested root gets `InProgress`. Only one ceremony may
/// be active at a time, across every root, which is what "one active Bitcoin
/// spend" means when spends share a UTXO set.
pub fn decide_anchor_attempt(
    root: &str,
    attempts: &std::collections::BTreeMap<String, AnchorAttemptState>,
) -> AnchorDecision {
    if let Some(state) = attempts.get(root) {
        match state {
            AnchorAttemptState::Broadcast { txid, .. } => return AnchorDecision::ReturnBroadcast(txid.clone()),
            AnchorAttemptState::Signed { txid, raw_tx, inputs } => {
                return AnchorDecision::Rebroadcast {
                    txid: txid.clone(),
                    raw_tx: raw_tx.clone(),
                    inputs: inputs.clone(),
                };
            }
            AnchorAttemptState::Reserved { .. } => {
                return AnchorDecision::InProgress { active_root: root.to_string() };
            }
            AnchorAttemptState::Failed { .. } => {
                // Falls through to the cross-root exclusivity check below.
            }
        }
    }
    match attempts.iter().find(|(other_root, state)| {
        other_root.as_str() != root
            && matches!(state, AnchorAttemptState::Reserved { .. } | AnchorAttemptState::Signed { .. })
    }) {
        Some((other_root, _)) => AnchorDecision::InProgress { active_root: other_root.clone() },
        None => AnchorDecision::Reserve,
    }
}

/// Whether `attempts[root]` is STILL the exact reservation identified by
/// `attempt_id`.
///
/// Called by the canister AFTER its network awaits (UTXO fetch, signing) and
/// BEFORE it persists `Signed` or `Failed` (P0.3.1, independent review,
/// 2026-08-08). A ceremony that resumes from an await must not assume it
/// still holds the root it reserved — an explicit stale-recovery act (see
/// `decide_stale_recovery`) may have superseded its reservation while it was
/// suspended, in which case a fresh ceremony could already be `Reserved`,
/// `Signed`, or even `Broadcast` under a DIFFERENT `attempt_id`. Writing
/// unconditionally at that point would let the superseded ceremony clobber
/// the current one's progress; checking first turns that into a detectable,
/// refusable mismatch instead.
pub fn reservation_matches(
    root: &str,
    attempt_id: &str,
    attempts: &std::collections::BTreeMap<String, AnchorAttemptState>,
) -> bool {
    matches!(
        attempts.get(root),
        Some(AnchorAttemptState::Reserved { attempt_id: stored, .. }) if stored == attempt_id
    )
}

/// Whether a POSSIBLY-STALE `Reserved` entry for `root` may be recovered
/// right now — i.e. cleared so a fresh ceremony may reserve again.
///
/// EXPLICIT AND SEPARATE FROM `decide_anchor_attempt` ON PURPOSE (P0.3.1,
/// operator ruling, 2026-08-08: "do not use normal retry to recover a
/// possibly stale Reserved; model stale-reservation recovery separately and
/// explicitly"). Ordinary retry traffic cannot tell a ceremony that is
/// merely slow from one that is genuinely stranded — `decide_anchor_attempt`
/// therefore never grants that distinction on its own. Recovery is instead a
/// deliberate act with its own staleness threshold, applied by the caller
/// (e.g. an operator-triggered or self-scheduled canister function) —
/// exactly which entry point does so remains internal to `btc_signer_psbt`
/// and is not yet part of this canister's public Candid surface.
///
/// Returns the `attempt_id` being recovered — for logging/audit, and for the
/// caller to confirm against — on success.
pub fn decide_stale_recovery(
    root: &str,
    now_ns: u64,
    staleness_threshold_ns: u64,
    attempts: &std::collections::BTreeMap<String, AnchorAttemptState>,
) -> Result<String, String> {
    match attempts.get(root) {
        Some(AnchorAttemptState::Reserved { attempt_id, reserved_at_ns }) => {
            let age_ns = now_ns.saturating_sub(*reserved_at_ns);
            if age_ns >= staleness_threshold_ns {
                Ok(attempt_id.clone())
            } else {
                Err(format!(
                    "root {root} is Reserved but only {age_ns}ns old, below the \
                     {staleness_threshold_ns}ns staleness threshold — refusing to recover a \
                     reservation that may still be a legitimately in-flight ceremony"
                ))
            }
        }
        Some(other) => Err(format!(
            "root {root} is not Reserved (currently {other:?}) — there is nothing to recover"
        )),
        None => Err(format!("root {root} has no anchor attempt on record — there is nothing to recover")),
    }
}

// ─── DURABLE SPENT-INPUT EXCLUSION — P0.4 (operator ruling, 2026-08-08) ────
//
// "`Broadcast` currently releases exclusivity and discards the inputs. A
// later root can therefore fetch an input already used by a broadcast-but-
// unconfirmed anchor... A Bitcoin outpoint associated with one
// constitutional root must never become selectable for another root merely
// because the first transaction has reached `Broadcast`."
//
// `Broadcast` now retains `inputs` (see the variant's docs above) precisely
// so this exclusion set can be built from durable state — the same map
// `pre_upgrade`/`post_upgrade` already persists — rather than a second,
// separately-maintained record that could drift from it.

/// Every raw Bitcoin outpoint already committed to a `Signed` or
/// `Broadcast` anchor attempt, for ANY root.
///
/// Both states are included, not just `Broadcast`: `Signed` means a valid
/// transaction spending these outpoints already exists and may still be
/// (re)broadcast at any time (see `decide_anchor_attempt`'s `Rebroadcast`
/// path) — treating it as available would risk a second root's transaction
/// confirming first and leaving the first root's now-invalid spend to fail
/// or, worse, racing it. `Reserved` and `Failed` contribute nothing: neither
/// has ever produced a transaction that spends anything.
pub fn spent_outpoints(
    attempts: &std::collections::BTreeMap<String, AnchorAttemptState>,
) -> std::collections::BTreeSet<(String, u32)> {
    attempts
        .values()
        .flat_map(|state| match state {
            AnchorAttemptState::Signed { inputs, .. } | AnchorAttemptState::Broadcast { inputs, .. } => {
                inputs.iter().map(|i| (i.txid_hex.clone(), i.vout)).collect::<Vec<_>>()
            }
            AnchorAttemptState::Reserved { .. } | AnchorAttemptState::Failed { .. } => Vec::new(),
        })
        .collect()
}

/// Exclude every candidate UTXO already committed to a `Signed`/`Broadcast`
/// attempt for ANY root (including, degenerately, the requesting root
/// itself — though `decide_anchor_attempt` never lets a fresh ceremony start
/// while its OWN root is already Signed/Broadcast, so in practice this only
/// ever excludes a DIFFERENT root's inputs), and refuse truthfully — rather
/// than silently falling back to a spent input — if nothing usable remains.
///
/// This is a REFUSAL, not a filter that merely returns empty: an empty `Ok`
/// would be indistinguishable, at the call site, from "there was genuinely
/// nothing to exclude", and the two failures call for different remedies
/// (wait for the in-flight anchor to confirm vs. fund a new address) — an
/// operator diagnosing a stuck ceremony must be told which one actually
/// happened.
pub fn select_unspent_inputs(
    candidates: Vec<AnchorInput>,
    attempts: &std::collections::BTreeMap<String, AnchorAttemptState>,
) -> Result<Vec<AnchorInput>, String> {
    let spent = spent_outpoints(attempts);
    let available: Vec<AnchorInput> =
        candidates.into_iter().filter(|c| !spent.contains(&(c.txid_hex.clone(), c.vout))).collect();
    if available.is_empty() {
        return Err(
            "ALL_UTXOS_ALREADY_RESERVED: every candidate UTXO is already committed to a Signed or \
             Broadcast anchor attempt for a different constitutional root — refusing to reuse an \
             input already spoken for rather than falling back to it"
                .to_string(),
        );
    }
    Ok(available)
}

/// Convert a millisatoshi/vB fee rate — the unit
/// `bitcoin_get_current_fee_percentiles` reports — to satoshi/vB, ROUNDING UP.
///
/// Ceiling, never floor (P0.3, operator ruling, 2026-08-08): flooring a rate
/// like 1999 msat/vB to 1 sat/vB understates what the network actually wants
/// and risks a transaction that never confirms. `.max(1)` guards only the
/// degenerate case of a zero input; `plan_anchor` independently refuses a zero
/// `fee_rate` outright, so this guard's job is to never hand it one.
pub fn msat_per_vb_to_sat_per_vb_ceil(msat_per_vb: u64) -> u64 {
    // Manual ceiling division rather than `u64::div_ceil` — this crate has no
    // MSRV pin that guarantees it, and the arithmetic is unambiguous either
    // way.
    ((msat_per_vb + 999) / 1000).max(1)
}
