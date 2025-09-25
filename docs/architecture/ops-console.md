---
id: ops-console
title: Ops Console Architecture
sidebar_label: Ops Console
slug: /architecture/ops-console
---

# Overview

The Ops Console provides real-time visibility and control over the protocol’s cross-chain health. It integrates Internet Computer (ICP) canisters and EVM/Non‑EVM networks, exposing diagnostics, reconciliation tools, and end‑to‑end testing flows.

Core components:
- ICP Canisters: `cross_chain_service` (DVN), `proof_of_state` (PoS), `evm_rpc`, `btc_signer_psbt`, `solana_signer_ed25519`
- Web App: `apps/aigent-z` Next.js application
- API routes: under `apps/aigent-z/app/api/ops/*` and `apps/aigent-z/app/api/core/*`
- Hooks: under `apps/aigent-z/hooks/ops/*`


# Data Model and Responsibilities

- **DVN (cross_chain_service)**
  - Tracks cross‑chain messages, attestation quorum, LayerZero integration.
  - Methods: `submit_dvn_message`, `get_dvn_message`, `monitor_evm_transaction`, `submit_attestation`, `verify_layerzero_message`, `get_pending_messages`.

- **Proof of State (proof_of_state)**
  - Anchors receipts into Bitcoin (via `btc_signer_psbt`), batches receipts and anchors Merkle roots.
  - Methods: `issue_receipt`, `get_pending_count`, `batch`, `batch_now`, `get_batches`, `anchor`.

- **EVM RPC (evm_rpc)**
  - Read-only access to multiple EVM networks, with cached receipts and blocks.
  - Methods: `init_chain_configs`, `get_supported_chains`, `get_transaction_receipt`, `get_latest_block_number`, etc.

- **BTC Signer (btc_signer_psbt)**
  - Creates and broadcasts Bitcoin testnet anchors via HTTP outcalls.


# Synchronization Architecture

DVN messages and PoS receipts have distinct lifecycles. The system provides continuous reconciliation to validate overall integrity:

- Sync Status API: `apps/aigent-z/app/api/ops/sync/status/route.ts`
  - Computes drift: DVN pending vs PoS pending/batched.
  - Reports severity: info, warning, critical, with recommendations.

- Auto-Repair API: `apps/aigent-z/app/api/ops/sync/repair/route.ts`
  - Strategy: create missing entries in the deficit canister when safe.
  - Logs before/after state and actions performed.

- LayerZero Processing API: `apps/aigent-z/app/api/ops/layerzero/process/route.ts`
  - Processes all pending DVN messages, submits validator attestations, optional verification.
  - Returns per‑message results and counts (processed/total).


# Ops Console UI (apps/aigent-z/app/ops/page.tsx)

- Cards: Cross‑Chain Status, Canister Sync Status, ICP DVN, BTC Testnet, EVM chains (Sepolia, Amoy, Optimism Sepolia, Arbitrum Sepolia, Base Sepolia).
- DVN Mint Tests:
  - Abbreviated chain dropdown.
  - Compact controls: `Test TX`, `Monitor`, `Use last`, `Clear`.
  - Idempotent monitor to prevent duplicate DVN entries.
- Health Indicator:
  - Inline chip shows “Local ICP connected” when `127.0.0.1:4943` is reachable.


# Checks and Balances

- Idempotent Monitoring
  - API `ops/dvn/monitor` inspects `get_pending_messages()` and returns existing `messageId` when the same `txHash` is already pending.

- Lazy Chain Initialization
  - `ops/crosschain/status` calls `evm.init_chain_configs()` when `get_supported_chains()` returns empty (first‑run convenience).

- Post‑Processing Refresh
  - After LayerZero processing, UI refreshes DVN and Sync immediately and after a delay to account for eventual consistency.

- Error Handling and Fallbacks
  - Local message fallback for DVN when canister update calls are unavailable.
  - Defensive returns in APIs to keep the dashboard responsive.


# Local Development

- ICP Host selection: `apps/aigent-z/services/ops/icAgent.ts`
  - Uses `http://127.0.0.1:4943` when `DFX_NETWORK=local` or when explicitly set in `.env.local`.
- Health check: `apps/aigent-z/app/api/ops/icp/health/route.ts` probes `/api/v2/status`.
- EVM chain configs are initialized lazily on status query.


# Key Files and Paths

- UI: `apps/aigent-z/app/ops/page.tsx`
- DVN APIs: `apps/aigent-z/app/api/ops/dvn/*`
- Sync APIs: `apps/aigent-z/app/api/ops/sync/*`
- BTC APIs: `apps/aigent-z/app/api/ops/btc/*`
- Cross‑chain status: `apps/aigent-z/app/api/ops/crosschain/status/route.ts`
- Health: `apps/aigent-z/app/api/ops/icp/health/route.ts`
- Hooks: `apps/aigent-z/hooks/ops/*`


# Operational Runbook (Summary)

- Detect Drift: Check `Canister Sync Status` card or call `GET /api/ops/sync/status`.
- Repair Drift: Click `Repair` or call `POST /api/ops/sync/repair`.
- Process DVN: Click `Process via LayerZero` or call `POST /api/ops/layerzero/process`.
- BTC Batch/Anchor: Use `Batch Now` / `Fast Anchor` buttons or corresponding APIs.


# Future Enhancements

- Post‑process reconciliation validation in LayerZero processing.
- Fine‑grained “process selected” controls.
- CI checks for route presence and canister connectivity.
