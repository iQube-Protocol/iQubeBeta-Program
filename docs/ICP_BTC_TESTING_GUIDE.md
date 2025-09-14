# iQube ICP/BTC Integration Testing Guide

## Overview

This guide covers testing the complete ICP/BTC integration stack including:
- ICP Rust canisters for proof-of-state and cross-chain services
- Bitcoin testnet anchoring via tECDSA
- EVM cross-chain verification
- LayerZero DVN integration
- SDK integration with frontend apps

## Prerequisites

### 1. Install dfx (Internet Computer SDK)

```bash
# Option 1: Direct download
curl -fsSL https://github.com/dfinity/sdk/releases/download/0.29.1/dfx-x86_64-apple-darwin.tar.gz -o /tmp/dfx.tar.gz
cd /tmp && tar -xzf dfx.tar.gz
mkdir -p "$HOME/Library/Application Support/org.dfinity.dfx/bin"
mv dfx "$HOME/Library/Application Support/org.dfinity.dfx/bin/"
echo 'export PATH="$HOME/Library/Application Support/org.dfinity.dfx/bin:$PATH"' >> ~/.zshenv
source ~/.zshenv

# Option 2: Homebrew
brew install dfinity/tap/dfx

# Verify installation
dfx --version
```

### 2. Environment Setup

```bash
cd /Users/hal1/CascadeProjects/iQubeBeta-Program

# Install dependencies
pnpm install

# Build all packages
pnpm build
```

## Testing Workflow

### Phase 1: ICP Canister Deployment

#### 1.1 Start Local ICP Replica

```bash
# Start dfx replica in background
dfx start --background

# Check replica status
dfx ping
```

#### 1.2 Deploy Canisters

```bash
# Deploy all canisters
dfx deploy

# Check canister status
dfx canister status proof_of_state
dfx canister status btc_signer_psbt
dfx canister status cross_chain_service
dfx canister status evm_rpc
```

#### 1.3 Test Proof-of-State Canister

```bash
# Submit a test receipt
dfx canister call proof_of_state submit_receipt '("test_data_123", "test_metadata")'

# Get receipt by ID (use returned receipt_id)
dfx canister call proof_of_state get_receipt '("receipt_id_here")'

# Check batch status
dfx canister call proof_of_state get_current_batch
```

### Phase 2: Bitcoin Integration Testing

#### 2.1 Test tECDSA Key Generation

```bash
# Get Bitcoin address from canister
dfx canister call btc_signer_psbt get_btc_address '("testnet")'

# Test key derivation
dfx canister call btc_signer_psbt get_public_key
```

#### 2.2 Test Transaction Creation

```bash
# Create unsigned transaction (mock)
dfx canister call btc_signer_psbt create_unsigned_transaction '(
  vec { 
    record { 
      txid = "mock_utxo_txid"; 
      vout = 0:nat32; 
      amount = 100000:nat64 
    } 
  },
  vec { 
    record { 
      address = "tb1qtest..."; 
      amount = 90000:nat64 
    } 
  }
)'
```

### Phase 3: Cross-Chain Service Testing

#### 3.1 Test DVN Message Submission

```bash
# Submit cross-chain message
dfx canister call cross_chain_service submit_dvn_message '(
  1:nat32,
  2:nat32,
  blob "test_payload",
  "test_sender"
)'

# Check pending messages
dfx canister call cross_chain_service get_pending_messages
```

#### 3.2 Test DVN Attestations

```bash
# Submit attestation (use message_id from previous step)
dfx canister call cross_chain_service submit_attestation '(
  "msg_id_here",
  "validator_1",
  blob "signature_data"
)'

# Check message attestations
dfx canister call cross_chain_service get_message_attestations '("msg_id_here")'
```

### Phase 4: EVM RPC Testing

#### 4.1 Initialize Chain Configurations

```bash
# Initialize supported chains
dfx canister call evm_rpc init_chain_configs

# Get supported chains
dfx canister call evm_rpc get_supported_chains
```

#### 4.2 Test Transaction Receipt Fetching

```bash
# Test with a real Sepolia transaction
dfx canister call evm_rpc get_transaction_receipt '(
  11155111:nat32,
  "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
)'

# Get latest block number
dfx canister call evm_rpc get_latest_block_number '(11155111:nat32)'
```

