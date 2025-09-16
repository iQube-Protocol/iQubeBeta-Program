# iQube Beta Program Development Workflow

## Overview
This monorepo supports concurrent development of three correlated projects:
- **Aigent Z Beta** (Next.js app at `apps/aigent-z/`)
- **21 Sats Market** (Next.js marketplace at `apps/21sats-market/`)
- **Ops Console** (Next.js dashboard at `apps/ops-console/`)

## Quick Start

### Prerequisites
- Node 20+ and pnpm 9+
- Rust and Cargo (for ICP canisters)
- Foundry (for smart contracts)
- dfx (for ICP deployment)

### Running All Three Tracks

```bash
# Terminal 1: Aigent Z Beta
cd /Users/hal1/CascadeProjects/iQubeBeta-Program/apps/aigent-z
npm run dev
# → http://localhost:3000

# Terminal 2: 21 Sats Market
cd /Users/hal1/CascadeProjects/iQubeBeta-Program
pnpm --filter @iqube/21sats-market dev
# → http://localhost:3006

# Terminal 3: Ops Console
pnpm --filter @iqube/ops-console dev
# → http://localhost:3007
```

### SDK Development

The shared SDK at `packages/sdk-js/` provides:
- TypeScript types aligned with Aigent Z Beta schema
- Cross-chain status functions (`getAnchorStatus`, `getDualLockStatus`)
- BTC indexer adapters (Blockstream, Mempool.space)

```bash
# Build SDK
pnpm --filter @iqube/sdk-js build

# Watch mode during development
pnpm --filter @iqube/sdk-js dev
```

### Smart Contracts (Foundry)

```bash
cd contracts

# Compile contracts
forge build

# Run tests
forge test -vv

# Deploy to testnet (requires env vars)
forge script script/Deploy.s.sol --rpc-url sepolia --broadcast
```

### ICP Canisters

```bash
# Start local replica
dfx start --background

# Deploy all canisters
dfx deploy

# Test proof-of-state canister
dfx canister call proof_of_state issue_receipt '("test_hash_123")'
dfx canister call proof_of_state batch
dfx canister call proof_of_state anchor
```

## Environment Configuration

Copy and configure environment files:
```bash
# For local development
cp ops/env/.env.example .env.local

# Configure testnet RPC URLs
ETH_RPC_URL_SEPOLIA=https://...
ARBITRUM_SEPOLIA_RPC_URL=https://...
# ... etc
```

## Workspace Commands

```bash
# Install all dependencies
pnpm install

# Build all packages
pnpm build

# Run tests across workspace
pnpm test

# Lint all packages
pnpm lint

# Format all code
pnpm format
```

## Makefile Targets

```bash
# Bootstrap entire environment
make bootstrap

# Start development (dfx + all apps)
make dev

# Build everything (packages + contracts + canisters)
make build

# Run all tests
make test

# Deploy to testnets
make deploy-dev    # Local dfx
make deploy-stage  # IC testnet
make deploy-prod   # IC mainnet (manual gate)
```

## Cross-Chain Integration

### BTC Anchoring Flow
1. Issue receipt: `proof_of_state.issue_receipt(data_hash)`
2. Batch receipts: `proof_of_state.batch()`
3. Anchor to BTC: `proof_of_state.anchor()`
4. Verify via indexers: `BTCIndexer.getConfirmations(txid)`

### EVM Integration
1. Deploy iQube classes: `IQubeClasses.createClass(...)`
2. Mint instances: `IQubeInstances.mintInstance(...)`
3. Anchor to BTC: `IQubeInstances.anchorToBitcoin(...)`

### SDK Usage in Apps
```typescript
import { getAnchorStatus, getDualLockStatus } from '@iqube/sdk-js';
import { BTCIndexer } from '@iqube/indexers';

// Check BTC anchor status
const status = await getAnchorStatus('iqube_id_123');
console.log(`Anchored: ${status.anchored}, Depth: ${status.depth}`);

// Check dual-lock status (EVM + BTC)
const dualLock = await getDualLockStatus('iqube_id_123');
console.log(`EVM: ${dualLock.evmBound}, BTC: ${dualLock.btcAnchored}`);
```

## Team Onboarding

### New Developer Setup
1. Clone monorepo: `git clone <repo-url>`
2. Initialize submodules: `git submodule update --init --recursive`
3. Install tooling: `make bootstrap`
4. Copy env template: `cp ops/env/.env.example .env.local`
5. Start development: `make dev`

### Project Structure
```
├── apps/
│   ├── aigent-z/          # Existing Aigent Z Beta (submodule)
│   ├── 21sats-market/     # Consumer marketplace
│   └── ops-console/       # Operations dashboard
├── canisters/             # ICP Rust canisters
├── contracts/             # Foundry smart contracts
├── packages/
│   ├── sdk-js/           # Shared TypeScript SDK
│   └── indexers/         # BTC/EVM indexers
├── docs/                 # Architecture and runbooks
└── ops/                  # CI/CD, env, scripts
```

### Git Workflow
- Main branch: `main`
- Feature branches: `feature/description`
- Submodules are read-only references
- SDK changes require version bumps
- CI/CD runs on all pushes to `main`

## Troubleshooting

### Common Issues
1. **Node version**: Ensure Node 20+ is active (`nvm use 20`)
2. **pnpm not found**: Run `corepack enable && corepack prepare pnpm@9 --activate`
3. **dfx not installed**: Install via `sh -c "$(curl -fsSL https://internetcomputer.org/install.sh)"`
4. **Port conflicts**: Apps run on 3000, 3006, 3007 by default

### Performance
- Use `pnpm` for faster installs
- Run apps in parallel for concurrent development
- Use `make dev` for unified startup
- Monitor with Ops Console dashboard

## Support
- Architecture docs: `/docs/architecture/`
- API docs: `/docs/openapi/`
- Runbooks: `/docs/runbooks/`
- Issues: GitHub Issues in monorepo
