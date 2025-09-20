---
sidebar_position: 1
title: ICP Canister Integration
description: Connect with and integrate Internet Computer Protocol canisters
---

# ICP Canister Integration

The iQube Protocol leverages Internet Computer Protocol (ICP) canisters for decentralized computation, cross-chain operations, and secure data management. This guide covers integration patterns, canister interactions, and best practices.

## Deployed Canisters

### Production Canisters

#### proof_of_state Canister
- **Canister ID**: `ulvla-h7777-77774-qaacq-cai`
- **Purpose**: Receipt generation, batch processing, and Bitcoin anchoring
- **Language**: Rust
- **Status**: Deployed and operational

**Key Methods:**
```rust
// Issue a new receipt
issue_receipt(data: Vec<u8>) -> Result<String, String>

// Create a batch from multiple receipts
batch(receipt_ids: Vec<String>) -> Result<String, String>

// Anchor batch to Bitcoin blockchain
anchor(batch_id: String) -> Result<String, String>

// Retrieve receipt by ID
get_receipt(receipt_id: String) -> Option<Receipt>

// Get all batches
get_batches() -> Vec<Batch>

// Get pending receipt count
get_pending_count() -> u64
```

#### btc_signer_psbt Canister
- **Canister ID**: `uxrrr-q7777-77774-qaaaq-cai`
- **Purpose**: Bitcoin transaction signing with threshold ECDSA
- **Language**: Rust
- **Status**: Deployed and operational

**Key Methods:**
```rust
// Generate Bitcoin testnet address
get_btc_address() -> String

// Create anchor transaction
create_anchor_transaction(data: Vec<u8>) -> Result<String, String>

// Sign transaction with threshold ECDSA
sign_transaction(tx_hex: String) -> Result<String, String>

// Broadcast signed transaction
broadcast_transaction(signed_tx: String) -> Result<String, String>
```

#### cross_chain_service Canister
- **Canister ID**: `u6s2n-gx777-77774-qaaba-cai`
- **Purpose**: LayerZero DVN integration and cross-chain messaging
- **Language**: Rust
- **Status**: Deployed and operational

**Key Methods:**
```rust
// Submit DVN message for processing
submit_dvn_message(message: DVNMessage) -> Result<String, String>

// Submit attestation for message
submit_attestation(message_id: String, attestation: Vec<u8>) -> Result<(), String>

// Monitor EVM transaction
monitor_evm_transaction(tx_hash: String) -> TransactionStatus

// Verify LayerZero message
verify_layerzero_message(message: Vec<u8>) -> Result<bool, String>
```

#### evm_rpc Canister
- **Canister ID**: `uzt4z-lp777-77774-qaabq-cai`
- **Purpose**: EVM chain RPC interface and transaction monitoring
- **Language**: Rust
- **Status**: Deployed and operational

**Key Methods:**
```rust
// Get transaction receipt
get_transaction_receipt(chain_id: u64, tx_hash: String) -> Option<TransactionReceipt>

// Get block information
get_block_info(chain_id: u64, block_number: u64) -> Option<BlockInfo>

// Get latest block number
get_latest_block_number(chain_id: u64) -> Option<u64>

// Initialize chain configuration
init_chain_config(config: ChainConfig) -> Result<(), String>
```

## Integration Patterns

### HTTP API Integration

#### Canister HTTP Calls
The iQube Protocol uses HTTP API calls to interact with ICP canisters:

