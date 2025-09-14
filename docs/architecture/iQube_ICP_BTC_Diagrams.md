# iQube ICP/BTC — Protocol Diagrams (v1)

**Purpose**: Protocol-grade visuals for engineers, auditors, and operators. Mirrors the style/coverage used for Aigent Z Beta. Copy-pasteable **Mermaid** so we can render in docs sites and IDE previews.

---

## 1) Layered Architecture & Trust Boundaries

```mermaid
flowchart TB
  classDef tb fill:#fff,stroke:#333,stroke-width:2px,stroke-dasharray: 5 5
  classDef svc fill:#eef,stroke:#336
  classDef chain fill:#efe,stroke:#363
  classDef infra fill:#fee,stroke:#633

  subgraph CLIENTS[Client Applications]
    A1[Aigent Z Beta UI]
    A2[21 Sats Site & Marketplace]
    A3[3rd-party Wallets/dApps]
  end

  subgraph EDGE[API Edge / Gateways]
    Z1[Registry API Gateway]
    Z2[Auth + KYC Gateway]
    Z3[Payments Proxy]
  end
  class EDGE tb

  subgraph ICP[iQube Protocol — ICP Canisters]
    C1[CrossChainService\n(LayerZero DVN on ICP)]:::svc
    C2[EVM RPC Canister]:::svc
    C3[BTC Signer (tECDSA) + PSBT]:::svc
    C4[Proof-of-State Anchor Publisher]:::svc
    C5[IdentityRegistry\n(DIDQube + FIO)]:::svc
    C6[StorageFabric\n(metaQube / blakQube / tokenQube)]:::svc
    C7[Risk & Policy Engine]:::svc
  end
  class ICP tb

  subgraph EVM[EVM Chains]
    E1[ERC-20/721/1155 Contracts]:::chain
    E2[LayerZero Endpoints (OFT/ONFT/OSFT)]:::chain
    E3[Treasury, Staking, Escrow]:::chain
  end
  class EVM tb

  subgraph BTC[Bitcoin]
    B1[Ordinals / BRC-721]:::chain
    B2[Runes Policies]:::chain
    B3[Anchors (OP_RETURN)]:::chain
  end
  class BTC tb

  A1-->Z1
  A2-->Z1
  A3-->Z1
  Z1-->C6
  Z1-->C5
  Z1-->C1
  Z1-->C2
  Z1-->C3
  Z1-->C4
  Z2-->C5
  Z3-->E3
  C1<-->E2
  C2<-->E1
  C3-->B1
  C3-->B2
  C4-->B3
  C7-.policy.->C6
  C7-.policy.->C1
  C7-.policy.->E3
```

---

## 2) C4-Style Container View

```mermaid
flowchart LR
  user[Users/Creators]
  admin[Operators]

  ui[Web UI (Aigent Z Beta)]
  apigw[API Gateway]
  auth[Auth/KYC Gateway]

  can_xcs[CrossChainService]
  can_evmm[EVM RPC]
  can_btc[BTC Signer/PSBT]
  can_pos[Proof-of-State]
  can_id[IdentityRegistry]
  can_store[StorageFabric]
  can_risk[Risk & Policy]

  evm[EVM Contracts]
  lz[LayerZero Endpoints]
  btc[Bitcoin Network]

  user-->ui
  admin-->ui
  ui-->apigw
  ui-->auth
  apigw-->can_store
  apigw-->can_id
  apigw-->can_xcs
  apigw-->can_evmm
  apigw-->can_btc
  apigw-->can_pos
  can_xcs<-->lz
  can_evmm<-->evm
  can_btc-->btc
  can_pos-->btc
```

---

## 3) Plugin Architecture — CrossChainService

```mermaid
classDiagram
  class CrossChainService {
    +submit(payload): MsgId
    +finalize(msgId): Result
    +verify(proof): bool
  }
  class LayerZeroAdapter {
    +send(srcChain,dstChain,payload)
    +verify(proof)
  }
  class BTCAdapter {
    +mintOrdinal(meta)
    +mintBRC721(coll)
    +mintRunes(policy)
    +anchorRoot(root)
    +verifySPV(proof)
  }
  class TachiAdapter {
    +submitVUTXO(payload)
    +verifyProof(recursiveProof)
  }
  CrossChainService o--> LayerZeroAdapter
  CrossChainService o--> BTCAdapter
  CrossChainService o--> TachiAdapter
```

