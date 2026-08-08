#!/usr/bin/env python3
"""CAP-1 — independent txid + OP_RETURN verifier.

Parses a RAW Bitcoin transaction hex byte-for-byte (no bitcoin library,
nothing that trusts PoS or the signer's own code) and:
  1. recomputes the txid as sha256d(witness-stripped serialisation),
     reversed — the same rule btc_anchor_core::compute_txid implements,
     verified here independently in a different language;
  2. finds the transaction's OP_RETURN output and checks its script is
     EXACTLY `6a20 || root_bytes` (OP_RETURN OP_PUSHBYTES_32 <32-byte root>).

Parser slicing was verified against BIP-143's own worked two-input segwit
example (https://github.com/bitcoin/bips/blob/master/bip-0143.mediawiki) —
the witness-stripped legacy serialisation this script reconstructs matched
that spec's own annotated field breakdown byte-for-byte before this script
was written.

Usage:
    python3 verify_op_return.py <raw-tx-hex> <expected-root-hex>

Get <raw-tx-hex> from an independent source (e.g. mempool.space testnet4:
`curl https://mempool.space/testnet4/api/tx/<txid>/hex`), NOT from the PoS
or signer canister.
"""
import hashlib
import sys


def read_varint(b: bytes, i: int):
    n = b[i]
    if n < 0xFD:
        return n, i + 1
    elif n == 0xFD:
        return int.from_bytes(b[i + 1 : i + 3], "little"), i + 3
    elif n == 0xFE:
        return int.from_bytes(b[i + 1 : i + 5], "little"), i + 5
    else:
        return int.from_bytes(b[i + 1 : i + 9], "little"), i + 9


def parse_and_verify(raw_hex: str, expected_root_hex: str):
    b = bytes.fromhex(raw_hex)
    i = 0
    version = b[i : i + 4]
    i += 4

    segwit = b[i] == 0x00 and b[i + 1] == 0x01
    if segwit:
        i += 2  # marker + flag — NOT part of the legacy (no-witness) serialisation

    start_inputs = i
    n_in, i = read_varint(b, i)
    for _ in range(n_in):
        i += 36  # prevout: 32-byte txid (internal order) + 4-byte vout
        script_len, i2 = read_varint(b, i)
        i = i2 + script_len
        i += 4  # sequence
    end_inputs = i

    start_outputs = i
    n_out, i = read_varint(b, i)
    outputs = []
    for _ in range(n_out):
        value = int.from_bytes(b[i : i + 8], "little")
        i += 8
        script_len, i2 = read_varint(b, i)
        script = b[i2 : i2 + script_len]
        outputs.append((value, script))
        i = i2 + script_len
    end_outputs = i

    if segwit:
        for _ in range(n_in):
            n_items, i = read_varint(b, i)
            for _ in range(n_items):
                item_len, i2 = read_varint(b, i)
                i = i2 + item_len

    locktime = b[i : i + 4]
    i += 4

    if i != len(b):
        raise AssertionError(
            f"parser did not consume the whole transaction ({i} of {len(b)} bytes) — "
            "the raw hex may be malformed, or this parser has a bug; do not trust the result"
        )

    legacy = version + b[start_inputs:end_inputs] + b[start_outputs:end_outputs] + locktime
    txid = hashlib.sha256(hashlib.sha256(legacy).digest()).digest()[::-1].hex()

    op_return_outputs = [(idx, s) for idx, (_, s) in enumerate(outputs) if s[:1] == b"\x6a"]
    expected_script = bytes.fromhex("6a20" + expected_root_hex)

    result = {
        "computed_txid": txid,
        "num_inputs": n_in,
        "num_outputs": n_out,
        "op_return_outputs_found": len(op_return_outputs),
        "op_return_matches_expected": False,
    }

    if len(op_return_outputs) != 1:
        result["error"] = f"expected exactly one OP_RETURN output, found {len(op_return_outputs)}"
        return result

    idx, op_return_script = op_return_outputs[0]
    result["op_return_output_index"] = idx
    result["op_return_script_hex"] = op_return_script.hex()
    result["expected_script_hex"] = expected_script.hex()
    result["op_return_matches_expected"] = op_return_script == expected_script
    return result


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    result = parse_and_verify(sys.argv[1], sys.argv[2])
    for k, v in result.items():
        print(f"{k}: {v}")
    if not result.get("op_return_matches_expected"):
        sys.exit(1)
