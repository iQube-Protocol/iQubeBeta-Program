# CAP-1 — Live Bitcoin Testnet4 Constitutional Anchor Proof

Operator runbook. Every command below is meant to be run by the operator,
on a machine with `dfx` installed and a funded IC identity — the executing
agent's sandbox has neither `dfx` nor a cycles wallet nor testnet4 BTC, so
this runbook exists precisely to hand off the credentialed steps cleanly.

**Pinned source**: commit `32fff4b7d5df82d9e8f1658dc9cf255e829bcab1` is the
FLOOR — it is the first commit to carry `proof_of_state_v2`'s `dfx.json`
registration. It is NOT sufficient by itself: `a455f47d5a120bad462e6ba96a7600d35afa56b2`
(the last PoS v2 *code* commit) predates it and must never be used as the
runbook checkout, because it lacks the dfx registration and all CAP-1
tooling entirely. This correction (the commit that introduced this exact
paragraph, the corrected Step 9, the split A4 claims, the 3-receipt
ceremony, and portable hashing) supersedes `32fff4b` as the actual pin.
Confirm the exact hash before building:

```bash
git log -1 --format=%H -- docs/runbooks/cap1-testnet-anchor-proof.md scripts/cap1 dfx.json
```

Use THAT hash for `git checkout` below — do not hardcode a hash from this
paragraph once a newer correction has landed.

**Scope discipline**: this ceremony does NOT set `POS_LEG_SUBMISSION_ENABLED`,
apply any migration, touch `canisters/proof_of_state` (`n2hhv`), or repair
any historical receipt. It creates two NEW canisters and runs exactly one
small, real ceremony through them.

Scripts referenced below live in `scripts/cap1/` in this repo and have
each been validated against an independent, published source before being
committed (BIP-173's own bech32 test vectors; BIP-143's own worked segwit
transaction; the `pos_core` Merkle oracle already verified in Rust; the
offline key-derivation vectors described in Step 9) — see each script's
own docstring for exactly what was checked.

**Portable hashing** — macOS ships `shasum -a 256`, not GNU `sha256sum`.
Define this once per shell session and use `sha256_file <path>` everywhere
below instead of calling either tool directly:

```bash
sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}
```

---

## Step 3 — create two empty canisters, principals before code

```bash
cd iQubeBeta-Program
git checkout <PIN_HASH>   # see "Pinned source" above

dfx canister create --network ic btc_signer_psbt
dfx canister create --network ic proof_of_state_v2

dfx canister id --network ic btc_signer_psbt
dfx canister id --network ic proof_of_state_v2
```

**Paste back to me**: both principals, labelled which is which.

Requires cycles on your identity (`dfx cycles balance --network ic` to
check; top up via your existing cycles wallet or `dfx cycles convert` from
ICP if needed — that step is yours, it moves real funds).

---

## Step 4 — build via the deployment pipeline, hash BEFORE install

This is the artifact whose hash you compare against the *deployed* module
hash later — not a bare `cargo build` (dfx may gzip / add metadata; see
Step 8).

```bash
dfx build --network ic btc_signer_psbt
sha256_file constitutional-anchor/target/wasm32-unknown-unknown/release/btc_signer_psbt.wasm
# also hash whatever dfx staged for install, if different from the raw wasm:
ls -la .dfx/ic/canisters/btc_signer_psbt/
[ -f .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm ] && sha256_file .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm
[ -f .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm.gz ] && sha256_file .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm.gz

dfx build --network ic proof_of_state_v2
sha256_file target/wasm32-unknown-unknown/release/proof_of_state_v2.wasm
ls -la .dfx/ic/canisters/proof_of_state_v2/
[ -f .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm ] && sha256_file .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm
[ -f .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm.gz ] && sha256_file .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm.gz
```

For reference only — NOT the A4 comparison hash (see Step 8) — a plain
`cargo build --release` from commit `a455f47` produced, in the preflight
sandbox:

| canister | raw `cargo build` wasm sha256 |
|---|---|
| `btc_signer_psbt` | `e93d556663fffcaae7ee22b1fdd973f7a9b89c97a4550fa26d2774201bebee17` |
| `proof_of_state_v2` | `62c3e5a2bdc161a75f6ac856a0a7b040833dad736e22ddbd6716351397920820` |

