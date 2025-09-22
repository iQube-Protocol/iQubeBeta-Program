# Cross-Platform BTC Testing Guide

## Overview

This guide covers testing Bitcoin integration and cross-chain functionality in the iQube Protocol, including EVM transaction monitoring, DVN (Decentralized Verifier Network) operations, and BTC anchoring processes.

## Prerequisites

### Environment Setup
- Node.js 18+ and npm
- MetaMask browser extension
- Access to Bitcoin testnet and Polygon Amoy testnet
- ICP canister environment configured

### Required Environment Variables
```bash
# ICP Canister IDs (in .env.local)
NEXT_PUBLIC_PROOF_OF_STATE_CANISTER_ID=ulvla-h7777-77774-qaacq-cai
NEXT_PUBLIC_BTC_SIGNER_CANISTER_ID=uxrrr-q7777-77774-qaaaq-cai
NEXT_PUBLIC_CROSS_CHAIN_SERVICE_CANISTER_ID=u6s2n-gx777-77774-qaaba-cai
NEXT_PUBLIC_EVM_RPC_CANISTER_ID=uzt4z-lp777-77774-qaabq-cai

# BTC Testnet API
NEXT_PUBLIC_RPC_BTC_TESTNET=https://mempool.space/testnet/api

# EVM Network RPCs
NEXT_PUBLIC_INFURA_PROJECT_ID=your_infura_project_id
```

## Testing Procedures

### 1. Operations Console Access

1. **Start the development server:**
   ```bash
   cd apps/aigent-z
   npm run dev -- -p 3007
   ```

2. **Navigate to Operations Console:**
   - Open browser to `http://localhost:3007/ops`
   - Verify all network cards are visible and showing status

### 2. Network Connectivity Testing

#### ICP Canister Health
- **Expected**: Green status indicators for all canisters
- **Test**: Click refresh buttons to verify connectivity
- **Troubleshoot**: Check canister IDs in environment variables

#### BTC Testnet Monitoring
- **Expected**: Current block height displayed (4689000+)
- **Test**: Verify RPC endpoint shows correct API (mempool.space or blockstream.info)
- **Fallback**: Should automatically switch to blockstream.info if mempool.space fails

#### EVM Networks (Sepolia/Amoy)
- **Expected**: Live block numbers and transaction counts
- **Test**: Verify chain IDs (11155111 for Sepolia, 80002 for Amoy)
- **Troubleshoot**: Check Infura project ID if failing

### 3. EVM Transaction Creation and Monitoring

#### MetaMask Transaction Creation
1. **Setup MetaMask:**
   - Add Polygon Amoy testnet (Chain ID: 80002)
   - Ensure you have test MATIC tokens

2. **Create Test Transaction:**
   - In DVN card, select "Amoy (80002)" from dropdown
   - Paste any valid transaction hash or leave empty
   - Click "Create Test TX (MetaMask)"
   - **Expected**: MetaMask opens with transaction prompt
   - **Expected**: Self-transfer transaction (0 value)

3. **Monitor Transaction:**
   - After transaction confirmation, click "Monitor"
   - **Expected**: POST to `/api/ops/dvn/monitor` returns success
   - **Expected**: Message ID appears (format: `local:0x...` for fallback mode)

#### DVN Transaction Monitoring
1. **Verify Monitoring Status:**
   - Transaction should appear in "Tracked ID" field
   - Status should show as "confirmed" after block confirmation
   - **Fallback Mode**: If DVN canister unavailable, uses local RPC queries

2. **Test Attestation (Optional):**
   - Use same transaction hash for Validator ID and Signature Hex
   - Click "Submit Attestation"
   - **Expected**: Error message about canister availability (normal in fallback mode)

### 4. BTC Anchor Testing

#### Anchor Status Verification
1. **Check Anchor Card:**
   - **Expected**: Green status (canister reachable)
   - **Expected**: "Last Anchor: —" (no anchors created yet)
   - **Expected**: "Pending: 0"
   - **Expected**: "Details: no batches"

2. **Test Anchor Creation:**
   - Click "Anchor" button
   - **Expected**: Detailed error message explaining missing methods
   - **Message Should Include**: Available methods (get_batches, get_pending_count)
   - **Note**: This is expected until canister redeployment

