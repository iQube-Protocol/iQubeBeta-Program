---
sidebar_position: 4
title: Network Operations (Network Ops)
description: Monitor and manage blockchain networks and ICP canisters through the Network Ops interface
---

# Network Operations (Network Ops)

The Network Operations interface, accessible through the **Settings → Network Ops** menu in Aigent Z, provides real-time monitoring and management of blockchain networks and ICP canisters. This interface has migrated from the standalone iQube Operations console and is now integrated directly into the Aigent Z application.

## Overview

Network Ops serves as the central command center for:
- **Live blockchain monitoring** across multiple testnets
- **ICP canister health** and status tracking  
- **Cross-chain operations** and DVN message processing
- **System diagnostics** and troubleshooting tools
- **Testing infrastructure** for validation workflows

## Accessing Network Ops

1. Open the Aigent Z application
2. Navigate to **Settings** in the sidebar
3. Click **Network Ops** in the Settings submenu
4. The Network Ops dashboard will load with real-time data

:::info Migration Note
Network Ops was previously available as a standalone "iQube Operations console" but has been migrated into the Aigent Z application for better integration and user experience.
:::

## Dashboard Components

### Status Cards Overview

The Network Ops dashboard displays real-time status cards for all integrated services:

#### BTC Anchor
- **Purpose**: Bitcoin testnet anchor transaction monitoring
- **Status Indicators**: Green (Active) / Red (Inactive)
- **Data**: Latest anchor transactions, confirmation status
- **Response Time**: API call latency tracking

#### ICP Canister Health  
- **Purpose**: Monitor deployed ICP canisters
- **Canisters Tracked**: proof_of_state, btc_signer_psbt, cross_chain_service, evm_rpc
- **Status Indicators**: Operational health and response times
- **Data**: Canister cycles, memory usage, method call success rates

#### Cross-Chain DVN
- **Purpose**: LayerZero DVN message processing status
- **Integration**: Direct connection to cross_chain_service canister
- **Data**: Message queue status, attestation processing, verification results
- **Monitoring**: EVM transaction monitoring and ICP receipt generation

#### Ethereum Sepolia
- **Purpose**: Live Ethereum testnet monitoring
- **RPC Provider**: Infura endpoint integration
- **Data**: Latest block number, transaction status, chain health
- **Refresh Rate**: 30-second automatic updates
- **Explorer Links**: Direct links to Etherscan for transaction details

#### Polygon Amoy  
- **Purpose**: Live Polygon testnet monitoring
- **RPC Provider**: Official Polygon RPC endpoints
- **Data**: Latest block number, transaction status, chain health
- **Refresh Rate**: 30-second automatic updates
- **Explorer Links**: Direct links to PolygonScan for transaction details

#### Solana Integration
- **Purpose**: Solana network integration status
- **Features**: Address generation, airdrop testing, balance monitoring
- **Status**: Development and testing phase

### Status Indicators

#### Green Badge (Active)
- Service is operational and responding normally
- All API calls successful within acceptable timeframes
- Data is current and accurate

#### Red Badge (Inactive)  
- Service is experiencing issues or unreachable
- API calls failing or timing out
- Data may be stale or unavailable

#### Response Time Display
- Shows API call latency in milliseconds
- Helps identify performance issues
- Updated with each data refresh

#### Last Updated Timestamp
- Shows when data was last successfully retrieved
- Indicates data freshness
- Helps identify stale data issues

## System Diagnostics

### Manual Refresh
- **Purpose**: Force immediate data refresh for all monitoring cards
- **Usage**: Click "Manual Refresh" button to update all status cards
- **Benefit**: Bypass automatic refresh intervals for immediate updates

### Testing Dashboard Access
- **Purpose**: Quick access to comprehensive testing interface
- **Location**: Available via "Testing Dashboard" button
- **Features**: System health tests, application feature validation

### Network Validation
- **Purpose**: Verify connectivity to all external services
- **Process**: Automated connectivity checks to RPC endpoints and canisters
- **Results**: Detailed success/failure reporting with error details

### Error Reporting
- **Purpose**: Detailed error messages and troubleshooting information
- **Display**: Error details shown in status cards when issues occur
- **Information**: Specific error codes, messages, and suggested resolutions

