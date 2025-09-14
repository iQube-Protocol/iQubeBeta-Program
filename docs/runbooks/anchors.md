# Runbook — BTC Anchors & Proof-of-State

**Goal**: Keep BTC anchoring reliable and cost-effective; ensure SPV proofs verify at configured depth.

## KPIs
- Anchor success rate ≥ 99.5%
- Median fee/anchor below budget
- Reorg recovery < 10 minutes

## Cadence
- `BTC_ANCHOR_CADENCE_MIN` (default 30min in stage, 60–180min in prod)
- Adaptive: increase cadence when queue length > threshold; reduce when fees spike

## Procedure
1. **Check queue**: pending ReceiptQube count and oldest age.
2. **Estimate fee**: use fee estimator; set RBF policy for spikes.
3. **Broadcast**: `dfx canister call proof_of_state anchor '()'`
4. **Confirmations**: wait until `depth >= BTC_ANCHOR_MIN_DEPTH` (default 6).
5. **Publish proof**: expose SPV proof to clients.

## Reorg Handling
- Detect reorg via block hash mismatch.
- Move state to `Reorged` and re-batch.
- Re-anchor with higher fee; update proofs.

## Incident Playbook
- If anchors stall > 60 min: check mempool congestion; raise fee; RBF existing tx.
- If repeated failures: switch to known good broadcaster peers; test on testnet.
- If proof verification fails: rebuild Merkle tree; verify leaf; re-anchor if needed.
