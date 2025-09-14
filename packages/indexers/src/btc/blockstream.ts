/**
 * Blockstream API adapter for Bitcoin testnet/mainnet
 * https://blockstream.info/api/
 */

export interface BTCTransaction {
  txid: string;
  version: number;
  locktime: number;
  size: number;
  weight: number;
  fee: number;
  vin: Array<{
    txid: string;
    vout: number;
    prevout: {
      scriptpubkey: string;
      scriptpubkey_asm: string;
      scriptpubkey_type: string;
      scriptpubkey_address?: string;
      value: number;
    };
    scriptsig: string;
    scriptsig_asm: string;
    sequence: number;
  }>;
  vout: Array<{
    scriptpubkey: string;
    scriptpubkey_asm: string;
    scriptpubkey_type: string;
    scriptpubkey_address?: string;
    value: number;
  }>;
  status: {
    confirmed: boolean;
    block_height?: number;
    block_hash?: string;
    block_time?: number;
  };
}

export interface BTCBlock {
  id: string;
  height: number;
  version: number;
  timestamp: number;
  tx_count: number;
  size: number;
  weight: number;
  merkle_root: string;
  previousblockhash?: string;
  mediantime: number;
  nonce: number;
  bits: number;
  difficulty: number;
}

export class BlockstreamAPI {
  private baseUrl: string;

  constructor(network: 'mainnet' | 'testnet' = 'testnet') {
    this.baseUrl = network === 'mainnet' 
      ? 'https://blockstream.info/api'
      : 'https://blockstream.info/testnet/api';
  }

  async getTransaction(txid: string): Promise<BTCTransaction> {
    const response = await fetch(`${this.baseUrl}/tx/${txid}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch transaction: ${response.statusText}`);
    }
    return response.json();
  }

  async getTransactionStatus(txid: string): Promise<BTCTransaction['status']> {
    const response = await fetch(`${this.baseUrl}/tx/${txid}/status`);
    if (!response.ok) {
      throw new Error(`Failed to fetch transaction status: ${response.statusText}`);
    }
    return response.json();
  }

  async getBlock(blockHash: string): Promise<BTCBlock> {
    const response = await fetch(`${this.baseUrl}/block/${blockHash}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch block: ${response.statusText}`);
    }
    return response.json();
  }

  async getBlockAtHeight(height: number): Promise<BTCBlock> {
    const response = await fetch(`${this.baseUrl}/block-height/${height}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch block at height: ${response.statusText}`);
    }
    const blockHash = await response.text();
    return this.getBlock(blockHash);
  }

  async getCurrentBlockHeight(): Promise<number> {
    const response = await fetch(`${this.baseUrl}/blocks/tip/height`);
    if (!response.ok) {
      throw new Error(`Failed to fetch current block height: ${response.statusText}`);
    }
    return response.json();
  }

  async getConfirmations(txid: string): Promise<number> {
    const [tx, currentHeight] = await Promise.all([
      this.getTransactionStatus(txid),
      this.getCurrentBlockHeight()
    ]);

    if (!tx.confirmed || !tx.block_height) {
      return 0;
    }

    return currentHeight - tx.block_height + 1;
  }

  async isTransactionAnchored(txid: string, requiredDepth: number = 6): Promise<boolean> {
    const confirmations = await this.getConfirmations(txid);
    return confirmations >= requiredDepth;
  }

  async waitForConfirmations(
    txid: string, 
    requiredDepth: number = 6, 
    pollInterval: number = 30000
  ): Promise<number> {
    return new Promise((resolve, reject) => {
      const checkConfirmations = async () => {
        try {
          const confirmations = await this.getConfirmations(txid);
          
          if (confirmations >= requiredDepth) {
            resolve(confirmations);
          } else {
            setTimeout(checkConfirmations, pollInterval);
          }
        } catch (error) {
          reject(error);
        }
      };

      checkConfirmations();
    });
  }
}
