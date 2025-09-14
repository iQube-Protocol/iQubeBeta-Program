# Build Manual & Setup Plan — iQube ICP/BTC + Aigent Z Beta + 21 Sats

**Audience**: Windsurf IDE Agent (and human engineers)

**Goal**: Stand up the iQube ICP/BTC protocol stack, integrate it with Aigent Z Beta, and scaffold the 21 Sats consumer app so all three tracks run in parallel and converge. Includes reproducible commands, envs, targets, and acceptance gates.

---

## 0) Repo Strategy & Monorepo Layout
Create a top‑level monorepo (or workspace) that references existing repos as submodules:

```
iqube-program/
├─ apps/
│  ├─ aigent-z/                 # Aigent Z Beta (Next.js) — as submodule
│  ├─ 21sats-market/            # 21 Sats site/marketplace (Next.js)
│  └─ ops-console/              # Operator dashboards
├─ canisters/                   # ICP program (Rust/Motoko)
│  ├─ cross_chain_service/
│  ├─ evm_rpc/
│  ├─ btc_signer_psbt/
│  ├─ proof_of_state/
│  ├─ identity_registry/
│  ├─ storage_fabric/
│  └─ risk_policy/
├─ contracts/                   # EVM smart contracts
│  ├─ tokens/ (QOYN, QCNT, staking, treasury)
│  ├─ iq_classes/ (ERC-721/1155, OSFT)
│  └─ iq_instances/ (ERC-721/1155, ONFT)
├─ packages/
│  ├─ sdk-js/                   # TypeScript SDK used by apps
│  └─ indexers/                 # BTC ordinal/runes + EVM event indexers
├─ vendor/
│  ├─ dvn/                      # OracleKit/dvn (LayerZero DVN on ICP) — submodule
│  └─ refs/                     # Other external refs (read‑only)
├─ docs/                        # Architecture & diagrams (Mermaid)
│  ├─ architecture/
│  └─ runbooks/
├─ ops/                         # CI/CD, infra, scripts
│  ├─ github/                   # GitHub Actions
│  ├─ scripts/
│  └─ env/
└─ Makefile
```

**Submodules**
- `apps/aigent-z` → `https://github.com/iQube-Protocol/AigentZBeta`
- `vendor/dvn` → `https://github.com/OracleKit/dvn`

---

## 1) Tooling Prereqs (Windsurf Agent)

**OS packages**
- Node.js ≥ 20, `pnpm` ≥ 9
- Rust ≥ 1.78, targets: `wasm32-unknown-unknown`
- `dfx` ≥ latest stable
- Foundry (`foundryup`) or Hardhat (choose one; plan assumes Foundry)
- `jq`, `yq`, `openssl`, `git`, `docker`

**Install (scriptable)**
```
# Node/pnpm
corepack enable && corepack prepare pnpm@latest --activate

# Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y
rustup target add wasm32-unknown-unknown

# DFX
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

dfx --version

# Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

---

## 2) Environment & Secrets

Create environment files per environment. Never commit secrets.

```
ops/env/.env.local
ops/env/.env.stage
ops/env/.env.prod
```

**.env keys (sample)**
```
# EVM
EVM_NETWORK=sepolia
ETH_RPC_URL_SEPOLIA=...  # Alchemy/Infura
LZ_ENDPOINT_SEPOLIA=...
LZ_DVN_ADDR=...          # if using DVN

# ICP/DFX
DFX_NETWORK=ic
ICP_IDENTITY=default

# BTC (testnet for dev/stage)
BTC_NETWORK=testnet
BTC_ANCHOR_MIN_DEPTH=6
BTC_ANCHOR_CADENCE_MIN=30

# Aigent Z
NEXT_PUBLIC_IQUBE_API=https://api.dev.iqube
NEXT_PUBLIC_SHOW_BTC_BADGES=true

# Payments
QOYN_TREASURY_ADDR=0x...
QCNT_TREASURY_ADDR=0x...
STABLECOIN_CUSTODIAN_ID=...

# KYC
KYC_PROVIDER_ID=sumsub
KYC_API_URL=...
KYC_WEBHOOK_SECRET=...
```

**Secrets**: store locally via `.env` (dev) and in CI via GitHub Encrypted Secrets. For production, prefer vault (e.g., GitHub OIDC → cloud KMS).

---

## 3) Bootstrap Commands

```
# 1) Clone monorepo & submodules
mkdir -p ~/workspace && cd ~/workspace
git clone <your_repo> iqube-program && cd iqube-program
git submodule add https://github.com/iQube-Protocol/AigentZBeta apps/aigent-z
git submodule add https://github.com/OracleKit/dvn vendor/dvn

