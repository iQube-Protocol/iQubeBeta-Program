---
id: qoyn
title: Qoyn Overview
sidebar_position: 4
---

Qoyn provides economic primitives and tokenization models used across the iQube ecosystem.

## Concepts

- Minting and activation flows
- Access control via TokenQube
- Registry visibility: Public vs Private

```mermaid
graph TD
  A[Template] -->|Mint| B[Registry: Private]
  B -->|Activate| C[Registry: Public]
  C --> D[Consumption / Forks]
```