These are stale relative to the corrected pin (they predate `dfx.json`'s
registration) and are kept here only as a sanity floor, never as the A4
comparison value.

**Paste back to me**: every hash printed above (some `.gz` lines may be
skipped depending on your dfx version — that's fine, just tell me which
files existed).

---

## Step 5 — install Constitutional Anchor v2 (`btc_signer_psbt`)

Substitute `<POS_V2_PRINCIPAL>` with the principal from Step 3.

```bash
dfx canister install --network ic btc_signer_psbt --mode install --argument \
  '(record {
     network = "testnet";
     ecdsa_key_name = "test_key_1";
     authorized_pos_principal = principal "<POS_V2_PRINCIPAL>"
   })'
```

`network = "testnet"` is Bitcoin **testnet4** on the current IC Bitcoin
integration — confirmed directly from the `ic-btc-interface` crate source
(`ic-btc-interface-0.4.0/src/lib.rs`): the `Network::Testnet` variant's own
doc comment reads `/// Bitcoin Testnet4.`, not testnet3. This is not an
assumption; it is read from the dependency this canister actually links.

---

## Step 6 — create two SEPARATE dfx identities for operator and reconciler

The reconciler MUST be a different principal from the operator (P1's role
separation). If you only have one dfx identity today:

```bash
dfx identity new cap1-operator
dfx identity new cap1-reconciler

dfx identity get-principal --identity cap1-operator
dfx identity get-principal --identity cap1-reconciler
```

**Paste back to me**: both principals.

---

## Step 7 — install PoS v2 (`proof_of_state_v2`)

Substitute the three principals (signer from Step 3, operator + reconciler
from Step 6). `min_confirmations = 1` per the instruction — this is the
FIRST CAP-1 proof, not a finality-depth test.

```bash
dfx canister install --network ic proof_of_state_v2 --mode install --argument \
  '(record {
     anchor_signer_principal = principal "<SIGNER_PRINCIPAL>";
     authorized_operator_principal = principal "<CAP1_OPERATOR_PRINCIPAL>";
     authorized_reconciler_principal = principal "<CAP1_RECONCILER_PRINCIPAL>";
     min_confirmations = 1 : nat32
   })'
```

---

## Step 8 — TWO SEPARATE A4 claims, not one

The A4 manifest (`services/ops/canisterSourceManifest.ts` in AigentZBeta)
distinguishes two different, non-substitutable facts. Do not conflate them:

- **`deploymentArtifactHashVerified`** — the artifact you staged locally in
  Step 4 (whichever file dfx actually sent — raw `.wasm` or `.wasm.gz`,
  determined below) has the SAME sha256 as what the IC reports installed.
  This is achievable RIGHT NOW, in this same session, and proves only that
  nothing was corrupted or substituted between your build and the install
  call.
- **`moduleHashVerifiedAgainstSource`** — a SEPARATE, independently
  reproducible build of the SAME pinned commit, from a clean environment
  (a fresh clone, `cargo clean`, or a different machine — anything that
  rules out "my local cache happened to produce this"), ALSO produces that
  exact artifact hash. This is the actual provenance claim and requires
  the rebuild in Step 8b below — it is NEVER satisfied by Step 8a alone.

**8a — deploymentArtifactHashVerified:**

```bash
dfx canister --network ic info btc_signer_psbt
dfx canister --network ic info proof_of_state_v2
```

Each prints a `Module hash:` line. Compare it against the Step 4 hashes —
find WHICH Step-4 file (raw `.wasm` or `.wasm.gz`) has this exact hash;
that tells you whether your dfx version compresses on install. **Do not
assume** — check both file hashes from Step 4 against this output.

**Paste back to me**: the `dfx canister --network ic info` output for both
canisters (full text), and which Step-4 file matched each one.

**8b — moduleHashVerifiedAgainstSource (do this separately, before or after
the ceremony — it does not block Steps 9-13):**

```bash
# In a DIFFERENT directory (or after `cargo clean` in this one):
git clone https://github.com/iQube-Protocol/iQubeBeta-Program.git cap1-rebuild-check
cd cap1-rebuild-check
git checkout <PIN_HASH>
dfx build --network ic btc_signer_psbt      # network context only selects build flags; no install happens
sha256_file constitutional-anchor/target/wasm32-unknown-unknown/release/btc_signer_psbt.wasm
dfx build --network ic proof_of_state_v2
sha256_file target/wasm32-unknown-unknown/release/proof_of_state_v2.wasm
```

