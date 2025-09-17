# Changelog

## [Unreleased] - 2025-09-16

### Added
- Ops Console: Single, clean set of chain explorer cards (Ethereum Sepolia, Polygon Amoy, Bitcoin Testnet) with explorer deep-links.
- ICP DVN: Prominent subheader and verification tools (View DVN Status, View Anchor Status, Get Receipt, View Batches, Get DVN Message, List Attestations).
- MetaMask integration for Polygon Amoy (auto add/switch + account display). Sepolia wiring pending contract.
- Polygon Amoy: In-app mint flow wired via `ethers.js` with minimal ABI for `mintQube(string,string)` and on-chain readbacks (ownerOf, tokenURI, getMetaQubeLocation, getEncryptionKey).
- Bitcoin Testnet: End-to-end flow in UI — Generate Testnet Key (in-browser WIF + tb1 address), Derive Address (from pasted WIF), Get UTXOs, Broadcast Test TX, Explorer links.
- API routes (server-side proxies) for BTC:
  - `GET /api/btc/utxos?address=`
  - `GET /api/btc/fee`
  - `GET /api/btc/txhex?txid=`
  - `POST /api/btc/broadcast`
  These avoid browser CORS and provide better error surfaces.

### Changed
- Simplified Ops Console UI to remove duplicated sections and clarify Live vs Simulation status.
- Labeled ICP DVN clearly under Cross-Chain Status.
- Explorer buttons upgraded to context-aware deep links (dynamic tx hash; address link before first tx) with copy/open helpers.
- Added request timeouts and improved error logging for BTC network calls in UI.

### Notes
- Polygon Amoy contract address: `0x632E1d32e34F0A690635BBcbec0D066daa448ede` (mint function wired with minimal ABI).
- Ethereum Sepolia contract not yet deployed (awaiting address + ABI).
- Next.js (ops-console) webpack configured for async WebAssembly to support `tiny-secp256k1`.

