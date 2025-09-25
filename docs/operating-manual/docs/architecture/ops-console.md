---
id: architecture/ops-console
title: Ops Console Architecture
sidebar_label: Ops Console
---

import DocCardList from '@theme/DocCardList';

# Ops Console Architecture

This page mirrors and summarizes the architecture documented in the repository at `docs/architecture/ops-console.md`.

For full context, see:
- `apps/aigent-z/app/ops/page.tsx`
- `apps/aigent-z/app/api/ops/*`
- `apps/aigent-z/hooks/ops/*`

## Overview

The Ops Console integrates ICP canisters and EVM/Non‑EVM networks, exposing diagnostics, reconciliation tools, and test flows.

Core components:
- DVN (`cross_chain_service`)
- Proof‑of‑State (`proof_of_state`)
- EVM RPC (`evm_rpc`)
- BTC Signer (`btc_signer_psbt`)
- Solana Signer (`solana_signer_ed25519`)

## Synchronization Architecture

- Sync Status API detects drift and reports severity.
- Auto‑Repair API balances state when safe.
- LayerZero Processing API processes DVN messages and submits attestations.

## Checks and Balances

- Idempotent DVN monitor to prevent duplicates.
- Lazy EVM chain initialization.
- Post‑processing refresh for eventual consistency.

## Key References

- Cross‑Chain Status: `app/api/ops/crosschain/status/route.ts`
- DVN Monitor: `app/api/ops/dvn/monitor/route.ts`
- LayerZero Processing: `app/api/ops/layerzero/process/route.ts`
- Sync: `app/api/ops/sync/*`
- Health: `app/api/ops/icp/health/route.ts`

<DocCardList />
