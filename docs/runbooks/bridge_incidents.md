# Runbook — Bridge/DVN Incidents

**Goal**: Maintain safe cross-chain operations; halt on disagreement; recover deterministically.

## Symptoms
- DVN quorum disagreement
- Replay detected / nonce mismatch
- Finalization timeout

## Immediate Actions
1. **Pause routes** using circuit breaker.
2. **Collect evidence**: DVN attestation logs, payload hash, nonce.
3. **Retry idempotently** after quorum health returns.

## Recovery
- For stuck EVM↔EVM: re-emit message with new nonce; mark old as abandoned.
- For BTC legs: ensure invariant (1:1 supply) holds; run reconciliation job.

## Post-Incident
- Root-cause analysis
- Add chaos test for the scenario
- Tune thresholds (timeouts, quorum size)
