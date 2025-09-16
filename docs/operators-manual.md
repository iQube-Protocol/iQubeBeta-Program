# iQube Ops Console – Operator Manual

This manual explains how to operate the iQube Ops Console to mint, move, and verify iQubes across ICP, Ethereum (Sepolia), Polygon (Amoy), and Bitcoin Testnet. It also clarifies the ICP DVN control-plane and Bitcoin anchorage audit trail.

## Contents
- Overview
- ICP DVN (Control Plane)
- EVM Testnets (MetaMask)
- Polygon Amoy Mint (Live)
- Ethereum Sepolia Mint (Planned)
- Bitcoin Testnet (WIF · UTXO · Broadcast)
- Verifications & Explorers
- End-to-End Flow
- Troubleshooting

---

## Overview
The Ops Console coordinates cross-chain iQube lifecycle:
- __ICP DVN__: The control-plane where iQubes enter a locked, canonical state. This governs cross-chain movement and enforces the single-active representation invariant.
- __EVM Chains__: Ethereum Sepolia and Polygon Amoy for NFT minting via MetaMask.
- __Bitcoin Testnet__: Produces an immutable audit trail (anchorage) of state transitions.

The UI provides:
- __ICP Canister Health__ – live health of core canisters
- __BTC Anchor Status__ – latest anchor state
- __ICP DVN Status__ – cross-chain lock state and identifiers
- __Chain Explorer Cards__ – quick links and MetaMask connect
- __End-to-End Test Suite__ – buttons to perform common tests
- __Test Anchoring__ – submit sample anchor payloads
- __Test Results__ – log of recent actions and outcomes

---

## ICP DVN (Control Plane)
- DVN resides on ICP canisters. No browser wallet is needed.
- The DVN maintains __single-active__ invariant: an iQube is either locked in DVN or realized on exactly one destination chain at a time.
- UI: `Cross-Chain Status` header with `ICP DVN` subheader.
- Fields:
  - __EVM TX__ – a DVN-related EVM hash if applicable
  - __ICP Receipt__ – identifier returned by DVN/receipts flow
  - __Lock Status__ – `Locked` or `Unlocked`
  - __Unlock Height__ – target height for unlock conditions

__Verification Buttons__ (under ICP DVN):
- __View DVN Status__ – show current `dualLockStatus`
- __View Anchor Status__ – show current `anchorStatus`
- __Get Receipt__ – fetch a specific receipt (wired via SDK when ID is provided)
- __View Batches__ – fetch anchor batches
- __Get DVN Message__ – fetch DVN message details
- __List Attestations__ – list attestations for a DVN message

Notes:
- These calls are read-only and query live canisters.

---

## EVM Testnets (MetaMask)
- Supported chains in UI:
  - __Ethereum Sepolia__ (Chain ID 11155111)
  - __Polygon Amoy__ (Chain ID 80002)
- Use the `Connect MetaMask` buttons on each card:
  - Requests account access
  - Adds the network if missing
  - Switches to the network
  - Displays the connected account once successful

__Minting__
- Requires the ERC-721 contract address and ABI with `mintQube(string uri, string key)`.
- Resulting transaction hash is written to `Test Results` and also appears on the corresponding chain card’s explorer link.

---

## Polygon Amoy Mint (Live)
- __Contract__: `0x632E1d32e34F0A690635BBcbec0D066daa448ede`
- __ABI__: Pending (provide to enable the mint button).
- Steps once ABI is provided:
  1. Click `Connect MetaMask (Amoy)`.
  2. Click `Mint on Amoy` (End-to-End Test Suite) – the dapp will:
     - Call `mintQube(uri, key)` via ethers.js signer
     - Wait for the transaction to broadcast
     - Log the tx hash to `Test Results`
     - Populate the Amoy explorer link with the tx hash

---

## Ethereum Sepolia Mint (Planned)
- __Contract__: Pending
- __ABI__: Pending
- Once provided, the flow mirrors Amoy:
  1. `Connect MetaMask (Sepolia)`
  2. `Mint on Sepolia`
  3. Tx hash is logged and linked to Etherscan

---

## Bitcoin Testnet (WIF · UTXO · Broadcast)
- Paste your __testnet WIF__ (private key) into the BTC card input.
- Use the faucet links to fund the derived address:
  - https://bitcoinfaucet.uo1.net/
  - https://coinfaucet.eu/en/btc-testnet/
- Planned UI actions:
  - __Get UTXOs__ – fetch unspent outputs for the derived address
  - __Broadcast Test TX__ – construct/sign/broadcast a small test transaction
- After broadcast, the tx hash is logged to `Test Results` and the BTC explorer card links to mempool.space with that hash.

Security:
- WIF is kept in memory only and never written to disk.

---

## Verifications & Explorers
- __ICP DVN__ – Verify via DVN/Receipt/Batches buttons (canister reads)
- __Sepolia__ – Etherscan link opens with last mint tx hash
- __Amoy__ – Polygonscan link opens with last mint tx hash
- __Bitcoin Testnet__ – Mempool link opens with last broadcast tx hash

---

## End-to-End Flow
1. __Lock at ICP DVN__ (issue receipt / lock state)
2. __Choose Destination__ (EVM or BTC)
3. __Realize Asset__ on the destination (NFT mint on EVM, transaction on BTC)
4. __DVN Records Movement__ ensuring single-active representation
5. __Bitcoin Anchorage__ periodically records a public, immutable summary of state transitions

---

## Troubleshooting
- __MetaMask not found__: Install MetaMask and refresh the page.
- __Network not added__: The dapp offers to add Amoy or Sepolia automatically.
- __Certificate verification errors (ICP)__: The UI falls back to mock data; retry or ensure canisters are reachable.
- __Explorer link shows placeholder__: Perform a real action (mint/broadcast); the UI will capture and update the hash.

---

## References
- Canisters (deployed):
  - proof_of_state
  - btc_signer_psbt
  - cross_chain_service
  - evm_rpc

- Ops Console URL: http://localhost:3007