# 2) Install JS deps
pnpm i --filter ./apps/** --filter ./packages/**

# 3) Install canister deps
cargo build --workspace

# 4) DFX localnet (for fast inner loop)
dfx start --clean --background

# 5) Deploy core canisters (local)
dfx deploy cross_chain_service
dfx deploy evm_rpc
dfx deploy btc_signer_psbt
dfx deploy proof_of_state
dfx deploy identity_registry
dfx deploy storage_fabric
dfx deploy risk_policy

# 6) Build & run apps (dev)
pnpm --filter @iqube/aigent-z dev
pnpm --filter @iqube/21sats-market dev
```

> **Tip**: Provide canned `dfx.json` entries for all canisters; expose Candid UIs for manual pokes.

---

## 4) Makefile Targets (Agent‑friendly)

```
.PHONY: bootstrap dev test lint format build deploy-dev deploy-stage deploy-prod anchor

bootstrap:
	pnpm i --filter ./apps/** --filter ./packages/**
	cargo build --workspace

lint:
	npx biome check . || true
	cargo clippy --all-targets -- -D warnings || true

format:
	npx biome format .
	cargo fmt --all

dev:
	dfx start --background || true
	dfx deploy
	pnpm -w dev

build:
	pnpm -w build
	cargo build --release --workspace

test:
	pnpm -w test
	cargo test --workspace

anchor:
	dfx canister call proof_of_state anchor '()'

deploy-dev:
	dfx deploy --network local

deploy-stage:
	dfx deploy --network ic

deploy-prod:
	# promote stage artifacts with tag
```

---

## 5) EVM Contracts — Build & Deploy

**Foundry layout**
```
contracts/
  src/
    tokens/QOYN.sol
    tokens/QCNT.sol
    treasury/Treasury.sol
    staking/Staking.sol
    iq/Classes.sol
    iq/Instances.sol
  script/
    DeployTokens.s.sol
    DeployIQube.s.sol
  test/
```

**Commands**
```
forge build
forge test -vv
forge script script/DeployTokens.s.sol \  --rpc-url $$ETH_RPC_URL_SEPOLIA \  --private-key $$DEPLOYER_KEY \  --broadcast
```

Record deployed addresses in `ops/artifacts/<env>/contracts.json`.

---

## 6) ICP Canisters — Build & Deploy

**Example canister names**
- `cross_chain_service` (adapters: LayerZero DVN, BTC signer)
- `evm_rpc` (JSON‑RPC proxy; rate‑limited)
- `btc_signer_psbt` (tECDSA; PSBT; broadcast)
- `proof_of_state` (Merkle batches; BTC anchor caller)
- `identity_registry` (FIO + KYC attestations)
- `storage_fabric` (meta/blak/tokenQube orchestration)
- `risk_policy` (limits, sanctions, geo, circuit‑breakers)

**Commands**
```
dfx deploy <canister>
dfx canister call cross_chain_service submit '(record { data = vec {1;2;3} })'
dfx canister call btc_signer_psbt anchorRoot '(record { root = "0x..." })'
```

**Artifacts**: write `ops/artifacts/<env>/canisters.json` with canister IDs.

---

## 7) LayerZero DVN on ICP

- Pull `vendor/dvn` and follow its README for building the DVN canister(s).
- Configure DVN participants/quorum in `ops/env/.env.*`.
- Smoke‑test an **EVM↔EVM** message across two testnets (e.g., Sepolia → Arbitrum Sepolia) using the DVN for verification.

**Acceptance**: `finalize()` observed on destination; DVN quorum logs show attestation.

---

## 8) BTC Anchoring & PSBT Flow

- `proof_of_state` batches receipts (Merkle) per cadence env var.
- `btc_signer_psbt` constructs/broadcasts anchor tx (OP_RETURN root), tracks confirmations, handles RBF/reorg policy.

**Commands**
```
dfx canister call proof_of_state issueReceipt '(record { leaf = "0x..." })'
dfx canister call proof_of_state batch '()'
dfx canister call proof_of_state anchor '()'
```

**Acceptance**: Anchor txid returned; `depth >= BTC_ANCHOR_MIN_DEPTH` triggers finalized status.

---

## 9) Indexers

- **EVM**: subgraph or Foundry script watchers → write to Postgres/SQLite.
- **BTC**: lightweight ordinal & runes indexer (reuse public indexers or write a minimal watcher) → write to DB.
- Expose a read API: `packages/sdk-js` consumes for UI badges.

---

## 10) Aigent Z Beta Integration

**Add environment** in `apps/aigent-z/.env.local`:
```
NEXT_PUBLIC_IQUBE_API=http://localhost:8787
NEXT_PUBLIC_BTC_ANCHOR_DEPTH=6
```

**Wire API client** to the new SDK (`packages/sdk-js`).

**UI updates**
- Creation flow: toggles for **Dual‑Lock**, **Anonymous Mode**.
- Registry list: BTC badges (Ordinal present, Anchored, Dual‑Lock bound).
- Receipts panel: SPV proof, depth status.
- Payments: $QOYN / $QCNT checkout.

**Smoke test**
- Create class → mint instance → see BTC anchor badge after cadence; verify SPV via button.

---

## 11) 21 Sats App Scaffold

```
apps/21sats-market/
  pages/
  components/
  lib/
  .env.local
```

**Features to toggle on**
- Product SKUs map to **iQube classes**.
- Checkout with **$QCNT/$QOYN**.
- Token‑gated content via **tokenQube**.
- Public “Provenance” page showing anchor receipts.

**Smoke test**
- Buy a shard → receive instance (EVM) → see incoming Ordinal mirror (when enabled) → show anchored receipt.

---

## 12) KYC/AML Adapter (modular)

- Implement interface: `attest(providerId, level, expiry) -> attestationId`.
- Store only attestations (no PII); enforce gate per route.
- Webhook endpoint for provider result → writes attestation row.

**Acceptance**: Flows requiring KYC reject unauthenticated users; valid attestation passes.

---

## 13) Payments, Treasury, Staking

- Deploy $QOYN with **21M/21y** emissions controller (parametrized schedule, upgradeable policy gates).
- Deploy $QCNT (custodian‑backed claims). Hook **Proof‑of‑Reserves** source(s) to ops console.
- Fee router: burn / stake / liquidity split (configurable).

**Acceptance**: Payments route; staking APR within bounds; PoR dashboard green.

---

## 14) CI/CD

**GitHub Actions** (sketch)
```
name: ci
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
      - run: pnpm i --frozen-lockfile
      - run: pnpm -w lint && pnpm -w build && pnpm -w test
      - name: Rust
        run: |
          rustup toolchain install stable
          rustup target add wasm32-unknown-unknown
          cargo build --workspace
```

**Deploy pipelines**: separate workflows for stage/prod; require manual approvals and tag promotions; publish artifacts `contracts.json`, `canisters.json`.

---

## 15) Ops & Runbooks

- **Anchors**: monitor fee spikes; adjust cadence; re‑anchor after reorg.
- **Bridge/DVN**: watch quorum health; pause routes on disagreement.
- **Custodian/PoR**: alert on delta; auto‑pause QCNT minting when attestation stale.
- **KYC**: rotate API keys; monitor webhook failures.

---

## 16) Acceptance Gates (per environment)

**DEV (local/testnets)**
- EVM: contracts deployed; mint class/instance works.
- ICP: canisters deployed; `submit/finalize/verify` flows; anchor tx observed on BTC testnet.
- UI: Aigent Z shows BTC badges; checkout with mock $QCNT.

**STAGE**
- DVN quorum active; LayerZero EVM↔EVM message finalizes.
- BTC anchors at cadence; SPV verify depth ≥ configured.
- KYC gates active; escrow flows pass.

**PROD (gated)**
- External audit sign‑off for bridge/canisters/contracts.
- Runbooks approved; alerting wired.
- Canary mints/anchors clean for 72h.

---

## 17) Windsurf Agent — Execution Checklist

1. **Bootstrap**: run `make bootstrap` → success.
2. **Local canisters**: `dfx start` → `dfx deploy` → all canisters healthy.
3. **Contracts**: `forge build && forge test` → deploy to Sepolia; write `contracts.json`.
4. **SDK**: build `packages/sdk-js` → link to apps.
5. **Apps**: `pnpm -w dev` → Aigent Z + 21Sats running.
6. **Flows**: create class (Dual‑Lock on) → mint instance → see receipt → `make anchor` → BTC testnet tx visible.
7. **CI**: open PR → CI green; artifacts saved.
8. **Stage**: `make deploy-stage` → repeat smoke tests; enable DVN path and LayerZero EVM↔EVM.
9. **Prod prep**: gather audit outputs; tag release candidates; run runbooks; schedule launch.

---

## 18) Where to put things in the repo

- This manual → `/docs/BUILD_MANUAL.md`
- Architecture doc → `/docs/architecture/iQube_ICP_BTC_Architecture.md`
- Diagrams → `/docs/architecture/iQube_ICP_BTC_Diagrams.md`
- Runbooks → `/docs/runbooks/*.md`
- Artifacts → `/ops/artifacts/<env>/*.json`

---

## 19) Notes

- Keep adapters **swap‑ready** (future TachiAdapter) by only calling cross‑chain functions via `cross_chain_service`.
- $QOYN emissions and $QCNT PoR endpoints are **parametrized**; adjust via config not code where possible.
- Prefer **staged rollouts**: EVM first; then BTC mirrors; then dual‑lock; then Rune policy.
