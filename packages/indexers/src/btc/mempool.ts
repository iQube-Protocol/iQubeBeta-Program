/**
 * Mempool.space API adapter for Bitcoin testnet/mainnet
 * https://mempool.space/api/
 */

export interface MempoolTransaction {
  txid: string;
  version: number;
  locktime: number;
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
  size: number;
  weight: number;
  fee: number;
  status: {
    confirmed: boolean;
    block_height?: number;
    block_hash?: string;
    block_time?: number;
  };
}

export interface MempoolBlock {
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

export interface MempoolFees {
  fastestFee: number;
  halfHourFee: number;
  hourFee: number;
  economyFee: number;
  minimumFee: number;
}

export class MempoolAPI {
  private baseUrl: string;

  constructor(network: 'mainnet' | 'testnet' = 'testnet') {
    this.baseUrl = network === 'mainnet' 
      ? 'https://mempool.space/api'
      : 'https://mempool.space/testnet/api';
  }

  async getTransaction(txid: string): Promise<MempoolTransaction> {
    const response = await fetch(`${this.baseUrl}/tx/${txid}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch transaction: ${response.statusText}`);
    }
    return response.json();
  }

  async getTransactionStatus(txid: string): Promise<MempoolTransaction['status']> {
    const response = await fetch(`${this.baseUrl}/tx/${txid}/status`);
    if (!response.ok) {
      throw new Error(`Failed to fetch transaction status: ${response.statusText}`);
    }
    return response.json();
  }

  async getBlock(blockHash: string): Promise<MempoolBlock> {
    const response = await fetch(`${this.baseUrl}/block/${blockHash}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch block: ${response.statusText}`);
    }
    return response.json();
  }

  async getBlockAtHeight(height: number): Promise<MempoolBlock> {
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

  async getRecommendedFees(): Promise<MempoolFees> {
    const response = await fetch(`${this.baseUrl}/v1/fees/recommended`);
    if (!response.ok) {
      throw new Error(`Failed to fetch recommended fees: ${response.statusText}`);
    }
    return response.json();
  }

  async getMempoolInfo(): Promise<{
    loaded: boolean;
    size: number;
    bytes: number;
    usage: number;
    total_fee: number;
    maxmempool: number;
    mempoolminfee: number;
    minrelaytxfee: number;
  }> {
    const response = await fetch(`${this.baseUrl}/mempool`);
    if (!response.ok) {
      throw new Error(`Failed to fetch mempool info: ${response.statusText}`);
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

  async broadcastTransaction(txHex: string): Promise<string> {
    const response = await fetch(`${this.baseUrl}/tx`, {
      method: 'POST',
      headers: {
        'Content-Type': 'text/plain',
      },
      body: txHex,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to broadcast transaction: ${error}`);
    }

    return response.text(); // Returns txid
  }

  async getAddressUtxos(address: string): Promise<Array<{
    txid: string;
    vout: number;
    status: {
      confirmed: boolean;
      block_height?: number;
      block_hash?: string;
      block_time?: number;
    };
    value: number;
  }>> {
    const response = await fetch(`${this.baseUrl}/address/${address}/utxo`);
    if (!response.ok) {
      throw new Error(`Failed to fetch UTXOs: ${response.statusText}`);
    }
    return response.json();
  }

  async getAddressTransactions(address: string): Promise<MempoolTransaction[]> {
    const response = await fetch(`${this.baseUrl}/address/${address}/txs`);
    if (!response.ok) {
      throw new Error(`Failed to fetch address transactions: ${response.statusText}`);
    }
    return response.json();
  }
}
