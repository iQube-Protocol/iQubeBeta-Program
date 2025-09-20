---
sidebar_position: 2
title: Technical Architecture Diagrams
description: Detailed technical diagrams showing system components and data flows
---

# Technical Architecture Diagrams

This document provides comprehensive technical diagrams for the iQube Protocol architecture, including system components, data flows, and integration patterns.

## 1. Layered Architecture & Trust Boundaries

```mermaid
flowchart TB
  subgraph CLIENTS[Client Applications]
    A1[Aigent Z Beta UI]
    A2[21 Sats Marketplace]
    A3[Network Ops Console]
    A4[Third-party dApps]
  end

  subgraph EDGE[API Edge and Gateways]
    Z1[Registry API Gateway]
    Z2[Auth and KYC Gateway]
    Z3[Payments Proxy]
    Z4[Network Ops Integration]
  end

  subgraph ICP[iQube Protocol - ICP Canisters]
    C1[CrossChainService<br/>LayerZero DVN on ICP]
    C2[EVM RPC Canister<br/>Multi-chain Gateway]
    C3[BTC Signer<br/>tECDSA and PSBT]
    C4[Proof of State<br/>Anchor Publisher]
    C5[IdentityRegistry<br/>DIDQube and FIO]
    C6[StorageFabric<br/>metaQube blakQube tokenQube]
    C7[Risk and Policy Engine<br/>Compliance & Governance]
  end

  subgraph CHAINS[Blockchain Networks]
    E1[Ethereum Sepolia<br/>Live Testnet]
    E2[Polygon Amoy<br/>Live Testnet]
    E3[ICP DVN<br/>Cross-chain Service]
    B1[Bitcoin Testnet<br/>Anchoring & Settlement]
  end

  A1-->Z1
  A2-->Z1
  A3-->Z4
  A4-->Z1
  Z1-->C6
  Z1-->C5
  Z1-->C1
  Z1-->C2
  Z1-->C3
  Z1-->C4
  Z2-->C5
  Z3-->E1
  Z4-->C1
  Z4-->C2
  Z4-->C3
  Z4-->C4
  C1<-->E1
  C1<-->E2
  C1<-->E3
  C2<-->E1
  C2<-->E2
  C3-->B1
  C4-->B1
  C7-- policy --> C6
  C7-- policy --> C1
  C7-- policy --> E1
```

## 2. Web3 Ops Console Integration Architecture

**MAJOR ACHIEVEMENT**: Complete Web3 Ops Console functionality integrated into Aigent Z application.

```mermaid
flowchart TB
  subgraph AIGENT_Z[Aigent Z Application]
    UI[Main UI Interface]
    SETTINGS[Settings Menu]
    NETWORK_OPS[Network Ops Console<br/>Settings → Network Ops]
  end

  subgraph MONITORING[Real-time Monitoring]
    ETH_MONITOR[Ethereum Sepolia<br/>Live RPC Monitoring]
    POLY_MONITOR[Polygon Amoy<br/>Live RPC Monitoring]
    ICP_MONITOR[ICP Canister Health<br/>30s Refresh Intervals]
    BTC_MONITOR[Bitcoin Testnet<br/>Transaction Monitoring]
  end

  subgraph TESTING[End-to-End Testing]
    MINT_TESTS[Mint Function Testing]
    SUPABASE_TESTS[Supabase Integration Testing]
    UX_TESTS[UX/UI Process Testing]
    WEB3_TESTS[Web3 Functionality Testing]
  end

  subgraph LIVE_DATA[Live Data Sources]
    INFURA[Infura RPC Endpoints]
    POLYGON_RPC[Official Polygon RPC]
    ICP_CANISTERS[Deployed ICP Canisters]
    BTC_TESTNET[Bitcoin Testnet Network]
  end

  UI --> SETTINGS
  SETTINGS --> NETWORK_OPS
  NETWORK_OPS --> ETH_MONITOR
  NETWORK_OPS --> POLY_MONITOR
  NETWORK_OPS --> ICP_MONITOR
  NETWORK_OPS --> BTC_MONITOR
  NETWORK_OPS --> MINT_TESTS
  NETWORK_OPS --> SUPABASE_TESTS
  NETWORK_OPS --> UX_TESTS
  NETWORK_OPS --> WEB3_TESTS

  ETH_MONITOR <--> INFURA
  POLY_MONITOR <--> POLYGON_RPC
  ICP_MONITOR <--> ICP_CANISTERS
  BTC_MONITOR <--> BTC_TESTNET
```

## 3. ICP Canister Architecture

