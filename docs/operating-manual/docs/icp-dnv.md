---
id: icp-dnv
title: ICP Cross-Chain DVN
sidebar_position: 2
---

The `cross_chain_service` canister provides LayerZero DVN quorum verification and cross-chain messaging utilities.

## Canister Overview

- ID: `u6s2n-gx777-77774-qaaba-cai`
- Key APIs:
  - `submit_dvn_message`
  - `get_dvn_message`
  - `submit_attestation`
  - `get_message_attestations`
  - `monitor_evm_transaction`
  - `verify_layerzero_message`

```mermaid
sequenceDiagram
  participant App as Ops Console
  participant API as Next.js API
  participant DVN as cross_chain_service
  App->>API: /api/ops/dvn/status
  API->>DVN: get_message_attestations()
  DVN-->>API: attestations, status
  API-->>App: ok, counts, last update
```

## Operations

- Quorum: 2 attestations required for execution
- Monitoring: DVN status surfaced in Ops Console DVN card
- Errors: network failures return `ok: false` with detailed message

