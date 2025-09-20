---
sidebar_position: 3
title: Glossary
description: Definitions of key terms and concepts in the iQube Protocol
---

# Glossary

This glossary provides definitions for key terms, concepts, and technologies used throughout the iQube Protocol and its documentation.

## Core Concepts

### iQube
A decentralized information asset that combines public metadata (MetaQube), private encrypted data (BlakQube), and access control mechanisms (TokenQube). iQubes serve as the fundamental unit of information in the protocol.

### MetaQube
The public, verifiable metadata component of an iQube. Contains information such as:
- Basic identification (name, creator, type)
- Scoring metrics (sensitivity, accuracy, verifiability, risk)
- Business model and pricing information
- Ownership and visibility settings

### BlakQube
The private, encrypted data component of an iQube. Contains sensitive or proprietary information that requires access control and decryption capabilities to view.

### TokenQube
The access control component that manages permissions for BlakQube data. Implements token-gated decryption and time-limited access grants through capability tokens.

## iQube Types

### DataQube
An iQube designed for alpha-numeric data representation, such as structured datasets, configuration files, or database records.

### ContentQube
An iQube designed for multi-modal content (blob) representation, including images, videos, documents, and other binary data.

### AgentQube
An iQube designed for AI agent performance and compliance tracking, containing agent behavior logs, performance metrics, and compliance records.

### ToolQube
An iQube containing tools, utilities, or software components that can be executed or integrated into other systems.

### ModelQube
An iQube containing machine learning models, training data, or AI-related assets.

## Operational Modes

### View Mode
A read-only operational mode for browsing and inspecting iQube metadata and structure without the ability to make modifications.

### Use Mode
An operational mode for creating instances from existing templates with controlled editing capabilities, maintaining template integrity while allowing customization.

### Edit Mode
A full editing mode that provides complete customization capabilities for creating and modifying iQube templates with dynamic field management.

### Decrypt Mode
A secure operational mode for accessing encrypted BlakQube data with proper authorization and capability token validation.

### Mint Mode
An operational mode for converting completed iQube templates into blockchain-backed instances with permanent, immutable registration.

### Activate Mode
An operational mode for enabling iQube functionality using secure activation codes, typically used for purchased or licensed iQubes.

## Visibility and Privacy

### Library (Private)
A local storage mechanism where iQubes are saved privately in the user's browser using localStorage. These iQubes are not visible to other users and exist only on the local device.

### Registry (Public)
A server-side storage mechanism where iQubes are publicly visible and discoverable by all users. Public registry iQubes can be viewed, forked, and used by anyone.

### Registry (Private)
A server-side storage mechanism where iQubes are stored on the server but visible only to the owner. These can potentially be activated to public status later.

### Template
A reusable iQube structure that defines fields, validation rules, and default values. Templates serve as blueprints for creating multiple instances.

### Instance
A specific implementation of a template with populated data fields. Instances maintain a relationship to their source template while allowing independent modifications.

## Blockchain and Technical Terms

### ICP (Internet Computer Protocol)
A blockchain network that hosts smart contracts (canisters) and provides decentralized computation capabilities. The iQube Protocol uses ICP for canister-based services.

### Canister
A smart contract on the Internet Computer Protocol that can store data and execute code. The iQube Protocol uses multiple canisters for different functions:
- proof_of_state: Receipt generation and anchoring
- btc_signer_psbt: Bitcoin transaction signing
- cross_chain_service: Cross-chain messaging
- evm_rpc: EVM chain interactions

### DVN (Decentralized Verifier Network)
A component of LayerZero protocol that provides decentralized message verification for cross-chain communications.

### Threshold ECDSA
A cryptographic technique that distributes private key operations across multiple parties, requiring a threshold number of participants to create signatures.

### PSBT (Partially Signed Bitcoin Transaction)
A Bitcoin standard for transactions that can be signed by multiple parties or devices, enabling complex multi-signature workflows.

### RPC (Remote Procedure Call)
A protocol that allows programs to execute procedures on remote systems. Used for blockchain interactions via endpoints like Infura or official network RPCs.

## Scoring and Assessment

### Sensitivity Score
A metric (1-10 scale) indicating how sensitive or private the data in an iQube is, with higher scores indicating more sensitive information.

### Accuracy Score
A metric (1-10 scale) indicating the accuracy and reliability of the data in an iQube, with higher scores indicating more accurate information.

### Verifiability Score
A metric (1-10 scale) indicating how easily the data in an iQube can be verified or validated, with higher scores indicating more verifiable information.

### Risk Score
A metric (1-10 scale) indicating the potential risk associated with using or sharing the iQube data, with higher scores indicating higher risk.

### Trust Score
A calculated composite metric derived from other scores that indicates the overall trustworthiness of an iQube.