Compare these two hashes against the Step 8a deployed module hashes
(matching the SAME raw-vs-gz artifact you identified in 8a — if dfx
compresses, gzip the freshly rebuilt `.wasm` the same way before comparing,
or compare the two raw `.wasm` hashes against each other AND separately
confirm the gzip step is deterministic).

**Paste back to me**: both rebuild hashes, and an explicit statement of
whether each matches its corresponding Step 8a deployed hash.

I will not report `moduleHashVerifiedAgainstSource: true` for a canister
where 8b was skipped or did not match, even if 8a matched.

---

## Step 9 — derive the signer's testnet4 funding public key and address, OFFLINE

**Correction**: external `dfx` ingress CANNOT call the management
canister's `ecdsa_public_key` — that method only accepts calls arriving
from another canister's inter-canister-call context, not from an
externally-authenticated dfx identity's ingress message. There is no
management-canister call to make here at all.

Instead, the derivation is reproduced entirely OFFLINE using
[`@dfinity/ic-pub-key`](https://www.npmjs.com/package/@dfinity/ic-pub-key)
— DFINITY's own TypeScript port of the `ic_secp256k1` crate the IC replica
itself uses for this derivation. No secret is involved in deriving a
public key from a public master key, so this needs no live call, no
failed-anchor discovery, and no code change.

**9a — install and cross-check the tooling BEFORE trusting it for a real principal:**

```bash
cd scripts/cap1/js
npm install
node verify_derivation_vector.mjs
```

Must print `PASS` for both checks. The first reproduces DFINITY's own
published test vector (shipped inside `@dfinity/ic-pub-key`'s test suite);
the second reproduces the EXACT derivation shape `btc_signer_psbt` uses
(canister-principal prefix + the single literal component
`"constitutional-anchor-v2"`) against a value independently cross-checked
during CAP-1 preflight via the canonical Rust `ic_secp256k1` crate — see
below to reproduce that cross-check yourself:

```bash
# Optional but recommended: independent Rust-side reproduction of the
# SAME shape-matching vector verify_derivation_vector.mjs checks (npm
# install may warn about Node >=24; @dfinity/ic-pub-key ran successfully
# on Node 22.22.2 during CAP-1 preflight despite that declared minimum).
mkdir -p /tmp/cap1-rust-check/src && cd /tmp/cap1-rust-check
cat > Cargo.toml <<'EOF'
[package]
name = "cap1-rust-check"
version = "0.1.0"
edition = "2021"
[dependencies]
ic-secp256k1 = "0.1"
hex = "0.4"
EOF
cat > src/main.rs <<'EOF'
use ic_secp256k1::{DerivationPath, PublicKey};
fn main() {
    let master = PublicKey::deserialize_sec1(
        &hex::decode("02f9ac345f6be6db51e1c5612cddb59e72c3d0d493c994d12035cf13257e3b1fa7").unwrap()
    ).unwrap();
    let canister_id = hex::decode("0000000001b0655c0101").unwrap(); // h5jwf-5iaaa-aaaan-qmvoa-cai
    let path = DerivationPath::from_canister_id_and_path(
        &canister_id, &[b"constitutional-anchor-v2".to_vec()]
    );
    let (derived, _cc) = master.derive_subkey_with_chain_code(&path, &[0u8; 32]);
    println!("{}", hex::encode(derived.serialize_sec1(true)));
}
EOF
cargo run --quiet
# expected: 02d33b814b589e3d9eda827960360cfec546d6ace9ca82aa15b3839be81ba73963
```

**9b — derive the REAL signer's public key**, substituting `<SIGNER_PRINCIPAL>`
from Step 3:

```bash
node derive_signer_pubkey.mjs <SIGNER_PRINCIPAL>
```

This reproduces exactly what `btc_signer_psbt::own_pubkey()` gets from the
real `ecdsa_public_key` call: `canister_id: None` in the canister's own
Candid args means the IC implicitly prefixes the derivation path with the
CALLING canister's own principal (here, the signer, once deployed) —
`derive_signer_pubkey.mjs` supplies that prefix explicitly via
`DerivationPath.withCanisterPrefix(signerPrincipal, ...)`, and appends the
single component `"constitutional-anchor-v2"`
(`DERIVATION_PATH_DEFAULT` in the Rust source), under the `"test_key_1"`
master key (matching the `ecdsa_key_name` installed in Step 5).

**9c — derive the P2WPKH address:**

```bash
cd ../
python3 derive_testnet_address.py <PUBLIC_KEY_HEX_FROM_9b>
```

This script's bech32 encoder was verified against BIP-173's own published
mainnet AND testnet test vectors before being committed.

**Paste back to me**: the 9a PASS/PASS output (and the Rust cross-check
output if you ran it), the 9b public key + chain code, and the 9c derived
`tb1...` address.

---

## Step 10 — fund the address; wait for ≥1 confirmation

Use a Bitcoin testnet4 faucet (this step moves real, if worthless, funds
and needs a human — I cannot do it from this sandbox). Then confirm
funding independently:

```bash
curl -sS https://mempool.space/testnet4/api/address/<TB1_ADDRESS>/utxo
```

Wait until the returned UTXO's `status.confirmed` is `true` (≥1
confirmation, matching `min_confirmations = 1`).

**Paste back to me**: the funding txid, vout, and the UTXO JSON showing
`confirmed: true`.

---

## Step 11 — the CAP-1 ceremony: THREE receipts, ONE batch, ONE anchor (as `cap1-operator`)

Three deterministic H values, so the batch actually exercises Merkle
combination (not a trivial single-leaf tree) and includes one PROMOTED
(odd) leaf:

```bash
H1=$(python3 -c "import hashlib; print(hashlib.sha256(b'CAP-1 constitutional anchor proof 2026-08-08 #1').hexdigest())")
H2=$(python3 -c "import hashlib; print(hashlib.sha256(b'CAP-1 constitutional anchor proof 2026-08-08 #2').hexdigest())")
H3=$(python3 -c "import hashlib; print(hashlib.sha256(b'CAP-1 constitutional anchor proof 2026-08-08 #3').hexdigest())")
echo "H1=$H1"
echo "H2=$H2"
echo "H3=$H3"
```

```bash
dfx canister call proof_of_state_v2 issue_receipt "(\"$H1\")" --network ic --identity cap1-operator
dfx canister call proof_of_state_v2 issue_receipt "(\"$H2\")" --network ic --identity cap1-operator
dfx canister call proof_of_state_v2 issue_receipt "(\"$H3\")" --network ic --identity cap1-operator

dfx canister call proof_of_state_v2 batch_now "()" --network ic --identity cap1-operator
# capture root_hex; confirm h_hexes == [H1, H2, H3] in that exact order
# (PENDING is FIFO by issuance — pos_core's 3-leaf tree makes H1,H2 an
# ordinary pair and PROMOTES H3, the odd one, unchanged to the next layer)

dfx canister call proof_of_state_v2 request_anchor "(\"<ROOT_HEX>\")" --network ic --identity cap1-operator
# capture the returned txid — this is ONE anchor call for the WHOLE batch
```

**Paste back to me**: H1, H2, H3, the full `issue_receipt` responses, the
full `batch_now` response (root_hex + h_hexes in order), the txid from
`request_anchor`.

---

## Step 12 — independent verification from the Bitcoin transaction, backward

**12a — fetch the raw transaction from an independent observer** (not PoS,
not the signer):

```bash
curl -sS https://mempool.space/testnet4/api/tx/<TXID>/hex > /tmp/rawtx.hex
curl -sS https://mempool.space/testnet4/api/tx/<TXID>
```

**12b — recompute the txid and verify the OP_RETURN, byte-for-byte**:

```bash
python3 scripts/cap1/verify_op_return.py "$(cat /tmp/rawtx.hex)" "<ROOT_HEX>"
```

Must print `op_return_matches_expected: True` and a `computed_txid` equal
to `<TXID>`. If you have Bitcoin Core built with testnet4 support,
cross-check independently against the reference implementation too:

```bash
bitcoin-cli -testnet4 decoderawtransaction "$(cat /tmp/rawtx.hex)"
```

**12c — block height / confirmation depth, from Bitcoin only**:

```bash
curl -sS https://mempool.space/testnet4/api/tx/<TXID>/status
curl -sS https://mempool.space/testnet4/api/blocks/tip/height
```

`status.block_height` and `status.block_hash` are the Bitcoin evidence.
`confirmations = tip_height - block_height + 1`. Neither PoS nor the
signer's own self-report may substitute for these two curl calls.

**12d — walk backward from the transaction to H1 (the target) AND H3 (the
promoted leaf), and recompute the root for both:**

```bash
dfx canister call proof_of_state_v2 get_batch "(\"<ROOT_HEX>\")" --network ic --identity cap1-operator
dfx canister call proof_of_state_v2 get_receipt "(\"$H1\")" --network ic --identity cap1-operator
dfx canister call proof_of_state_v2 get_receipt "(\"$H3\")" --network ic --identity cap1-operator
```

`get_receipt("$H1")`'s `inclusion_proof` must be NON-EMPTY (two
`Sibling` steps for a 3-leaf tree). `get_receipt("$H3")`'s `inclusion_proof`
must contain a `Promoted` step — H3 is the odd leaf a 3-element tree
promotes unchanged at the leaf layer.

