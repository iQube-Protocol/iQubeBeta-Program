---
sidebar_position: 1
title: System Monitoring
description: Real-time monitoring of blockchain networks, ICP canisters, and system health
---

# System Monitoring

The iQube Protocol features comprehensive real-time monitoring across all integrated blockchain networks and ICP canisters. This monitoring system provides operators with complete visibility into system health, performance metrics, and operational status.

## Monitoring Architecture

### Live Data Integration
The monitoring system uses **only live testnet data** from real blockchain networks and deployed ICP canisters. No mock or demonstration data is used, ensuring accurate operational insights.

### Data Sources
- **Ethereum Sepolia**: Live RPC monitoring via Infura API
- **Polygon Amoy**: Live RPC monitoring via official Polygon endpoints
- **ICP Canisters**: Real-time canister health and method call monitoring
- **Cross-Chain Services**: LayerZero DVN message processing status
- **Bitcoin Testnet**: Anchor transaction monitoring and signing services

### Update Frequency
- **Automatic Refresh**: 30-second intervals for all blockchain data
- **Manual Refresh**: On-demand updates via dashboard controls
- **Real-Time Events**: Immediate updates for critical status changes

## Monitoring Dashboard

### Access Points
1. **Network Ops**: Primary monitoring interface in Aigent Z (Settings → Network Ops)
2. **Testing Dashboard**: Comprehensive validation interface at `/test`
3. **System Diagnostics**: Detailed troubleshooting tools within Network Ops

### Status Card System

Each monitored service displays comprehensive status information:

#### Status Indicators
- **🟢 Green Badge (Active)**: Service operational and responding normally
- **🔴 Red Badge (Inactive)**: Service experiencing issues or unreachable
- **⚡ Response Time**: API call latency in milliseconds
- **🕒 Last Updated**: Timestamp of most recent successful data retrieval

#### Data Display
- **Current Values**: Real-time metrics and status information
- **Historical Context**: Trend indicators where applicable
- **External Links**: Direct links to blockchain explorers for verification
- **Error Details**: Specific error messages when issues occur

## Monitored Services

### Ethereum Sepolia Testnet

#### Monitoring Scope
- **Latest Block Number**: Current blockchain height
- **Transaction Status**: Recent transaction processing
- **Chain Health**: Overall network operational status
- **RPC Connectivity**: Infura endpoint responsiveness

#### Key Metrics
- **Block Time**: Average time between blocks
- **Transaction Throughput**: Transactions per second
- **Gas Prices**: Current network congestion indicators
- **Node Sync Status**: RPC endpoint synchronization state

#### Alert Conditions
- **RPC Timeout**: Response time > 5 seconds
- **Stale Data**: Block data older than 2 minutes
- **Connection Failure**: Unable to reach Infura endpoint
- **Invalid Response**: Malformed or unexpected API responses

### Polygon Amoy Testnet

#### Monitoring Scope
- **Latest Block Number**: Current blockchain height
- **Transaction Status**: Recent transaction processing
- **Chain Health**: Overall network operational status
- **RPC Connectivity**: Official Polygon endpoint responsiveness

#### Key Metrics
- **Block Time**: Average time between blocks (typically 2 seconds)
- **Transaction Throughput**: Transactions per second
- **Gas Prices**: Current network congestion indicators
- **Validator Status**: Network validator health

#### Alert Conditions
- **RPC Timeout**: Response time > 3 seconds
- **Stale Data**: Block data older than 1 minute
- **Connection Failure**: Unable to reach Polygon RPC
- **Chain Halt**: No new blocks for extended period

### ICP Canister Health

#### Monitored Canisters
1. **proof_of_state** (ulvla-h7777-77774-qaacq-cai)
2. **btc_signer_psbt** (uxrrr-q7777-77774-qaaaq-cai)
3. **cross_chain_service** (u6s2n-gx777-77774-qaaba-cai)
4. **evm_rpc** (uzt4z-lp777-77774-qaabq-cai)

#### Health Metrics
- **Canister Status**: Running, stopped, or error states
- **Cycles Balance**: Available computational resources
- **Memory Usage**: Current memory consumption
- **Method Call Success Rate**: Percentage of successful calls
- **Response Time**: Average method call latency

#### Canister-Specific Monitoring

##### proof_of_state Canister
- **Receipt Generation**: Success rate of receipt issuance
- **Batch Processing**: Pending batch count and processing time
- **Anchor Operations**: Bitcoin anchoring transaction status
- **Data Integrity**: Verification of stored receipts and batches

##### btc_signer_psbt Canister
- **Address Generation**: Bitcoin testnet address creation
- **Transaction Signing**: PSBT signing success rate
- **Broadcast Status**: Transaction broadcast to Bitcoin network
- **Key Management**: Threshold ECDSA key health

##### cross_chain_service Canister
- **DVN Messages**: LayerZero message processing status
- **EVM Monitoring**: Cross-chain transaction tracking
- **Attestations**: Message attestation processing
- **Queue Status**: Pending message queue length