---

## 4) Data Model (ERD)

```mermaid
classDiagram
  class IqubeClass {
    classId: string
    creator: address
    policy: json
    metaQubeHash: bytes32
    osftClassId?: string
    runesPolicyId?: string
  }
  class IqubeInstance {
    instanceId: string
    classId: string
    owner: address | bc1
    tokenQubeRef: string
  }
  class MetaQube {
    id: string
    hash: bytes32
    uri: string
  }
  class BlakQube {
    id: string
    uri: string
    encSpec: string
  }
  class TokenQube {
    id: string
    policy: json
    keywrap: string
  }
  class ReceiptQube {
    id: string
    merkleLeaf: bytes32
    anchorTxid: string
  }
  class Attestation {
    id: string
    type: string
    issuer: string
    expiry: uint64
  }
  class FIOHandle {
    id: string
    handle: string
    owner: address
  }

  IqubeClass "1" -- "*" IqubeInstance: materializes
  IqubeClass "1" -- "1" MetaQube: describes
  IqubeInstance "*" -- "*" BlakQube: protects
  IqubeInstance "1" -- "1" TokenQube: gates
  IqubeInstance "*" -- "*" ReceiptQube: proves
  FIOHandle "1" -- "*" Attestation: binds
```

---

## 5) Sequence — Dual-Lock Class Mint (EVM ↔ BTC)

```mermaid
sequenceDiagram
  participant UI as Aigent Z UI
  participant GW as API Gateway
  participant XCS as CrossChainService (ICP)
  participant EVM as EVM Contracts
  participant BTC as BTC Signer/PSBT
  participant AN as Anchor Publisher

  UI->>GW: createClass(policy.dualLock=true)
  GW->>EVM: deploy ERC-1155/721 class
  GW->>XCS: bindDualLock(EVM.class)
  XCS->>BTC: mint BRC-721 collection
  XCS-->>GW: dualLockBound(evmClass, btcClass)
  GW->>AN: scheduleAnchor(merkleRoot)
  AN->>BTC: publish OP_RETURN(root)
  GW-->>UI: Class ready (IDs + anchor ref)
```

---

## 6) Sequence — Instance Mint + BTC Mirror + Proof-of-State

```mermaid
sequenceDiagram
  participant UI
  participant GW
  participant XCS
  participant EVM
  participant BTC
  participant AN

  UI->>GW: mintInstance(classId, to)
  GW->>EVM: ERC-721 mint
  GW->>XCS: mirrorOrdinal(instanceMeta)
  XCS->>BTC: inscribe Ordinal
  GW->>GW: build ReceiptQube(leaf)
  GW->>AN: batchAndAnchor()
  AN->>BTC: publish root
  GW-->>UI: instanceIds + SPV verifiable receipt
```

---

## 7) Sequence — EVM↔EVM via LayerZero DVN (on ICP)

```mermaid
sequenceDiagram
  participant SRC as EVM Chain A
  participant LZ as LayerZero Endpoints
  participant DVN as DVN (Validators on ICP)
  participant DST as EVM Chain B

  SRC->>LZ: send(payload)
  LZ-->>DVN: deliver for verification
  DVN-->>LZ: attest quorum
  LZ->>DST: finalize(payload)
```

---

## 8) Sequence — BTC PSBT Escrow (Time-lock + Oracle)

```mermaid
sequenceDiagram
  participant BUY as Buyer
  participant ESC as EVM Escrow
  participant XCS as BTC Adapter (ICP)
  participant ORA as Oracle
  participant BTC as Bitcoin

  BUY->>ESC: lock funds (EVM)
  ESC->>XCS: request PSBT escrow (btcAddr, amount)
  XCS->>BTC: create PSBT (nLocktime)
  ORA-->>XCS: condition met (proof)
  XCS->>BTC: finalize & broadcast
  ESC->>BUY: release/settle
```

---

## 9) Sequence — Payments with $QOYN/$QCNT + KYC Gate

```mermaid
sequenceDiagram
  participant UI
  participant KYC as KYC Gateway
  participant PAY as Payments Proxy
  participant TRE as Treasury/Router

  UI->>KYC: attest(level >= L2)
  UI->>PAY: pay(asset=$QCNT, amount)
  PAY->>TRE: route (burn/stake/liquidity)
  TRE-->>UI: receipt
```

