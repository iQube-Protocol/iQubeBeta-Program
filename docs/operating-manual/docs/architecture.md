---
id: architecture
title: iQube Architecture Overview
sidebar_position: 1
---

Welcome to the iQube Operators Manual. This document provides a high-level architecture overview of the iQube Program.

## Core Components

- iQube Types: MetaQube, BlakQube, TokenQube
- Frontend Apps: Ops Console, Aigent Z, 21 Sats Market
- ICP Canisters: proof_of_state, btc_signer_psbt, cross_chain_service, evm_rpc
- EVM Integrations: Ethereum Sepolia, Polygon Amoy

```mermaid
flowchart LR
  A[iQube Frontends]\n/apps/ops-console\n/apps/aigent-z --- B{Next.js API}
  B --> C[Supabase]
  B --> D[ICP Canisters]
  D --> D1[proof_of_state]
  D --> D2[btc_signer_psbt]
  D --> D3[cross_chain_service]
  D --> D4[evm_rpc]
  B --> E[EVM RPCs]\nSepolia/Amoy
```

## Data Flows

- Ops Console polls Next.js API routes for live testnet status
- Aigent Z reads/writes Registry data via API (Supabase backend)
- Canisters provide cross-chain capabilities (anchoring, DVN, EVM RPC)