##### evm_rpc Canister
- **Chain Connectivity**: Connection status to configured EVM chains
- **RPC Health**: Success rate of RPC method calls
- **Block Synchronization**: Latest block tracking across chains
- **Transaction Receipts**: Receipt retrieval success rate

### Cross-Chain DVN Operations

#### Monitoring Scope
- **Message Queue**: Pending DVN messages awaiting processing
- **Attestation Status**: Message verification and attestation progress
- **EVM Transaction Monitoring**: Cross-chain transaction tracking
- **ICP Receipt Generation**: Receipt creation for processed messages

#### Key Metrics
- **Queue Length**: Number of pending messages
- **Processing Time**: Average message processing duration
- **Success Rate**: Percentage of successfully processed messages
- **Error Rate**: Failed message processing percentage

### Bitcoin Testnet Integration

#### Monitoring Scope
- **Anchor Transactions**: Bitcoin testnet transaction creation and broadcast
- **Address Generation**: Bitcoin testnet address creation success
- **Balance Monitoring**: Bitcoin testnet balance tracking
- **Network Connectivity**: Bitcoin testnet node accessibility

#### Key Metrics
- **Transaction Confirmation**: Average confirmation time
- **Fee Estimation**: Current network fee rates
- **UTXO Management**: Unspent transaction output tracking
- **Signing Success**: Threshold signature success rate

## Performance Monitoring

### Response Time Tracking
- **Target Response Times**: < 2 seconds for all API calls
- **Performance Alerts**: Warnings when response times exceed thresholds
- **Trend Analysis**: Historical performance tracking
- **Bottleneck Identification**: Identify slow components

### Availability Monitoring
- **Uptime Tracking**: Service availability percentages
- **Downtime Alerts**: Immediate notification of service failures
- **Recovery Monitoring**: Time to recovery after failures
- **SLA Compliance**: Service level agreement adherence

### Resource Utilization
- **ICP Cycles**: Monitor canister computational resource usage
- **API Rate Limits**: Track usage against provider limits
- **Memory Usage**: Monitor memory consumption across services
- **Network Bandwidth**: Track data transfer volumes

## Alerting and Notifications

### Alert Levels
- **🔴 Critical**: Service completely unavailable
- **🟡 Warning**: Performance degradation or intermittent issues
- **🔵 Info**: Status changes or maintenance notifications

### Alert Channels
- **Dashboard Indicators**: Visual status changes in monitoring interface
- **Browser Notifications**: Real-time alerts for critical issues
- **Console Logging**: Detailed error information in browser console
- **Error Reporting**: Structured error messages with troubleshooting guidance

### Alert Conditions
- **Service Unavailability**: Complete loss of service connectivity
- **Performance Degradation**: Response times exceeding thresholds
- **Data Staleness**: Data older than acceptable refresh intervals
- **Error Rate Increases**: Elevated failure rates for API calls

## Monitoring Best Practices

### Daily Operations
1. **Morning Health Check**: Review all status cards for overnight issues
2. **Performance Review**: Check response times and identify trends
3. **Error Log Analysis**: Review any errors or warnings from previous day
4. **Capacity Planning**: Monitor resource usage trends

### Weekly Operations
1. **Trend Analysis**: Review weekly performance patterns
2. **Capacity Review**: Assess resource utilization trends
3. **Alert Tuning**: Adjust alert thresholds based on operational experience
4. **Documentation Updates**: Update procedures based on lessons learned

### Monthly Operations
1. **Performance Reporting**: Generate monthly performance summaries
2. **Capacity Planning**: Plan for resource scaling needs
3. **Monitoring Enhancement**: Identify additional metrics to track
4. **Process Improvement**: Refine monitoring and alerting procedures

## Integration with External Monitoring

### Blockchain Explorers
- **Etherscan (Sepolia)**: Verify Ethereum transaction data
- **PolygonScan (Amoy)**: Verify Polygon transaction data
- **IC Dashboard**: Monitor ICP canister health and performance
- **Bitcoin Testnet Explorer**: Verify Bitcoin transaction data

### External Status Pages
- **Infura Status**: Monitor RPC provider availability
- **Polygon Status**: Monitor network health and maintenance
- **Internet Computer Status**: Monitor IC network health
- **Third-Party Services**: Monitor any additional service dependencies

### API Monitoring Tools
- **Response Time Monitoring**: Track API performance across providers
- **Uptime Monitoring**: Monitor service availability
- **Error Rate Tracking**: Monitor API error rates and patterns
- **Rate Limit Monitoring**: Track usage against provider limits

## Troubleshooting Integration

### Diagnostic Tools
- **Manual Refresh**: Force immediate data updates
- **Connection Testing**: Verify connectivity to all services
- **Error Analysis**: Detailed error message interpretation
- **Performance Profiling**: Identify performance bottlenecks

### Recovery Procedures
- **Service Restart**: Procedures for restarting failed services
- **Failover**: Switch to backup services when available
- **Data Recovery**: Restore from backups when necessary
- **Escalation**: When to escalate issues to development team

---

*Comprehensive system monitoring provides the operational foundation for reliable iQube Protocol operations, ensuring high availability and performance across all integrated services.*