From each `inclusion_proof` field, build the JSON array
`verify_inclusion_proof.py` expects (each `Sibling{hash_hex; side}` /
`Promoted` variant maps directly — see the script's docstring for the
exact shape), then:

```bash
python3 scripts/cap1/verify_inclusion_proof.py "$H1" /tmp/proof_h1.json "<ROOT_HEX>"
python3 scripts/cap1/verify_inclusion_proof.py "$H3" /tmp/proof_h3.json "<ROOT_HEX>"
```

Both must print `ROOT MATCHES: True`, and `<ROOT_HEX>` here must be the
SAME 32 bytes `verify_op_return.py` (12b) confirmed are actually in the
OP_RETURN — this is the full loop closing: Bitcoin transaction →
OP_RETURN root → PoS `get_batch` → H → `get_receipt` → inclusion proof →
recomputed root → back to the Bitcoin bytes, for both a normal leaf and
the promoted one.

**Paste back to me**: all of 12a–12d's raw outputs (the mempool.space JSON
responses, both scripts' full stdout for both H1 and H3, and the
`get_batch`/`get_receipt` Candid responses).

---

## Step 13 — record_confirmation, as the SEPARATE reconciler, only after 12 passes

```bash
dfx canister call proof_of_state_v2 record_confirmation \
  "(\"<ROOT_HEX>\", \"<TXID>\", <BLOCK_HEIGHT> : nat64, <CONFIRMATIONS> : nat32)" \
  --network ic --identity cap1-reconciler
```

