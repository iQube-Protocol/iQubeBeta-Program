# CAP-1 — Live Bitcoin Testnet4 Constitutional Anchor Proof

Operator runbook. Every command below is meant to be run by the operator,
on a machine with `dfx` installed and a funded IC identity — the executing
agent's sandbox has neither `dfx` nor a cycles wallet nor testnet4 BTC, so
this runbook exists precisely to hand off the credentialed steps cleanly.

**Pinned source**: commit `a455f47d5a120bad462e6ba96a7600d35afa56b2` on
`iQube-Protocol/iQubeBeta-Program`. Every build below must be run from a
checkout at exactly this commit — `git checkout a455f47d5a120bad462e6ba96a7600d35afa56b2`
(or a branch whose tip is that commit) before building anything.

**Scope discipline**: this ceremony does NOT set `POS_LEG_SUBMISSION_ENABLED`,
apply any migration, touch `canisters/proof_of_state` (`n2hhv`), or repair
any historical receipt. It creates two NEW canisters and runs exactly one
small, real ceremony through them.

Scripts referenced below live in `scripts/cap1/` in this repo and have
each been validated against an independent, published source before being
committed (BIP-173's own bech32 test vectors; BIP-143's own worked segwit
transaction; the `pos_core` Merkle oracle already verified in Rust) — see
each script's own docstring for exactly what was checked.

---

## Step 3 — create two empty canisters, principals before code

```bash
cd iQubeBeta-Program
git checkout a455f47d5a120bad462e6ba96a7600d35afa56b2

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
sha256sum constitutional-anchor/target/wasm32-unknown-unknown/release/btc_signer_psbt.wasm
# also hash whatever dfx staged for install, if different from the raw wasm:
ls -la .dfx/ic/canisters/btc_signer_psbt/
sha256sum .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm 2>/dev/null
sha256sum .dfx/ic/canisters/btc_signer_psbt/btc_signer_psbt.wasm.gz 2>/dev/null

dfx build --network ic proof_of_state_v2
sha256sum target/wasm32-unknown-unknown/release/proof_of_state_v2.wasm
ls -la .dfx/ic/canisters/proof_of_state_v2/
sha256sum .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm 2>/dev/null
sha256sum .dfx/ic/canisters/proof_of_state_v2/proof_of_state_v2.wasm.gz 2>/dev/null
```

For reference only — NOT the A4 comparison hash (see Step 8) — a plain
`cargo build --release` from this exact commit produced, in the preflight
sandbox:

| canister | raw `cargo build` wasm sha256 |
|---|---|
| `btc_signer_psbt` | `e93d556663fffcaae7ee22b1fdd973f7a9b89c97a4550fa26d2774201bebee17` |
| `proof_of_state_v2` | `62c3e5a2bdc161a75f6ac856a0a7b040833dad736e22ddbd6716351397920820` |

**Paste back to me**: every hash `sha256sum` printed above (some `.gz`
lines may say "No such file" depending on your dfx version — that's fine,
just tell me which files existed).

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

## Step 8 — capture deployed module hashes; compare to the deployment-build hash

```bash
dfx canister --network ic info btc_signer_psbt
dfx canister --network ic info proof_of_state_v2
```

Each prints a `Module hash:` line. Compare it against the Step 4 hashes —
specifically, find WHICH Step-4 file (raw `.wasm` or `.wasm.gz`) has this
exact hash; that tells you whether your dfx version compresses on install.
**Do not assume** — check both file hashes from Step 4 against this
output.

**Paste back to me**:
- `dfx canister --network ic info` output for both canisters (full text)
- which Step-4 file matched each one

I will refuse to call `moduleHashVerifiedAgainstSource` satisfied for any
canister where this comparison doesn't come back as an exact match.

---

## Step 9 — derive the signer's testnet4 funding address OFF-CANISTER

No failed anchor call, no code change. `ecdsa_public_key` is a
public-derivation-material lookup — it can be queried for `<SIGNER_PRINCIPAL>`
by ANY caller, not only by the canister itself, using the exact
`derivation_path`/`key_id` the canister uses internally
(`DERIVATION_PATH_DEFAULT = [b"constitutional-anchor-v2"]`,
`ecdsa_key_name = "test_key_1"`):

