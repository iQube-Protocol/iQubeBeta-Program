export * from './types/iqube';
import type { AnchorStatus, DualLockStatus } from './types/iqube';

// ICP Canister interfaces
interface ICPCanisterConfig {
  canisterId: string;
  host: string;
}

const CANISTER_CONFIG = {
  proof_of_state: {
    canisterId: process.env.PROOF_OF_STATE_CANISTER_ID || 'rdmx6-jaaaa-aaaaa-aaadq-cai',
    host: process.env.ICP_HOST || 'http://localhost:8000',
  },
  cross_chain_service: {
    canisterId: process.env.CROSS_CHAIN_SERVICE_CANISTER_ID || 'rrkah-fqaaa-aaaaa-aaaaq-cai',
    host: process.env.ICP_HOST || 'http://localhost:8000',
  },
  evm_rpc: {
    canisterId: process.env.EVM_RPC_CANISTER_ID || 'ryjl3-tyaaa-aaaaa-aaaba-cai',
    host: process.env.ICP_HOST || 'http://localhost:8000',
  },
};

async function callICPCanister(
  canister: keyof typeof CANISTER_CONFIG,
  method: string,
  args: any[] = []
): Promise<any> {
  const config = CANISTER_CONFIG[canister];
  
  try {
    // For now, use HTTP requests to local dfx replica
    const response = await fetch(`${config.host}/api/v2/canister/${config.canisterId}/call`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/cbor',
      },
      body: JSON.stringify({
        method_name: method,
        arg: args,
      }),
    });
    
    if (!response.ok) {
      throw new Error(`ICP canister call failed: ${response.statusText}`);
    }
    
    return await response.json();
  } catch (error) {
    console.warn(`ICP canister call failed, falling back to mock data:`, error);
    return null;
  }
}

export async function getAnchorStatus(iQubeId: string): Promise<AnchorStatus> {
  try {
    // Try to get real anchor status from proof_of_state canister
    const receipt = await callICPCanister('proof_of_state', 'get_receipt', [iQubeId]);
    
    if (receipt && receipt.btc_tx_hash) {
      return {
        btcTxHash: receipt.btc_tx_hash,
        confirmations: receipt.confirmations || 0,
        blockHeight: receipt.block_height || 0,
        isConfirmed: receipt.confirmations >= 6,
      };
    }
  } catch (error) {
    console.warn('Failed to get real anchor status:', error);
  }
  
  // Fallback to mock data
  return {
    btcTxHash: 'mock_btc_tx_hash_' + iQubeId.slice(-8),
    confirmations: 6,
    blockHeight: 850000,
    isConfirmed: true,
  };
}

export async function getDualLockStatus(iQubeId: string): Promise<DualLockStatus> {
  try {
    // Try to get real dual lock status from cross_chain_service canister
    const transaction = await callICPCanister('cross_chain_service', 'get_transaction', [iQubeId]);
    
    if (transaction) {
      return {
        evmTxHash: transaction.tx_hash || '',
        icpReceiptId: transaction.id || '',
        isLocked: transaction.status === 'confirmed',
        unlockHeight: transaction.block_height + 100 || 851000,
      };
    }
  } catch (error) {
    console.warn('Failed to get real dual lock status:', error);
  }
  
  // Fallback to mock data
  return {
    evmTxHash: 'mock_evm_tx_hash_' + iQubeId.slice(-8),
    icpReceiptId: 'mock_icp_receipt_' + iQubeId.slice(-8),
    isLocked: true,
    unlockHeight: 851000,
  };
}

// New function to submit data for anchoring
export async function submitForAnchoring(
  data: string,
  metadata: string
): Promise<{ receiptId: string; batchId?: string }> {
  try {
    const result = await callICPCanister('proof_of_state', 'submit_receipt', [data, metadata]);
    
    if (result && result.receipt_id) {
      return {
        receiptId: result.receipt_id,
        batchId: result.batch_id,
      };
    }
  } catch (error) {
    console.warn('Failed to submit for anchoring:', error);
  }
  
  // Fallback to mock
  return {
    receiptId: 'mock_receipt_' + Date.now(),
    batchId: 'mock_batch_' + Date.now(),
  };
}

// New function to check EVM transaction status
export async function getEVMTransactionStatus(
  chainId: number,
  txHash: string
): Promise<{ confirmed: boolean; blockNumber?: number; gasUsed?: number }> {
  try {
    const result = await callICPCanister('evm_rpc', 'get_transaction_receipt', [chainId, txHash]);
    
    if (result && result.Ok) {
      const receipt = result.Ok;
      return {
        confirmed: receipt.status,
        blockNumber: receipt.block_number,
        gasUsed: receipt.gas_used,
      };
    }
  } catch (error) {
    console.warn('Failed to get EVM transaction status:', error);
  }
  
  // Fallback to mock
  return {
    confirmed: true,
    blockNumber: 18500000,
    gasUsed: 21000,
  };
}

export async function getOrdinalPresence(iqubeId: string): Promise<boolean> {
  // TODO: query ordinal adapter
  return false;
}
