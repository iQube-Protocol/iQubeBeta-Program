---
id: anchoring
title: Bitcoin Anchoring (ICP/BTC)
sidebar_position: 3
---

End-to-end ICP/BTC anchoring using `proof_of_state` and `btc_signer_psbt` canisters.

## Components

- `proof_of_state` (ulvla-h7777-77774-qaacq-cai)
- `btc_signer_psbt` (uxrrr-q7777-77774-qaaaq-cai)

## Flow

```mermaid
sequenceDiagram
  participant App as Ops Console
  participant PoS as proof_of_state
  participant BTC as btc_signer_psbt
  App->>PoS: issue_receipt()
  PoS-->>App: receipt_id
  App->>PoS: batch()
  App->>PoS: anchor()
  PoS->>BTC: create_and_broadcast_anchor()
  BTC-->>PoS: txid
  PoS-->>App: get_batches() with txid
```

## Notes

- Real Bitcoin testnet HTTP outcalls via Blockstream API
- Asynchronous anchoring with fallback to mock on failure