#### BTC Testnet Integration
1. **Verify Block Height:**
   - **Expected**: Live Bitcoin testnet block height
   - **Expected**: Updates every 30 seconds
   - **Fallback**: Should show cached data during API outages

2. **API Reliability:**
   - **Primary**: mempool.space/testnet
   - **Fallback**: blockstream.info/testnet
   - **Caching**: 10-minute client-side cache for resilience

### 5. Cross-Chain Flow Testing

#### End-to-End Transaction Flow
1. **Create EVM Transaction** → DVN monitors it
2. **DVN Processing** → Should create receipt in proof_of_state (currently fallback mode)
3. **Batch Creation** → Groups receipts for anchoring (pending canister update)
4. **BTC Anchoring** → Creates Bitcoin transaction (pending canister update)

#### Current Status (September 2025)
- ✅ **EVM Transaction Creation**: Working with MetaMask
- ✅ **DVN Monitoring**: Working with fallback system
- ✅ **Transaction Persistence**: Survives page refreshes
- ⚠️ **BTC Anchoring**: Diagnostic mode (needs canister redeployment)

## Known Issues and Workarounds

### DVN Canister Dependencies
- **Issue**: `canister_not_found` errors on update calls
- **Cause**: DVN compiled with outdated dependency canister IDs
- **Workaround**: Fallback system uses direct RPC queries
- **Resolution**: Redeploy DVN with correct dependencies

### BTC Anchor Methods Missing
- **Issue**: proof_of_state canister missing anchor methods
- **Available**: get_batches, get_pending_count (query methods only)
- **Missing**: issue_receipt, batch, anchor (update methods)
- **Resolution**: Redeploy canister with full functionality

### API Rate Limiting
- **Issue**: Occasional timeouts from external APIs
- **Mitigation**: Dual-API approach with automatic fallback
- **Caching**: Client-side persistence for resilience

## Troubleshooting Guide

### Common Error Messages

#### "Network error: NetworkError when attempting to fetch resource"
- **Cause**: Development server crashed or disconnected
- **Fix**: Restart with `npm run dev -- -p 3007`

#### "Anchor functionality not yet implemented"
- **Cause**: Expected behavior - canister needs redeployment
- **Info**: Shows available methods for diagnostics

#### "Internal JSON-RPC error" from MetaMask
- **Cause**: Network issues or insufficient gas
- **Fix**: Check network connection, ensure test tokens available

#### BTC Testnet showing "—" for block height
- **Cause**: Both APIs temporarily unavailable
- **Fix**: Wait for automatic retry or check cached value

### Performance Expectations
- **Page Load**: < 3 seconds for initial load
- **Network Updates**: 30-second refresh intervals
- **Transaction Monitoring**: Real-time updates after confirmation
- **Fallback Activation**: < 5 seconds when primary services fail

## Success Criteria

### Functional Requirements
- [ ] All network cards show green status
- [ ] MetaMask transaction creation works
- [ ] DVN monitoring tracks transactions (with or without fallback)
- [ ] BTC testnet shows live block height
- [ ] Transaction hashes persist across page refreshes
- [ ] Error messages are clear and actionable

### Performance Requirements
- [ ] Page loads within 3 seconds
- [ ] Network data updates every 30 seconds
- [ ] Fallback systems activate within 5 seconds
- [ ] No console errors during normal operation

### User Experience Requirements
- [ ] Clear status indicators for all services
- [ ] Informative error messages with next steps
- [ ] Graceful degradation when services unavailable
- [ ] Persistent transaction state across sessions

## Future Enhancements

### Planned Improvements
1. **Full DVN Integration**: Deploy canisters with correct dependencies
2. **Complete BTC Anchoring**: Enable end-to-end anchor creation
3. **Enhanced Monitoring**: Add transaction history and audit trails
4. **Automated Testing**: E2E test suite for cross-chain flows
5. **Performance Monitoring**: Metrics and alerting for service health

### Integration Roadmap
1. **Phase 1**: Fix canister dependencies and redeploy
2. **Phase 2**: Enable full anchor workflow
3. **Phase 3**: Add advanced monitoring and analytics
4. **Phase 4**: Implement automated failover and recovery

---

**Last Updated**: September 22, 2025
**Version**: 1.0
**Status**: Active Development with Fallback Systems