### Phase 5: Frontend Integration Testing

#### 5.1 Start Development Servers

```bash
# Terminal 1: Aigent Z Beta
cd apps/aigent-z && pnpm dev

# Terminal 2: 21 Sats Market
cd apps/21sats-market && pnpm dev

# Terminal 3: Ops Console
cd apps/ops-console && pnpm dev
```

#### 5.2 Test SDK Integration

Open the Ops Console at `http://localhost:3002` and:

1. **Test Anchor Status**: Verify mock data displays correctly
2. **Test Anchoring Submission**: 
   - Enter test data in the textarea
   - Click "Submit for Anchoring"
   - Verify test results show success/failure
3. **Test EVM Transaction Check**:
   - Click "Test EVM TX Check"
   - Verify mock EVM transaction data appears

#### 5.3 Test Real Canister Integration

Once canisters are deployed:

1. Update environment variables:
```bash
# In apps/ops-console/.env.local
PROOF_OF_STATE_CANISTER_ID=your_canister_id_here
CROSS_CHAIN_SERVICE_CANISTER_ID=your_canister_id_here
EVM_RPC_CANISTER_ID=your_canister_id_here
ICP_HOST=http://localhost:8000
```

2. Restart the ops console and test real canister calls

### Phase 6: End-to-End Integration Test

#### 6.1 Complete Anchor Flow

1. **Submit Data for Anchoring**:
   ```bash
   dfx canister call proof_of_state submit_receipt '("real_iqube_data", "metadata_json")'
   ```

2. **Trigger Bitcoin Anchoring**:
   ```bash
   dfx canister call proof_of_state anchor_batch_to_btc
   ```

3. **Verify Cross-Chain Message**:
   ```bash
   dfx canister call cross_chain_service get_ready_messages
   ```

4. **Check EVM Transaction Status**:
   ```bash
   dfx canister call evm_rpc get_transaction_receipt '(11155111:nat32, "tx_hash")'
   ```

## Expected Results

### Successful Test Indicators

- ✅ All canisters deploy without errors
- ✅ Proof-of-state canister accepts receipts and creates batches
- ✅ BTC signer generates valid testnet addresses
- ✅ Cross-chain service processes DVN messages
- ✅ EVM RPC fetches transaction data from testnets
- ✅ Frontend apps build and run without errors
- ✅ SDK functions return data (mock or real)

### Common Issues and Solutions

#### dfx Installation Issues
- **Problem**: `dfx: command not found`
- **Solution**: Ensure PATH is updated and terminal restarted

#### Canister Deployment Failures
- **Problem**: `Cannot find canister id`
- **Solution**: Run `dfx deploy` again or check dfx.json configuration

#### HTTP Request Failures
- **Problem**: EVM RPC calls fail
- **Solution**: Check internet connection and RPC endpoints

#### Frontend Build Errors
- **Problem**: TypeScript compilation errors
- **Solution**: Run `pnpm build` to check for type issues

## Performance Benchmarks

### Expected Response Times
- Canister calls: < 2 seconds
- Bitcoin address generation: < 5 seconds
- EVM transaction fetching: < 10 seconds
- Frontend page loads: < 3 seconds

### Resource Usage
- Local replica memory: ~500MB
- Canister cycles: Monitor with `dfx canister status`
- Network requests: Rate-limited by external APIs

## Security Considerations

### Testnet Only
- All Bitcoin operations use testnet
- EVM calls use testnets (Sepolia, Mumbai)
- No mainnet funds at risk

### Key Management
- tECDSA keys are managed by ICP
- Private keys never exposed
- All signatures happen on-canister

### API Rate Limits
- Blockstream API: 10 req/min
- Mempool.space: No strict limits
- Alchemy: 100 req/day (demo key)

## Next Steps

After successful testing:

1. **Mainnet Preparation**: Update configurations for mainnet deployment
2. **Performance Optimization**: Profile and optimize canister performance
3. **Security Audit**: Conduct thorough security review
4. **Documentation**: Create user guides and API documentation
5. **CI/CD Setup**: Implement automated testing pipeline

## Support

For issues or questions:
- Check dfx documentation: https://internetcomputer.org/docs/
- Review ICP samples: https://github.com/dfinity/examples
- Bitcoin integration guide: https://internetcomputer.org/bitcoin-integration
