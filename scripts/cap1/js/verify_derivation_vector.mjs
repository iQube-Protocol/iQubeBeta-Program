#!/usr/bin/env node
/**
 * CAP-1 — cross-check that offline key derivation matches the canonical
 * implementation, BEFORE trusting it for a real signer principal.
 *
 * Two checks:
 *
 * 1. DFINITY's OWN published test vector, shipped inside
 *    @dfinity/ic-pub-key's test suite (dist/ecdsa/secp256k1.tests/
 *    mainnet_derivation.test.js) — reproduced here as an independent
 *    runtime check rather than trusted from reading the file. Uses the
 *    SAME master key name ("test_key_1") this deployment's InitArg uses.
 *
 * 2. A shape-matching vector using the EXACT derivation shape
 *    btc_signer_psbt actually uses — ONE extra path component, the literal
 *    bytes "constitutional-anchor-v2" — against an arbitrary canister
 *    principal (h5jwf-5iaaa-aaaan-qmvoa-cai, the same one DFINITY's own
 *    vector uses, so this reuses a principal already known to be a
 *    validly-formatted canister id). This exact value
 *    (02d33b814b589e3d9eda827960360cfec546d6ace9ca82aa15b3839be81ba73963)
 *    was independently reproduced via the canonical Rust `ic-secp256k1`
 *    crate (the crate @dfinity/ic-pub-key is itself a TypeScript port of)
 *    during CAP-1 preflight — see the runbook's Step 9 for the exact
 *    Rust commands to reproduce that cross-check yourself.
 *
 * Run this BEFORE using derive_signer_pubkey.mjs on a real signer
 * principal. Exits non-zero if either check fails.
 */
import { PublicKeyWithChainCode, DerivationPath } from '@dfinity/ic-pub-key/dist/ecdsa/secp256k1.js';
import { Principal } from '@dfinity/principal';

let allPassed = true;

function check(label, got, expected) {
  const pass = got === expected;
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${label}`);
  console.log(`  got:      ${got}`);
  console.log(`  expected: ${expected}`);
  if (!pass) allPassed = false;
}

// 1. DFINITY's own shipped test vector.
{
  const mk = PublicKeyWithChainCode.forMainnetKey('test_key_1');
  const canisterId = Principal.fromText('h5jwf-5iaaa-aaaan-qmvoa-cai');
  const path = DerivationPath.withCanisterPrefix(canisterId, [
    Buffer.from('48656C6C6F', 'hex'),
    Buffer.from('5468726573686F6C64', 'hex'),
    Buffer.from('5369676E617475726573', 'hex'),
  ]);
  const derived = mk.deriveSubkeyWithChainCode(path);
  check(
    "DFINITY's own mainnet_derivation.test.js vector (test_key_1)",
    derived.public_key.toHex(),
    '0315ae8bb8c6e9f78eec2167f5ac773067f37a39da1a1efbc585f9e90658d1c620',
  );
}

// 2. Shape-matching vector: exactly what btc_signer_psbt derives
//    (canister-id prefix + single "constitutional-anchor-v2" component),
//    cross-checked against an independent Rust ic-secp256k1 computation.
{
  const mk = PublicKeyWithChainCode.forMainnetKey('test_key_1');
  const canisterId = Principal.fromText('h5jwf-5iaaa-aaaan-qmvoa-cai');
  const path = DerivationPath.withCanisterPrefix(canisterId, [
    new TextEncoder().encode('constitutional-anchor-v2'),
  ]);
  const derived = mk.deriveSubkeyWithChainCode(path);
  check(
    'btc_signer_psbt-shaped vector, cross-checked against Rust ic-secp256k1',
    derived.public_key.toHex(),
    '02d33b814b589e3d9eda827960360cfec546d6ace9ca82aa15b3839be81ba73963',
  );
}

process.exit(allPassed ? 0 : 1);
