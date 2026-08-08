#!/usr/bin/env python3
"""CAP-1 — independent Merkle inclusion-proof recomputation.

Reimplements the normative pipeline from scratch, in Python, from the
published spec alone — NOT by calling PoS v2's own `verify_receipt` query
(which would trust pos_core's own code to grade itself):

    leaf = SHA256(0x00 || H)
    parent = SHA256(0x01 || left || right)

Feed it H and the `inclusion_proof` array PoS v2's `get_receipt(H)` query
returned (paste the Candid response, converted to the JSON shape below),
and the root it recomputes must equal the 32 bytes actually committed in
the Bitcoin OP_RETURN output (cross-check against verify_op_return.py's
output).

Input JSON shape (one array element per ProofStep, in order):
    [
      {"Sibling": {"hash_hex": "<64-hex>", "side": "Left"}},
      {"Promoted": null},
      {"Sibling": {"hash_hex": "<64-hex>", "side": "Right"}}
    ]

Usage:
    python3 verify_inclusion_proof.py <H-hex> <proof.json> <expected-root-hex>
"""
import hashlib
import json
import sys


def sha256(b: bytes) -> bytes:
    return hashlib.sha256(b).digest()


def leaf_hash(h_hex: str) -> str:
    return sha256(bytes.fromhex("00") + bytes.fromhex(h_hex)).hex()


def parent_hash(left_hex: str, right_hex: str) -> str:
    return sha256(bytes.fromhex("01") + bytes.fromhex(left_hex) + bytes.fromhex(right_hex)).hex()


def recompute_root(h_hex: str, proof_steps: list) -> str:
    cur = leaf_hash(h_hex)
    for step in proof_steps:
        if "Sibling" in step:
            sib_hex = step["Sibling"]["hash_hex"]
            side = step["Sibling"]["side"]
            if side == "Left":
                cur = parent_hash(sib_hex, cur)
            elif side == "Right":
                cur = parent_hash(cur, sib_hex)
            else:
                raise ValueError(f"unknown side {side!r} — expected 'Left' or 'Right'")
        elif "Promoted" in step:
            pass  # no sibling at this level; the hash passes through unchanged
        else:
            raise ValueError(f"unrecognised proof step {step!r}")
    return cur


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print(__doc__)
        sys.exit(1)
    h_hex, proof_path, expected_root_hex = sys.argv[1], sys.argv[2], sys.argv[3]
    with open(proof_path) as f:
        proof_steps = json.load(f)

    computed_leaf = leaf_hash(h_hex)
    computed_root = recompute_root(h_hex, proof_steps)
    matches = computed_root == expected_root_hex.lower()

    print(f"H:                 {h_hex}")
    print(f"computed leaf:     {computed_leaf}")
    print(f"proof steps:       {len(proof_steps)}")
    print(f"recomputed root:   {computed_root}")
    print(f"expected root:     {expected_root_hex}")
    print(f"ROOT MATCHES:      {matches}")
    sys.exit(0 if matches else 1)
