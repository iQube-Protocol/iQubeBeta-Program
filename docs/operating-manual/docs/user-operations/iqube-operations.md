---
sidebar_position: 2
title: iQube Operations
description: Comprehensive guide to creating, managing, and operating iQubes through all operational modes
---

# iQube Operations

iQube Operations form the core of the iQube Protocol, providing comprehensive tools for creating, managing, and interacting with decentralized information assets. The operations interface supports multiple modes to handle different aspects of the iQube lifecycle.

## Understanding iQubes

### Core Components

#### MetaQube (Public Metadata)
- **Purpose**: Public, verifiable metadata about the iQube
- **Visibility**: Always visible to all users
- **Content**: Basic information, scoring, business model, ownership details
- **Immutability**: Cannot be changed once minted to blockchain

#### BlakQube (Private Data)
- **Purpose**: Private, encrypted data content
- **Visibility**: Only accessible with proper decryption keys
- **Content**: Sensitive data fields, personal information, proprietary content
- **Encryption**: Multiple encryption levels and access controls

#### TokenQube (Access Control)
- **Purpose**: Token-gated data decryption and access management
- **Functionality**: Controls who can decrypt and access BlakQube data
- **Integration**: Works with wallet-based authentication systems

### iQube Types

#### DataQube
- **Purpose**: Alpha-numeric data representation
- **Use Cases**: Structured data, databases, configuration files
- **Format**: JSON, CSV, XML, or custom structured formats

#### ContentQube
- **Purpose**: Multi-modal content (blob) representation  
- **Use Cases**: Images, videos, documents, audio files
- **Format**: Binary data with metadata descriptors

#### AgentQube
- **Purpose**: AI agent performance and compliance tracking
- **Use Cases**: Agent behavior logs, performance metrics, compliance records
- **Format**: Structured logs with performance indicators

## Operational Modes

The iQube Operations interface provides six distinct modes, each optimized for specific tasks:

### View Mode

**Purpose**: Browse and inspect iQube metadata and structure without modification.

#### Features
- **Read-Only Access**: Safe inspection without accidental changes
- **Complete Metadata Display**: Full MetaQube information visibility
- **Score Visualization**: Risk, accuracy, verifiability, and sensitivity scores
- **Instance Information**: Template relationships and instance counts
- **Source Indicators**: Visual icons showing data source types

#### Use Cases
- Evaluating iQubes before use or purchase
- Auditing existing iQube structures
- Understanding template-instance relationships
- Reviewing scoring and risk assessments

#### Interface Elements
- **Header Information**: iQube name, type, and instance details
- **MetaQube Section**: Collapsible public metadata display
- **BlakQube Section**: Encrypted data field overview (if accessible)
- **Scoring Display**: Visual indicators for all score types
- **Action Buttons**: Navigate to other modes or external links

### Use Mode

**Purpose**: Populate iQube instances from templates with controlled editing capabilities.

#### Features
- **Template-Based Creation**: Start with pre-designed structures
- **Controlled Editing**: Limited modifications to maintain template integrity
- **Instance Generation**: Create numbered instances (e.g., "3 of 21")
- **Field Population**: Fill in template fields with specific data
- **Validation**: Real-time validation of required fields

#### Workflow
1. **Select Template**: Choose from available templates
2. **Configure Instance**: Set instance-specific parameters
3. **Populate Fields**: Fill in MetaQube and BlakQube data
4. **Validate**: Ensure all required fields are complete
5. **Save**: Store as draft or proceed to minting

#### Interface Elements
- **Template Selector**: Dropdown or modal for template selection
- **Instance Counter**: Display current instance number and total
- **Field Forms**: Structured input forms for data entry
- **Progress Indicators**: Show completion status
- **Save Options**: Draft to Library or mint to Registry

### Edit Mode

**Purpose**: Full template editing with dynamic field management and complete customization.

#### Features
- **Complete Customization**: Full control over iQube structure
- **Dynamic Field Management**: Add, remove, and modify fields
- **Schema Definition**: Define BlakQube data structures
- **Validation Rules**: Set field requirements and constraints
- **Version Control**: Track template modifications and updates

#### Capabilities
- **MetaQube Editing**: Modify all public metadata fields
- **BlakQube Schema**: Design private data structures
- **Scoring Configuration**: Set and adjust all scoring parameters
- **Business Model**: Configure pricing and access rights
- **Field Types**: Support for various data types and formats

#### Interface Elements
- **Field Editor**: Dynamic form builder interface
- **Schema Designer**: Visual schema creation tools
- **Validation Builder**: Rule configuration interface
- **Preview Mode**: Real-time preview of changes
- **Version History**: Track and revert changes

### Decrypt Mode

**Purpose**: Secure BlakQube data decryption with proper authorization.

#### Features
- **Secure Access**: Wallet-based authentication for decryption
- **Granular Permissions**: Field-level access control
- **Audit Trail**: Complete logging of access attempts
- **Time-Limited Access**: Temporary decryption capabilities
- **Multi-Level Encryption**: Support for various encryption schemes

#### Security Measures
- **Capability Tokens**: Time-boxed access grants
- **Wallet Verification**: Cryptographic proof of ownership
- **Access Logging**: Immutable record of all access attempts
- **Encryption Status**: Clear indicators of data protection levels

#### Interface Elements
- **Authentication Panel**: Wallet connection and verification
- **Decryption Controls**: Buttons for accessing encrypted fields
- **Status Indicators**: Encryption/decryption state display
- **Access History**: Log of previous decryption attempts
- **Security Warnings**: Clear indication of sensitive operations

### Mint Mode

**Purpose**: Convert completed templates to blockchain-backed instances.