## Live Data Integration

### No Mock Data Policy
:::warning Important
Network Ops uses **only live testnet data** from real blockchain networks and deployed ICP canisters. No mock or demonstration data is used in the monitoring systems.
:::

### Data Sources

#### Ethereum Sepolia
- **RPC Endpoint**: Infura API with authenticated access
- **Data Retrieved**: Latest block numbers, transaction hashes, chain status
- **Update Frequency**: 30-second intervals
- **Fallback**: Error handling with "—" display for failed calls

#### Polygon Amoy
- **RPC Endpoint**: Official Polygon testnet RPC
- **Data Retrieved**: Latest block numbers, transaction hashes, chain status  
- **Update Frequency**: 30-second intervals
- **Fallback**: Error handling with "—" display for failed calls

#### ICP DVN Canister
- **Canister ID**: u6s2n-gx777-77774-qaaba-cai (cross_chain_service)
- **Data Retrieved**: EVM TX status, ICP receipts, lock status, unlock height
- **Update Frequency**: 30-second intervals
- **Integration**: Direct canister method calls via HTTP API

## Operational Procedures

### Daily Health Check
1. Access Network Ops dashboard
2. Verify all status cards show green (active) indicators
3. Check response times are within acceptable ranges (< 2 seconds)
4. Use "Manual Refresh" if any cards show stale data
5. Navigate to Testing Dashboard for comprehensive validation

### Troubleshooting Network Issues

#### Red Status Indicators
1. Check network connectivity to external RPC endpoints
2. Verify API keys and environment variables are configured
3. Review browser console for detailed error messages
4. Use Testing Dashboard to isolate specific service failures

#### Slow Response Times
1. Monitor external RPC endpoint performance
2. Check for rate limiting or API quota issues  
3. Verify canister health and IC network status
4. Consider switching to backup RPC endpoints if available

#### Data Inconsistencies
1. Force manual refresh to clear cached data
2. Verify external blockchain explorer data matches displayed values
3. Check for API response format changes from external providers
4. Cross-reference data with multiple sources

### Emergency Response

#### Complete System Failure
1. Check application server status and logs
2. Verify all environment variables are properly configured
3. Restart development server if necessary
4. Escalate to development team if issues persist

#### Partial Service Failures
1. Use Testing Dashboard to identify specific failing services
2. Check individual API route responses in browser developer tools
3. Verify external service availability and status pages
4. Implement fallback procedures for critical operations

## Integration with Testing Dashboard

Network Ops provides seamless integration with the comprehensive Testing Dashboard:

### Access Methods
- Click "Testing Dashboard" button in Network Ops
- Direct navigation to `/test` route
- Quick access for immediate system validation

### Testing Capabilities
- **System Health Tests**: Validate all integrated services
- **Application Feature Tests**: End-to-end workflow validation
- **Custom Testing**: Input specific iQube IDs for targeted testing
- **Real-time Results**: Live status updates with detailed reporting

## Best Practices

### Regular Monitoring
- Perform daily health checks using the dashboard
- Monitor response time trends to identify performance degradation
- Review error logs regularly for early issue detection
- Subscribe to status pages for external services (Infura, Polygon, IC)

### Maintenance Procedures
- Regularly update and rotate external service API keys
- Monitor ICP canister deployments and version updates
- Stay informed of testnet RPC endpoint modifications
- Ensure development, staging, and production environments are synchronized

### Documentation
- Keep operational procedures updated as systems evolve
- Document any custom configurations or workarounds
- Maintain contact information for external service providers
- Record resolution procedures for common issues

## Future Enhancements

The Network Ops interface continues to evolve with planned enhancements:

- **Additional Blockchain Networks**: Integration with more testnets and mainnets
- **Advanced Analytics**: Historical performance tracking and trend analysis
- **Automated Alerting**: Proactive notifications for system issues
- **Custom Dashboards**: User-configurable monitoring views
- **API Management**: Enhanced API key rotation and management tools

---

*Network Ops provides the operational foundation for reliable iQube Protocol operations across all integrated blockchain networks and services.*