```mermaid
flowchart TB
  subgraph FRONTEND[Frontend Applications]
    AIGENT[Aigent Z UI]
    OPS[Network Ops Console]
    MARKET[21 Sats Market]
  end

  subgraph API_LAYER[API Layer]
    HTTP_API[HTTP API Gateway]
    CANDID[Candid Interface]
    SDK[Custom SDK]
  end

  subgraph CANISTERS[ICP Canisters - Live Deployed]
    PROOF[proof_of_state<br/>umunu-kh777-77774-qaaca-cai]
    BTC_SIGNER[btc_signer_psbt<br/>uxrrr-q7777-77774-qaaaq-cai]
    CROSS_CHAIN[cross_chain_service<br/>u6s2n-gx777-77774-qaaba-cai]
    EVM_RPC[evm_rpc<br/>uzt4z-lp777-77774-qaabq-cai]
  end

  subgraph EXTERNAL[External Networks]
    BTC_NET[Bitcoin Testnet]
    ETH_NET[Ethereum Sepolia]
    POLY_NET[Polygon Amoy]
    LZ_DVN[LayerZero DVN]
  end

  AIGENT --> HTTP_API
  OPS --> HTTP_API
  MARKET --> HTTP_API
  HTTP_API --> CANDID
  CANDID --> SDK
  SDK --> PROOF
  SDK --> BTC_SIGNER
  SDK --> CROSS_CHAIN
  SDK --> EVM_RPC

  PROOF --> BTC_NET
  BTC_SIGNER --> BTC_NET
  CROSS_CHAIN --> LZ_DVN
  EVM_RPC --> ETH_NET
  EVM_RPC --> POLY_NET
```

## 4. Cross-Chain Message Flow

```mermaid
sequenceDiagram
  participant UI as Aigent Z UI
  participant API as API Gateway
  participant XCS as CrossChainService
  participant DVN as LayerZero DVN
  participant EVM as EVM Chain
  participant MON as Network Ops

  UI->>API: Submit cross-chain message
  API->>XCS: Process message request
  XCS->>DVN: Submit to DVN network
  DVN->>DVN: Validator 1 attestation
  DVN->>DVN: Validator 2 attestation
  DVN->>XCS: Quorum reached (2+ attestations)
  XCS->>EVM: Execute on target chain
  EVM->>XCS: Transaction confirmation
  XCS->>API: Status update
  API->>UI: Success response
  XCS->>MON: Real-time monitoring update
  MON->>UI: Live status display
```

## 5. Bitcoin Anchoring Flow

```mermaid
sequenceDiagram
  participant APP as Application
  participant PROOF as Proof of State
  participant BTC as BTC Signer
  participant NET as Bitcoin Network
  participant OPS as Network Ops

  APP->>PROOF: issue_receipt(data)
  PROOF->>PROOF: Generate receipt with ID
  APP->>PROOF: batch()
  PROOF->>PROOF: Create Merkle tree batch
  APP->>PROOF: anchor()
  PROOF->>BTC: create_and_broadcast_anchor(root_hash)
  BTC->>BTC: Generate tECDSA signature
  BTC->>NET: Broadcast transaction
  NET->>BTC: Transaction confirmation
  BTC->>PROOF: Anchor confirmation
  PROOF->>APP: Anchoring complete
  PROOF->>OPS: Status update
  OPS->>APP: Live monitoring display
```

## 6. Data Flow Architecture

```mermaid
flowchart LR
  subgraph INPUT[Data Input]
    USER_DATA[User Data]
    BLOCKCHAIN_DATA[Blockchain Data]
    EXTERNAL_DATA[External APIs]
  end

  subgraph PROCESSING[Processing Layer]
    VALIDATION[Data Validation]
    ENCRYPTION[Encryption/Privacy]
    BATCHING[Batch Processing]
    SIGNING[Cryptographic Signing]
  end

  subgraph STORAGE[Storage Layer]
    METAQUBE[MetaQube<br/>Public Metadata]
    BLAKQUBE[BlakQube<br/>Private Encrypted]
    TOKENQUBE[TokenQube<br/>Token-gated Access]
  end

  subgraph OUTPUT[Output Destinations]
    BTC_ANCHOR[Bitcoin Anchors]
    EVM_CONTRACTS[EVM Contracts]
    ICP_STATE[ICP State]
    USER_INTERFACE[User Interface]
  end

  USER_DATA --> VALIDATION
  BLOCKCHAIN_DATA --> VALIDATION
  EXTERNAL_DATA --> VALIDATION
  VALIDATION --> ENCRYPTION
  ENCRYPTION --> BATCHING
  BATCHING --> SIGNING
  SIGNING --> METAQUBE
  SIGNING --> BLAKQUBE
  SIGNING --> TOKENQUBE
  METAQUBE --> BTC_ANCHOR
  BLAKQUBE --> EVM_CONTRACTS
  TOKENQUBE --> ICP_STATE
  ICP_STATE --> USER_INTERFACE
```

## 7. Network Operations Monitoring Architecture