```bash
dfx canister call aaaaa-aa ecdsa_public_key --network ic \
  '(record {
     canister_id = opt principal "<SIGNER_PRINCIPAL>";
     derivation_path = vec { blob "constitutional-anchor-v2" };
     key_id = record { curve = variant { secp256k1 }; name = "test_key_1" }
   })'
```

This returns `(record { public_key = blob "..."; chain_code = blob "..." })`.
Take the `public_key` blob, render it as hex (33 bytes / 66 hex chars), then:

```bash
python3 scripts/cap1/derive_testnet_address.py <PUBLIC_KEY_HEX>
```

This script's bech32 encoder was verified against BIP-173's own published
mainnet AND testnet test vectors before being committed (see the script's
docstring / this session's verification trail) — it is not a from-memory
reimplementation trusted on faith.

**Paste back to me**: the public key hex and the derived `tb1...` address.

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

## Step 11 — the CAP-1 ceremony (as `cap1-operator`)

Pick H. Any 64-hex-char value works; a reproducible, meaningful one for
the evidence bundle:

```bash
H=$(python3 -c "import hashlib; print(hashlib.sha256(b'CAP-1 constitutional anchor proof a455f47').hexdigest())")
echo "$H"
```

```bash
dfx canister call proof_of_state_v2 issue_receipt "(\"$H\")" --network ic --identity cap1-operator

dfx canister call proof_of_state_v2 batch_now "()" --network ic --identity cap1-operator
# capture root_hex from the returned BatchV2

dfx canister call proof_of_state_v2 request_anchor "(\"<ROOT_HEX>\")" --network ic --identity cap1-operator
# capture the returned txid
```

**Paste back to me**: H, the full `issue_receipt` response, the full
`batch_now` response (root_hex + h_hexes), the txid from `request_anchor`.

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

**12d — walk backward from the transaction to H and recompute the root**:

```bash
dfx canister call proof_of_state_v2 get_batch "(\"<ROOT_HEX>\")" --network ic --identity cap1-operator
dfx canister call proof_of_state_v2 get_receipt "(\"$H\")" --network ic --identity cap1-operator
```

From `get_receipt`'s `inclusion_proof` field, build the JSON array
`verify_inclusion_proof.py` expects (each `Sibling{hash_hex; side}` /
`Promoted` variant maps directly — see the script's docstring for the
exact shape), then:

```bash
python3 scripts/cap1/verify_inclusion_proof.py "$H" /tmp/proof.json "<ROOT_HEX>"
```

Must print `ROOT MATCHES: True`, and `<ROOT_HEX>` here must be the SAME 32
bytes `verify_op_return.py` (12b) confirmed are actually in the OP_RETURN —
this is the full loop closing: Bitcoin transaction → OP_RETURN root → PoS
`get_batch` → H → `get_receipt` → inclusion proof → recomputed root →
back to the Bitcoin bytes.

**Paste back to me**: all of 12a–12d's raw outputs (the mempool.space JSON
responses, both scripts' full stdout, and the `get_batch`/`get_receipt`
Candid responses).

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
   myself against the raw data you paste (not just read your summary).
2. Assemble the CAP-1 evidence bundle (H, leaf, proof, root, txid, raw tx,
   block hash/height, confirmation depth, both principals, both module-hash
   comparisons, source commit, deployment config, independent-verifier
   result).
3. Report PASS/FAIL.
4. Only then touch `iQube-Protocol/AigentZBeta/services/ops/canisterSourceManifest.ts`
   and `tests/canister-source-manifest.test.ts` — adding the new v2
   provenance entry distinctly (never overwriting the existing legacy
   `btc_signer_psbt`/`proof_of_state` entries), and updating
   `BITCOIN_PATH_CANISTERS` to the v2 path only if the module-hash
   comparison in Step 8 actually matched.

`POS_LEG_SUBMISSION_ENABLED` stays `false`. No migration. `n2hhv` stays
untouched. No historical receipt is repaired. Nothing beyond this one CAP-1
ceremony is activated.