#### Features
- **Blockchain Integration**: Permanent recording on distributed ledgers
- **Visibility Control**: Choose between Public and Private registry
- **Immutable Records**: Cannot be modified once minted
- **Ownership Proof**: Cryptographic ownership verification
- **Global Discovery**: Public iQubes become discoverable

#### Minting Process
1. **Validation Check**: Ensure all required fields are complete
2. **Visibility Selection**: Choose Public or Private registry
3. **Confirmation Dialog**: Clear warning about irreversible action
4. **Blockchain Transaction**: Submit to appropriate blockchain
5. **Registry Update**: Update server-side registry records

#### Visibility Options
- **Public Registry**: Visible to all users, forkable, discoverable
- **Private Registry**: Visible only to owner, can be activated later
- **Library (Local)**: Browser-only storage, not minted to blockchain

#### Interface Elements
- **Validation Status**: Pre-mint requirement checking
- **Visibility Selector**: Public/Private choice with warnings
- **Confirmation Dialog**: Clear explanation of consequences
- **Transaction Status**: Real-time minting progress
- **Success Confirmation**: Completion notification with links

### Activate Mode

**Purpose**: Activate existing iQube instances with secure activation codes.

#### Features
- **Code-Based Activation**: Secure activation using unique codes
- **Instance Verification**: Confirm instance authenticity
- **Access Granting**: Enable full functionality for activated instances
- **Audit Trail**: Record activation events
- **Batch Activation**: Activate multiple instances simultaneously

#### Activation Process
1. **Code Entry**: Input activation code or scan QR code
2. **Verification**: Validate code against blockchain records
3. **Instance Lookup**: Retrieve associated iQube instance
4. **Access Grant**: Enable full operational capabilities
5. **Confirmation**: Provide activation success confirmation

#### Interface Elements
- **Code Input**: Text field or QR scanner for activation codes
- **Verification Status**: Real-time validation feedback
- **Instance Preview**: Display of iQube being activated
- **Activation History**: Log of previous activations
- **Batch Controls**: Multi-instance activation tools

## Advanced Features

### Template & Instance Management

#### Template Creation
- **Custom Fields**: Define any data structure needed
- **Reusable Design**: Create templates for repeated use
- **Version Control**: Track template evolution over time
- **Sharing Options**: Make templates available to others

#### Instance Generation
- **Numbered Instances**: Automatic instance counting (e.g., "5 of 100")
- **Template Inheritance**: Maintain connection to source template
- **Independent Modification**: Customize instances while preserving template
- **Batch Creation**: Generate multiple instances efficiently

#### Provenance Tracking
- **Complete Audit Trail**: Track all modifications and access
- **Template Lineage**: Understand template-instance relationships
- **Change History**: Detailed log of all modifications
- **Ownership Chain**: Track ownership transfers and permissions

### Validation System

#### Field Validation
- **Required Fields**: Ensure critical data is provided
- **Data Types**: Validate appropriate data formats
- **Range Checking**: Verify numeric values within acceptable ranges
- **Pattern Matching**: Ensure text fields match required patterns

#### Business Rules
- **Scoring Validation**: Ensure scores are within valid ranges (1-10)
- **Consistency Checks**: Verify related fields are compatible
- **Completeness**: Ensure all sections are properly filled
- **Security Validation**: Check encryption and access requirements

### Integration Features

#### Blockchain Connectivity
- **Multi-Chain Support**: Ethereum, Polygon, ICP, and others
- **Transaction Monitoring**: Real-time transaction status tracking
- **Gas Optimization**: Efficient transaction structuring
- **Failure Recovery**: Robust error handling and retry logic

#### External Data Sources
- **API Integration**: Connect to external data providers
- **File Upload**: Support for various file formats
- **Database Connectivity**: Direct database integration options
- **Real-Time Sync**: Automatic updates from external sources

## Best Practices

### Workflow Optimization
1. **Start with Templates**: Use existing templates when possible
2. **Draft in Library**: Save work-in-progress locally before minting
3. **Validate Early**: Check requirements before final submission
4. **Plan Visibility**: Carefully consider Public vs Private implications

### Data Management
1. **Organize Fields**: Use clear, descriptive field names
2. **Secure Sensitive Data**: Use BlakQube encryption for private information
3. **Regular Backups**: Export important iQubes regularly
4. **Version Control**: Track significant changes to templates

### Security Considerations
1. **Access Control**: Carefully manage who can decrypt BlakQube data
2. **Audit Trails**: Monitor access to sensitive information
3. **Encryption Keys**: Securely manage decryption capabilities
4. **Public Exposure**: Understand implications of public minting

## Troubleshooting

### Common Issues

#### Validation Failures
- **Missing Required Fields**: Complete all mandatory sections
- **Invalid Data Formats**: Ensure data matches expected types
- **Score Range Errors**: Keep all scores within 1-10 range
- **Incomplete Sections**: Fill all required MetaQube and BlakQube fields

#### Minting Problems
- **Network Issues**: Check blockchain connectivity
- **Insufficient Funds**: Ensure adequate gas/fees for transactions
- **Validation Errors**: Resolve all validation issues before minting
- **Timeout Issues**: Retry failed transactions after network recovery

#### Access Issues
- **Decryption Failures**: Verify wallet connectivity and permissions
- **Missing Capabilities**: Ensure proper access tokens are available
- **Network Connectivity**: Check connection to ICP canisters and RPC endpoints
- **Authentication Problems**: Reconnect wallet or refresh session

---

*iQube Operations provide the complete toolkit for managing decentralized information assets throughout their entire lifecycle, from creation to activation and beyond.*
