---
id: ops-console-guide
title: Network Ops Console Guide
sidebar_label: Ops Console Guide
---

# Network Ops Console Guide

This guide explains how to use the Ops Console at `apps/aigent-z/app/ops/page.tsx` to monitor, reconcile, and test cross‑chain operations end‑to‑end.

## Cards Overview

- ICP Health: live status of canisters. Inline chip shows “Local ICP connected” when `127.0.0.1:4943` is reachable via `GET /api/ops/icp/health`.
- Cross‑Chain Status: counts EVM chains via `EVM RPC` canister and Non‑EVM (BTC, Solana). Lazily initializes chain configs if empty.
- Canister Sync Status: drift detection across `DVN` and `proof_of_state`. One‑click repair and LayerZero processing.
- ICP DVN: shows pending messages, unlock height, and attestations.
- BTC Testnet: batches, anchors, and latest anchor tx.
- EVM Cards: Sepolia, Amoy, Optimism Sepolia, Arbitrum Sepolia, Base Sepolia.

## DVN Mint Tests

- Abbreviated chain select (ETH Sepolia, POL Amoy, OP/ARB/BASE Sepolia; SOL Devnet, BTC Testnet).
- Controls:
  - Test TX: creates 0‑value EVM tx via MetaMask, then monitors.
  - Monitor: monitors the input `txHash` in DVN.
  - Use last: fills input from `localStorage.amoy_last_tx`.
  - Clear: clears input and storage.
- Idempotency: API prevents duplicate DVN entries for the same `txHash`.

## Sync and Processing

- Detect drift on the `Canister Sync Status` card, or `GET /api/ops/sync/status`.
- Repair drift with `POST /api/ops/sync/repair`.
- Process DVN via LayerZero with `POST /api/ops/layerzero/process`.
- The UI refreshes DVN and Sync both immediately and after a short delay to reflect eventual consistency.

## BTC Operations

- Create batch: `POST /api/ops/btc/batch-now`.
- Fast Anchor: `POST /api/ops/btc/fast-anchor`.
- Anchor: `POST /api/ops/btc/anchor`.
- Proof‑of‑state anchors via `btc_signer_psbt` and shows latest block/tx when available.

## Environment & Local Setup

- Local replica: `DFX_NETWORK=local`, `ICP_HOST=http://127.0.0.1:4943`.
- Set canister IDs in `apps/aigent-z/.env.local` to values from `dfx canister id ...`.
- Health endpoint: `GET /api/ops/icp/health`.
- Start app: in `apps/aigent-z/` run `npm run dev`.

## APIs and Hooks

- DVN APIs: `app/api/ops/dvn/*` with `useDVNStatus` and `useDVNMonitor` hooks.
- Sync APIs: `app/api/ops/sync/*` with `useSyncStatus` hook.
- BTC APIs: `app/api/ops/btc/*` with `useBTC_Testnet` hook.
- Cross‑chain: `app/api/ops/crosschain/status` with `useCrossChain` hook.

## Troubleshooting

- If EVM chains show 0, the status route lazily calls `evm.init_chain_configs()`; refresh after a moment.
- If DVN “Monitor” seems to duplicate entries, backend deduplication is active; repeated clicks for the same hash return the existing `messageId`.
- If LayerZero processing completes but counts lag, the UI will refresh twice; click `Re‑check DVN` to force it.
- If ICP is unreachable, ensure local replica is running and env points to `127.0.0.1:4943`.