```typescript
// Example: Call proof_of_state canister
const callCanister = async (method: string, args: any[]) => {
  const response = await fetch(`https://ic0.app/api/v2/canister/${canisterId}/call`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
    },
    body: encodeCBOR({
      request_type: 'call',
      canister_id: canisterId,
      method_name: method,
      arg: encodeCandid(args),
    }),
  });
  
  return decodeCBOR(await response.arrayBuffer());
};
```

#### API Route Implementation
Next.js API routes provide a clean interface for canister interactions:

```typescript
// /app/api/ops/dvn/status/route.ts
export async function GET() {
  try {
    const canisterId = 'u6s2n-gx777-77774-qaaba-cai';
    const result = await callCanister('get_dvn_status', []);
    
    return Response.json({
      success: true,
      data: result,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    return Response.json({
      success: false,
      error: error.message,
      timestamp: new Date().toISOString(),
    }, { status: 500 });
  }
}
```

### React Hook Integration

#### Custom Hooks for Canister Data
React hooks provide clean, reusable interfaces for canister data:

```typescript
// hooks/useDVNStatus.ts
export const useDVNStatus = () => {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch('/api/ops/dvn/status');
        const result = await response.json();
        setData(result.data);
        setError(null);
      } catch (err) {
        setError(err.message);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 30000); // 30-second refresh

    return () => clearInterval(interval);
  }, []);

  return { data, loading, error };
};
```

### Error Handling and Resilience

#### Retry Logic
Implement robust retry mechanisms for canister calls:

```typescript
const retryCanisterCall = async (
  canisterCall: () => Promise<any>,
  maxRetries: number = 3,
  delay: number = 1000
) => {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await canisterCall();
    } catch (error) {
      if (attempt === maxRetries) {
        throw error;
      }
      await new Promise(resolve => setTimeout(resolve, delay * attempt));
    }
  }
};
```

#### Fallback Mechanisms
Provide fallback data when canisters are unavailable:

```typescript
const getCanisterDataWithFallback = async (canisterId: string) => {
  try {
    return await callCanister('get_status', []);
  } catch (error) {
    console.warn(`Canister ${canisterId} unavailable, using fallback`);
    return {
      status: 'unknown',
      last_updated: new Date().toISOString(),
      error: 'Canister unavailable',
    };
  }
};
```

## Development Setup

### Local Development

#### dfx Installation
Install the DFINITY Canister SDK:

```bash
# Install dfx
sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"

# Add to PATH
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify installation
dfx --version
```

#### Local Replica
Start a local Internet Computer replica for development:

```bash
# Start local replica
dfx start --background

# Deploy canisters locally
dfx deploy

# Check canister status
dfx canister status --all
```

#### Environment Configuration
Configure environment variables for canister integration:

```env
# .env.local
NEXT_PUBLIC_IC_HOST=http://localhost:4943
NEXT_PUBLIC_DFX_NETWORK=local

# Production
NEXT_PUBLIC_IC_HOST=https://ic0.app
NEXT_PUBLIC_DFX_NETWORK=ic
```

### Testing Integration

#### Unit Tests
Test canister integration with Jest:

```typescript
// __tests__/canister-integration.test.ts
describe('Canister Integration', () => {
  test('should call proof_of_state canister', async () => {
    const mockResponse = { receipt_id: 'test_receipt_123' };
    
    // Mock fetch
    global.fetch = jest.fn().mockResolvedValue({
      json: () => Promise.resolve(mockResponse),
    });

    const result = await callCanister('issue_receipt', [new Uint8Array([1, 2, 3])]);
    expect(result.receipt_id).toBe('test_receipt_123');
  });
});
```

#### Integration Tests
Test end-to-end canister workflows:

```typescript
// __tests__/e2e-canister.test.ts
describe('End-to-End Canister Tests', () => {
  test('should complete proof-of-state workflow', async () => {
    // Issue receipt
    const receipt = await issueReceipt(testData);
    expect(receipt.receipt_id).toBeDefined();

    // Create batch
    const batch = await createBatch([receipt.receipt_id]);
    expect(batch.batch_id).toBeDefined();

    // Anchor to Bitcoin
    const anchor = await anchorBatch(batch.batch_id);
    expect(anchor.tx_hash).toBeDefined();
  });
});
```

## Security Considerations

### Authentication and Authorization

#### Identity Management
Use Internet Identity or other authentication mechanisms:

```typescript
import { AuthClient } from '@dfinity/auth-client';

const authClient = await AuthClient.create();

// Login with Internet Identity
await authClient.login({
  identityProvider: 'https://identity.ic0.app',
  onSuccess: () => {
    console.log('Authenticated successfully');
  },
});
```

#### Capability-Based Security
Implement capability tokens for fine-grained access control:

```rust
// Rust canister code
#[update]
fn access_protected_data(capability_token: String) -> Result<Vec<u8>, String> {
    if !validate_capability_token(&capability_token) {
        return Err("Invalid capability token".to_string());
    }
    
    // Return protected data
    Ok(get_protected_data())
}
```

### Data Validation

#### Input Sanitization
Always validate inputs before processing:

```rust
#[update]
fn process_data(data: Vec<u8>) -> Result<String, String> {
    if data.is_empty() {
        return Err("Data cannot be empty".to_string());
    }
    
    if data.len() > MAX_DATA_SIZE {
        return Err("Data too large".to_string());
    }
    
    // Process validated data
    Ok(process_validated_data(data))
}
```

#### Output Encoding
Properly encode outputs to prevent injection attacks:

```typescript
const sanitizeCanisterResponse = (response: any) => {
  if (typeof response === 'string') {
    return response.replace(/[<>]/g, '');
  }
  return response;
};
```

## Performance Optimization

### Caching Strategies

#### Response Caching
Cache canister responses to reduce calls:

```typescript
const canisterCache = new Map<string, { data: any; timestamp: number }>();

