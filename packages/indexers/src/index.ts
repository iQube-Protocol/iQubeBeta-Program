import { BlockstreamAPI } from './btc/blockstream';
import { MempoolAPI } from './btc/mempool';

export { BlockstreamAPI } from './btc/blockstream';
export { MempoolAPI } from './btc/mempool';
export type { 
  BTCTransaction, 
  BTCBlock 
} from './btc/blockstream';
export type { 
  MempoolTransaction, 
  MempoolBlock, 
  MempoolFees 
} from './btc/mempool';

/**
 * Unified BTC indexer that can switch between providers
 */
export class BTCIndexer {
  private blockstream: BlockstreamAPI;
  private mempool: MempoolAPI;
  private primaryProvider: 'blockstream' | 'mempool';

  constructor(
    network: 'mainnet' | 'testnet' = 'testnet',
    primaryProvider: 'blockstream' | 'mempool' = 'blockstream'
  ) {
    this.blockstream = new BlockstreamAPI(network);
    this.mempool = new MempoolAPI(network);
    this.primaryProvider = primaryProvider;
  }

  async getTransaction(txid: string) {
    try {
      return this.primaryProvider === 'blockstream' 
        ? await this.blockstream.getTransaction(txid)
        : await this.mempool.getTransaction(txid);
    } catch (error) {
      // Fallback to secondary provider
      return this.primaryProvider === 'blockstream'
        ? await this.mempool.getTransaction(txid)
        : await this.blockstream.getTransaction(txid);
    }
  }

  async getConfirmations(txid: string): Promise<number> {
    try {
      return this.primaryProvider === 'blockstream'
        ? await this.blockstream.getConfirmations(txid)
        : await this.mempool.getConfirmations(txid);
    } catch (error) {
      // Fallback to secondary provider
      return this.primaryProvider === 'blockstream'
        ? await this.mempool.getConfirmations(txid)
        : await this.blockstream.getConfirmations(txid);
    }
  }

  async isTransactionAnchored(txid: string, requiredDepth: number = 6): Promise<boolean> {
    const confirmations = await this.getConfirmations(txid);
    return confirmations >= requiredDepth;
  }

  async getCurrentBlockHeight(): Promise<number> {
    try {
      return this.primaryProvider === 'blockstream'
        ? await this.blockstream.getCurrentBlockHeight()
        : await this.mempool.getCurrentBlockHeight();
    } catch (error) {
      // Fallback to secondary provider
      return this.primaryProvider === 'blockstream'
        ? await this.mempool.getCurrentBlockHeight()
        : await this.blockstream.getCurrentBlockHeight();
    }
  }

  // Mempool-specific features
  async getRecommendedFees() {
    return this.mempool.getRecommendedFees();
  }

  async broadcastTransaction(txHex: string) {
    return this.mempool.broadcastTransaction(txHex);
  }
}