```mermaid
flowchart TB
  subgraph MONITORING[Real-time Monitoring System]
    DASHBOARD["Network Ops Dashboard<br/>Settings → Network Ops"]
    HEALTH[Health Check System]
    ALERTS[Alert Management]
  end

  subgraph DATA_SOURCES[Live Data Sources]
    ETH_RPC["Ethereum Sepolia RPC<br/>via Infura"]
    POLY_RPC["Polygon Amoy RPC<br/>Official Endpoint"]
    ICP_HEALTH["ICP Canister Health<br/>4 Deployed Canisters"]
    BTC_STATUS[Bitcoin Testnet Status]
  end

  subgraph HOOKS["React Hooks - 30s Refresh"]
    USE_ETH[useEthereumSepolia]
    USE_POLY[usePolygonAmoy]
    USE_DVN[useDVNStatus]
    USE_BTC[useBTCStatus]
  end

  subgraph API_ROUTES[API Routes]
    ETH_API["/api/ops/ethereum/sepolia"]
    POLY_API["/api/ops/polygon/amoy"]
    DVN_API["/api/ops/dvn/status"]
    BTC_API["/api/ops/btc/status"]
  end

  DASHBOARD --> USE_ETH
  DASHBOARD --> USE_POLY
  DASHBOARD --> USE_DVN
  DASHBOARD --> USE_BTC
  USE_ETH --> ETH_API
  USE_POLY --> POLY_API
  USE_DVN --> DVN_API
  USE_BTC --> BTC_API
  ETH_API --> ETH_RPC
  POLY_API --> POLY_RPC
  DVN_API --> ICP_HEALTH
  BTC_API --> BTC_STATUS
  HEALTH --> ALERTS
  ALERTS --> DASHBOARD
```

## 8. Security Architecture

```mermaid
flowchart TB
  subgraph SECURITY[Security Layers]
    AUTH[Authentication<br/>Wallet-based]
    AUTHZ[Authorization<br/>Policy-based]
    CRYPTO[Cryptographic Proofs<br/>Bitcoin-backed]
    PRIVACY[Privacy Controls<br/>BlakQube Encryption]
  end

  subgraph CONTROLS[Access Controls]
    CAPABILITY[Capability Tokens<br/>Time-boxed Access]
    POLICY[Policy Engine<br/>Risk Assessment]
    AUDIT[Audit Trail<br/>Immutable Logging]
    COMPLIANCE[Compliance<br/>KYC/AML Integration]
  end

  subgraph VALIDATION[Validation Mechanisms]
    CONSENSUS[DVN Consensus<br/>Multi-validator]
    SIGNATURES[Multi-signature<br/>tECDSA Support]
    PROOFS[Merkle Proofs<br/>Selective Disclosure]
    VERIFICATION[Chain Verification<br/>Cross-chain Validation]
  end

  AUTH --> CAPABILITY
  AUTHZ --> POLICY
  CRYPTO --> SIGNATURES
  PRIVACY --> PROOFS
  CAPABILITY --> CONSENSUS
  POLICY --> AUDIT
  SIGNATURES --> VERIFICATION
  PROOFS --> COMPLIANCE
```

## 9. Integration Patterns

```mermaid
flowchart LR
  subgraph PATTERNS[Integration Patterns]
    API_FIRST[API-First Design]
    EVENT_DRIVEN[Event-Driven Architecture]
    MICROSERVICES[Microservices Pattern]
    GATEWAY[Gateway Pattern]
  end

  subgraph PROTOCOLS[Communication Protocols]
    HTTP_REST[HTTP/REST APIs]
    CANDID_RPC[Candid RPC]
    WEBSOCKETS[WebSocket Streams]
    BLOCKCHAIN[Blockchain Calls]
  end

  subgraph RELIABILITY[Reliability Patterns]
    CIRCUIT_BREAKER[Circuit Breaker]
    RETRY_LOGIC[Retry Logic]
    FALLBACK[Fallback Mechanisms]
    HEALTH_CHECKS[Health Checks]
  end

  API_FIRST --> HTTP_REST
  EVENT_DRIVEN --> WEBSOCKETS
  MICROSERVICES --> CANDID_RPC
  GATEWAY --> BLOCKCHAIN
  HTTP_REST --> CIRCUIT_BREAKER
  CANDID_RPC --> RETRY_LOGIC
  WEBSOCKETS --> FALLBACK
  BLOCKCHAIN --> HEALTH_CHECKS
```

## 10. Deployment Architecture

```mermaid
flowchart TB
  subgraph ENVIRONMENTS[Deployment Environments]
    DEV[Development<br/>Local dfx replica]
    TEST[Testing<br/>ICP Testnet]
    PROD[Production<br/>ICP Mainnet]
  end

  subgraph INFRASTRUCTURE[Infrastructure Components]
    CANISTERS[ICP Canisters<br/>Rust/Motoko]
    FRONTEND[Frontend Apps<br/>Next.js/React]
    MONITORING[Monitoring Stack<br/>Real-time Dashboards]
    DOCS[Documentation<br/>Docusaurus Site]
  end

  subgraph CICD[CI/CD Pipeline]
    BUILD[Build & Test]
    DEPLOY[Deploy Canisters]
    VERIFY[Verify Deployment]
    MONITOR[Monitor Health]
  end

  DEV --> BUILD
  TEST --> DEPLOY
  PROD --> VERIFY
  BUILD --> CANISTERS
  DEPLOY --> FRONTEND
  VERIFY --> MONITORING
  MONITOR --> DOCS
```

---

These diagrams provide a comprehensive view of the iQube Protocol architecture, highlighting the successful integration of Web3 Ops Console functionality into the Aigent Z application and the live testnet integrations across multiple blockchain networks.