const getCachedCanisterData = async (canisterId: string, method: string) => {
  const cacheKey = `${canisterId}:${method}`;
  const cached = canisterCache.get(cacheKey);
  
  if (cached && Date.now() - cached.timestamp < 30000) { // 30-second cache
    return cached.data;
  }
  
  const data = await callCanister(method, []);
  canisterCache.set(cacheKey, { data, timestamp: Date.now() });
  return data;
};
```

#### Batch Operations
Combine multiple operations into single canister calls:

```rust
#[update]
fn batch_operations(operations: Vec<Operation>) -> Vec<Result<String, String>> {
    operations.into_iter()
        .map(|op| process_operation(op))
        .collect()
}
```

### Connection Pooling

#### HTTP Connection Management
Reuse HTTP connections for better performance:

```typescript
import { Agent, HttpAgent } from '@dfinity/agent';

const agent = new HttpAgent({
  host: process.env.NEXT_PUBLIC_IC_HOST,
});

// Reuse agent across calls
const callCanisterWithAgent = async (canisterId: string, method: string, args: any[]) => {
  const actor = Actor.createActor(idlFactory, {
    agent,
    canisterId,
  });
  
  return await actor[method](...args);
};
```

## Monitoring and Debugging

### Logging

#### Structured Logging
Implement comprehensive logging for canister interactions:

```typescript
const logger = {
  info: (message: string, data?: any) => {
    console.log(JSON.stringify({
      level: 'info',
      message,
      data,
      timestamp: new Date().toISOString(),
    }));
  },
  error: (message: string, error?: any) => {
    console.error(JSON.stringify({
      level: 'error',
      message,
      error: error?.message || error,
      timestamp: new Date().toISOString(),
    }));
  },
};

// Usage
logger.info('Calling canister', { canisterId, method });
```

#### Canister Metrics
Monitor canister performance and resource usage:

```rust
use ic_cdk::api::performance_counter;

#[query]
fn get_canister_metrics() -> CanisterMetrics {
    CanisterMetrics {
        cycles_balance: ic_cdk::api::canister_balance(),
        memory_usage: ic_cdk::api::stable::stable_size(),
        instruction_count: performance_counter(0),
        heap_memory: ic_cdk::api::performance_counter(1),
    }
}
```

### Debugging Tools

#### Canister Inspection
Use dfx commands to inspect canister state:

```bash
# Get canister status
dfx canister status proof_of_state

# Call canister methods
dfx canister call proof_of_state get_pending_count

# Check canister logs
dfx canister logs proof_of_state
```

#### Network Analysis
Monitor network traffic to identify issues:

```typescript
const monitorCanisterCalls = (canisterId: string) => {
  const originalFetch = window.fetch;
  
  window.fetch = async (url, options) => {
    if (url.includes(canisterId)) {
      console.log(`Canister call: ${url}`, options);
      const start = Date.now();
      
      try {
        const response = await originalFetch(url, options);
        console.log(`Canister response: ${Date.now() - start}ms`, response.status);
        return response;
      } catch (error) {
        console.error(`Canister error: ${Date.now() - start}ms`, error);
        throw error;
      }
    }
    
    return originalFetch(url, options);
  };
};
```

## Best Practices

### Code Organization
- **Separate Concerns**: Keep canister logic separate from UI logic
- **Type Safety**: Use TypeScript interfaces for all canister interactions
- **Error Boundaries**: Implement React error boundaries for canister failures
- **Testing**: Write comprehensive tests for all canister integrations

### Performance
- **Minimize Calls**: Batch operations when possible
- **Cache Responses**: Cache frequently accessed data
- **Async Operations**: Use async/await for non-blocking operations
- **Resource Management**: Monitor cycles and memory usage

### Security
- **Validate Inputs**: Always validate data before sending to canisters
- **Handle Errors**: Implement proper error handling and user feedback
- **Authentication**: Use proper identity management
- **Capability Tokens**: Implement fine-grained access control

---

*ICP canister integration provides the decentralized computational foundation for the iQube Protocol, enabling secure, scalable, and verifiable operations across the Internet Computer network.*