### Reliability Index
A calculated composite metric that indicates the overall reliability and dependability of an iQube based on multiple factors.

## Business and Economic Terms

### Business Model
The economic model for an iQube, defining how it can be accessed or used:
- **Buy**: One-time purchase for permanent access
- **Sell**: Available for sale to others
- **Rent**: Temporary access for a fee
- **Lease**: Long-term access agreement
- **Subscribe**: Recurring payment for continued access
- **Stake**: Access through staking tokens
- **License**: Access under specific licensing terms
- **Donate**: Free access, supported by donations

### Duration of Rights
The time period for which access rights are granted:
- **Forever**: Permanent access
- **Per Use**: Single-use access
- **Per Minute/Hour/Day**: Time-based access
- **Custom**: Specific duration defined by creator

### Capability Token
A cryptographic token that grants specific, time-limited access to BlakQube data or functionality. Implements fine-grained access control.

## User Interface Terms

### Sidebar Navigation
The persistent navigation menu in Aigent Z that provides access to different sections:
- Dashboard
- Aigents (AI personas)
- iQube Operations
- Registry
- Settings

### Network Ops
The network operations interface for monitoring blockchain networks and ICP canisters, accessible through Settings → Network Ops.

### Testing Dashboard
A comprehensive testing interface for validating system health and application features, accessible at `/test`.

### Status Cards
Visual indicators in the Network Ops dashboard that show the real-time status of various services and integrations.

### Badge System
Visual indicators used throughout the interface to show iQube types, business models, visibility status, and other important attributes.

## AI and Agent Terms

### Aigent
An AI agent persona in the Aigent Z platform, designed for specific use cases and contexts. Examples include metaMe for personal data management.

### Context Transformation
The ability to modify AI agent behavior and focus through configuration panels, allowing users to customize agent responses and capabilities.

### RAG (Retrieval Augmented Generation)
A technique that enhances AI responses by retrieving relevant information from knowledge bases before generating responses.

### Semantic Vectorization
The process of converting textual or contextual information into mathematical vector representations for similarity search and analysis.

## Development and Technical Terms

### Monorepo
A repository structure that contains multiple related projects or applications in a single repository, used by the iQube Protocol for managing all components.

### Next.js
A React framework used for building the frontend applications in the iQube Protocol, providing server-side rendering and API routes.

### API Routes
Server-side endpoints in Next.js applications that handle HTTP requests and provide data to frontend components.

### React Hooks
JavaScript functions that allow React components to use state and lifecycle features, used extensively in the iQube Protocol frontend.

### TypeScript
A typed superset of JavaScript that provides static type checking and enhanced development experience.

### Tailwind CSS
A utility-first CSS framework used for styling the iQube Protocol user interfaces.

### shadcn-ui
A component library built on top of Radix UI and Tailwind CSS, providing pre-built UI components for the iQube Protocol.

## Network and Infrastructure Terms

### Testnet
A test blockchain network used for development and testing purposes without real economic value:
- **Ethereum Sepolia**: Ethereum test network
- **Polygon Amoy**: Polygon test network
- **Bitcoin Testnet**: Bitcoin test network

### Mainnet
The main blockchain network where real transactions occur with economic value.

### Infura
A blockchain infrastructure provider that offers RPC endpoints for connecting to Ethereum and other networks.

### RPC Endpoint
A network address that provides remote procedure call access to blockchain networks.

### Gas
The computational fee required to execute transactions on Ethereum and compatible networks.

### Cycles
The computational resource unit used on the Internet Computer Protocol for canister operations.

## Security and Cryptography Terms

### Zero-Knowledge Proof
A cryptographic method that allows one party to prove knowledge of information without revealing the information itself.

### Homomorphic Encryption
A form of encryption that allows computation on encrypted data without decrypting it first.

### Multi-Party Computation
A cryptographic technique that enables multiple parties to jointly compute a function over their inputs while keeping those inputs private.

### Differential Privacy
A mathematical framework for providing privacy guarantees when sharing statistical information about datasets.

### Capability-Based Security
A security model where access rights are represented by unforgeable tokens (capabilities) rather than access control lists.

## Operational Terms

### Health Check
A routine procedure or automated test that verifies the operational status of system components.

### Manual Refresh
A user-initiated action to immediately update data or status information, bypassing automatic refresh intervals.

### System Diagnostics
Tools and procedures for identifying, analyzing, and resolving system issues or performance problems.

### Response Time
The time it takes for a system to respond to a request, typically measured in milliseconds.

### Uptime
The percentage of time a system or service is operational and available for use.

### Failover
The process of automatically switching to a backup system when the primary system fails.

### Load Balancing
The distribution of network traffic or computational load across multiple servers or resources.

---

*This glossary is a living document that will be updated as the iQube Protocol evolves and new concepts are introduced.*
