#!/usr/bin/env node
/**
 * CAP-1 Step 9 — derive btc_signer_psbt's threshold-ECDSA public key OFFLINE.
 *
 * External dfx ingress CANNOT call the management canister's
 * `ecdsa_public_key` — that method only accepts calls from a canister's own
 * inter-canister-call context, not from an externally-authenticated dfx
 * identity. There is therefore no failed-anchor-call, no code change, and
 * no live IC call needed here at all: the derivation is public math over a
 * public master key, reproducible entirely offline via
 * `@dfinity/ic-pub-key` — the same TypeScript port of `ic_secp256k1` that
 * DFINITY ships and tests against real IC derivations (see
 * verify_derivation_vector.mjs, run and cross-checked against an
 * independent Rust implementation before this script was committed —
 * see the runbook for the full verification trail).
 *
 * Replicates EXACTLY what btc_signer_psbt's own `own_pubkey()` gets from
 * the real `ecdsa_public_key` management canister call:
 *   - EcdsaPublicKeyArgs.canister_id: None  => the IC implicitly prefixes
 *     the derivation path with the CALLING canister's own principal. That
 *     "calling canister" is the signer itself once deployed, so the
 *     offline equivalent explicitly supplies the signer's principal as
 *     that same prefix (DerivationPath.withCanisterPrefix).
 *   - EcdsaPublicKeyArgs.derivation_path: DERIVATION_PATH_DEFAULT
 *     = [b"constitutional-anchor-v2"]  => exactly one extra path
 *     component, the literal ASCII bytes "constitutional-anchor-v2".
 *   - EcdsaPublicKeyArgs.key_id.name: "test_key_1"  => the InitArg value
 *     installed in Step 5, so PublicKeyWithChainCode.forMainnetKey
 *     ('test_key_1') is the correct master key — NOT forPocketIcKey,
 *     which would give a completely different (local-only) key.
 *
 * Usage:
 *   node derive_signer_pubkey.mjs <SIGNER_PRINCIPAL>
 *
 * Then feed the printed public_key hex into
 * ../derive_testnet_address.py to get the funding address.
 */
import { PublicKeyWithChainCode, DerivationPath } from '@dfinity/ic-pub-key/dist/ecdsa/secp256k1.js';
import { Principal } from '@dfinity/principal';

const DERIVATION_COMPONENT = 'constitutional-anchor-v2';
const KEY_NAME = 'test_key_1';

function main() {
  const signerPrincipalText = process.argv[2];
  if (!signerPrincipalText) {
    console.error('Usage: node derive_signer_pubkey.mjs <SIGNER_PRINCIPAL>');
    process.exit(1);
  }

  const masterKey = PublicKeyWithChainCode.forMainnetKey(KEY_NAME);
  const signerPrincipal = Principal.fromText(signerPrincipalText);
  const path = DerivationPath.withCanisterPrefix(signerPrincipal, [
    new TextEncoder().encode(DERIVATION_COMPONENT),
  ]);
  const derived = masterKey.deriveSubkeyWithChainCode(path);
  const { public_key, chain_code } = derived.toHex();

  console.log(`signer principal:      ${signerPrincipalText}`);
  console.log(`master key:            ${KEY_NAME}`);
  console.log(`derivation component:  "${DERIVATION_COMPONENT}"`);
  console.log(`derived public_key:    ${public_key}`);
  console.log(`derived chain_code:    ${chain_code}`);
  console.log('');
  console.log('Next: python3 ../derive_testnet_address.py ' + public_key);
}

main();