Must return `Anchored { txid = "<TXID>"; block_height = <N>; confirmations = <N> }`.
Confirm with:

```bash
dfx canister call proof_of_state_v2 get_batch "(\"<ROOT_HEX>\")" --network ic --identity cap1-operator
```

**Paste back to me**: the `record_confirmation` response and the follow-up
`get_batch` response.

---

## What happens after you paste all of this back

I will:
1. Independently re-run `verify_op_return.py` and `verify_inclusion_proof.py`
   (for both H1 and H3) myself against the raw data you paste — not just
   read your summary.
2. Assemble the CAP-1 evidence bundle (H1/H2/H3, leaves, both proofs, root,
   txid, raw tx, block hash/height, confirmation depth, both principals,
   `deploymentArtifactHashVerified` AND `moduleHashVerifiedAgainstSource`
   recorded as the two SEPARATE claims Step 8 produced, source commit,
   deployment config, independent-verifier result).
3. Report PASS/FAIL.
4. Only then touch `iQube-Protocol/AigentZBeta/services/ops/canisterSourceManifest.ts`
   and `tests/canister-source-manifest.test.ts` — adding the new v2
   provenance entry distinctly (never overwriting the existing legacy
   `btc_signer_psbt`/`proof_of_state` entries), setting
   `moduleHashVerifiedAgainstSource` ONLY if Step 8b's independent rebuild
   actually matched (never from Step 8a alone), and updating
   `BITCOIN_PATH_CANISTERS` to the v2 path only if that stronger claim
   holds for both canisters.

`POS_LEG_SUBMISSION_ENABLED` stays `false`. No migration. `n2hhv` stays
untouched. No historical receipt is repaired. Nothing beyond this one CAP-1
ceremony is activated.
