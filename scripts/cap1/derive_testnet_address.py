#!/usr/bin/env python3
"""CAP-1 — derive the signer's Bitcoin testnet4 P2WPKH address OFF-CANISTER.

Takes the compressed secp256k1 public key hex returned by a direct call to
the IC management canister's `ecdsa_public_key` (see the CAP-1 runbook,
step 6) and computes the SAME bech32 P2WPKH address
`btc_anchor_core::p2wpkh_address` would derive on-canister for
BtcNetwork::Testnet ("tb" bech32 HRP). This is why no failed anchor call is
needed to discover the funding address: the derivation is public math over
a public key, not a canister secret.

Usage:
    python3 derive_testnet_address.py <33-byte-compressed-pubkey-hex>
"""
import hashlib
import sys

CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"


def bech32_polymod(values):
    GEN = [0x3B6A57B2, 0x26508E6D, 0x1EA119FA, 0x3D4233DD, 0x2A1462B3]
    chk = 1
    for value in values:
        top = chk >> 25
        chk = (chk & 0x1FFFFFF) << 5 ^ value
        for i in range(5):
            chk ^= GEN[i] if ((top >> i) & 1) else 0
    return chk


def bech32_hrp_expand(hrp):
    return [ord(x) >> 5 for x in hrp] + [0] + [ord(x) & 31 for x in hrp]


def bech32_create_checksum(hrp, data):
    values = bech32_hrp_expand(hrp) + data
    polymod = bech32_polymod(values + [0, 0, 0, 0, 0, 0]) ^ 1
    return [(polymod >> 5 * (5 - i)) & 31 for i in range(6)]


def bech32_encode(hrp, data):
    combined = data + bech32_create_checksum(hrp, data)
    return hrp + "1" + "".join([CHARSET[d] for d in combined])


def convertbits(data, frombits, tobits, pad=True):
    acc = 0
    bits = 0
    ret = []
    maxv = (1 << tobits) - 1
    max_acc = (1 << (frombits + tobits - 1)) - 1
    for value in data:
        if value < 0 or (value >> frombits):
            return None
        acc = ((acc << frombits) | value) & max_acc
        bits += frombits
        while bits >= tobits:
            bits -= tobits
            ret.append((acc >> bits) & maxv)
    if pad:
        if bits:
            ret.append((acc << (tobits - bits)) & maxv)
    elif bits >= frombits or ((acc << (tobits - bits)) & maxv):
        return None
    return ret


def hash160(data: bytes) -> bytes:
    return hashlib.new("ripemd160", hashlib.sha256(data).digest()).digest()


def p2wpkh_testnet_address(pubkey_hex: str) -> str:
    pubkey = bytes.fromhex(pubkey_hex)
    if len(pubkey) != 33:
        raise ValueError(
            f"expected a 33-byte COMPRESSED secp256k1 public key, got {len(pubkey)} bytes — "
            "an uncompressed key yields a DIFFERENT hash160 and therefore a different address"
        )
    h160 = hash160(pubkey)
    return bech32_encode("tb", [0] + convertbits(list(h160), 8, 5))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        sys.exit(1)
    address = p2wpkh_testnet_address(sys.argv[1])
    print(f"pubkey (hex): {sys.argv[1]}")
    print(f"testnet4 P2WPKH address: {address}")
