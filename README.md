# iQube Beta Program

[![Deploy Docusaurus Docs (Operating Manual)](https://github.com/iQube-Protocol/iQubeBeta-Program/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/iQube-Protocol/iQubeBeta-Program/actions/workflows/deploy-docs.yml)

A complete monorepo for the iQube Protocol: ICP canisters, SDK packages, and production-ready frontends (Aigent Z, Ops Console, 21 Sats Market) with comprehensive operations documentation.

## Latest Documentation Release

- Release: Ops Manual — Mermaid rendering + zoom/pan + "Why iQubes Matter"
- Tag: [docs-ops-manual-mermaid-2025-09-20](https://github.com/iQube-Protocol/iQubeBeta-Program/releases/tag/docs-ops-manual-mermaid-2025-09-20)
- Live Docs (GitHub Pages): https://iqube-protocol.github.io/iQubeBeta-Program/

### Highlights

- Mermaid diagrams enabled across the Operations Manual
- GitHub-style diagram theme + interactive zoom/pan controls
- New page: Getting Started → Why iQubes Matter (personas, Aigents, Orchestrators, DNV)
- Fixed MDX/sequence diagram parsing issues; all diagrams render
- Standardized docs server to port 3001 with auto-kill on start

## Operations Manual (Local)

From `docs/operating-manual/`:

```bash
npm install
npm start   # auto-kills port 3001, starts at http://localhost:3001
```

## CI/CD for Docs

- `.github/workflows/deploy-docs.yml`: Builds and deploys the Operating Manual to GitHub Pages on main

## Key Paths

- Frontend apps: `apps/`
- ICP canisters: `canisters/`
- Docs site: `docs/operating-manual/`
- SDKs & packages: `packages/`

## Links

- Repository: https://github.com/iQube-Protocol/iQubeBeta-Program
- Docs Release: https://github.com/iQube-Protocol/iQubeBeta-Program/releases/tag/docs-ops-manual-mermaid-2025-09-20
- Live Docs: https://iqube-protocol.github.io/iQubeBeta-Program/
- Ops Console Architecture (repo): `docs/architecture/ops-console.md`
- Ops Console Architecture (site): https://iqube-protocol.github.io/iQubeBeta-Program/docs/architecture/ops-console/
- Ops Console Guide (site): https://iqube-protocol.github.io/iQubeBeta-Program/docs/system-operations/ops-console-guide/