---

## 10) Sequence — Proof-of-State Anchor Lifecycle

```mermaid
sequenceDiagram
  participant GW as Registry/API
  participant POS as Proof-of-State
  participant BTC as Bitcoin
  participant CLI as Verifier Client

  GW->>POS: issueReceipt(events)
  POS->>POS: batch(merkleRoot)
  POS->>BTC: publish(root)
  BTC-->>POS: confirmations(depth)
  CLI->>POS: fetch proof(leaf)
  CLI->>BTC: verify SPV(proof, depth>=k)
  CLI-->>CLI: status = Finalized
```

---

## 11) State Machine — Dual-Lock Token

```mermaid
stateDiagram-v2
  [*] --> EVM_Minted
  EVM_Minted --> Bound: bindDualLock
  Bound --> Locked_EVM: lockOnEVM
  Locked_EVM --> Minted_BTC: mintBTC
  Minted_BTC --> Released_BTC: releaseOnBTC
  Released_BTC --> Burned_EVM: burnEVM
  Burned_EVM --> [*]

  state ErrorStates {
    Desync
    Disputed
  }
  Bound --> Desync: invariantFail
  Desync --> Disputed: challenge
  Disputed --> Bound: resolve
```

---

## 12) State Machine — Anchor Lifecycle

```mermaid
stateDiagram-v2
  [*] --> Collected
  Collected --> Batched: buildMerkle
  Batched --> Published: OP_RETURN
  Published --> Confirmed: depth >= k
  Confirmed --> Finalized: checkpoint
  Published --> Reorged: reorg
  Reorged --> Batched
  Finalized --> [*]
```

---

## 13) Deployment Diagram (Envs & Observability)

```mermaid
flowchart TB
  subgraph Dev
    dfx[dfx localnet]
    sepolia[Sepolia]
    testbtc[Testnet BTC]
  end
  subgraph Stage
    icp_stage[ICP stage canisters]
    evm_stage[EVM testnets]
    btc_stage[BTC testnet]
  end
  subgraph Prod
    icp_prod[ICP prod canisters]
    evm_prod[EVM mainnets]
    btc_main[Bitcoin mainnet]
  end

  subgraph Observability
    otel[OpenTelemetry Collector]
    dash[Ops Dashboards]
    alerts[Alerting]
  end

  dfx-->icp_stage-->icp_prod
  sepolia-->evm_stage-->evm_prod
  testbtc-->btc_stage-->btc_main
  icp_prod-->otel-->dash
  icp_prod-->alerts
```

---

## 14) Threat Model (STRIDE) — Summary Table

| Threat             | Surface                   | Control                                               |
| ------------------ | ------------------------- | ----------------------------------------------------- |
| Spoofing           | KYC/Identity, Bridge Msgs | Attestations w/ expiry; DVN quorum; sig verification  |
| Tampering          | Merkle roots, receipts    | SPV proofs; immutable logs; audits                    |
| Repudiation        | Payments, mints           | ReceiptQube + anchors; non-repudiation via signatures |
| Info Disclosure    | blakQube payloads         | Envelope encryption; tokenQube gating; RLS            |
| DoS                | Bridge/Anchors            | Rate limits; circuit breakers; backpressure           |
| Elev. of Privilege | Admin ops                 | Multi-sig; role-based access; approvals               |

---

## 15) TachiAdapter — Integration Path (Future)

```mermaid
sequenceDiagram
  participant UI
  participant GW
  participant XCS as CrossChainService
  participant TAC as TachiAdapter
  participant BTC as Bitcoin

  UI->>GW: submitCrossChain(payload)
  GW->>XCS: submit(payload)
  XCS->>TAC: submitVUTXO(payload)
  TAC->>BTC: anchor proof
  TAC-->>XCS: recursiveProof
  XCS->>XCS: verify(proof) -> Ok
  XCS-->>GW: finalized
```

---

**Notes**

- All diagrams are intentionally **interface-driven** to keep the program modular and to de-risk future swaps (e.g., TachiAdapter).
- We’ll export SVG/PNG variants for decks once the team confirms these are the right set; source-of-truth remains Mermaid in repo `/docs/architecture/`.
