import { BlockstreamAPI } from './btc/blockstream';
import { MempoolAPI } from './btc/mempool';
export { BlockstreamAPI } from './btc/blockstream';
export { MempoolAPI } from './btc/mempool';
/**
 * Unified BTC indexer that can switch between providers
 */
export class BTCIndexer {
    constructor(network = 'testnet', primaryProvider = 'blockstream') {
        this.blockstream = new BlockstreamAPI(network);
        this.mempool = new MempoolAPI(network);
        this.primaryProvider = primaryProvider;
    }
    async getTransaction(txid) {
        try {
            return this.primaryProvider === 'blockstream'
                ? await this.blockstream.getTransaction(txid)
                : await this.mempool.getTransaction(txid);
        }
        catch (error) {
            // Fallback to secondary provider
            return this.primaryProvider === 'blockstream'
                ? await this.mempool.getTransaction(txid)
                : await this.blockstream.getTransaction(txid);
        }
    }
    async getConfirmations(txid) {
        try {
            return this.primaryProvider === 'blockstream'
                ? await this.blockstream.getConfirmations(txid)
                : await this.mempool.getConfirmations(txid);
        }
        catch (error) {
            // Fallback to secondary provider
            return this.primaryProvider === 'blockstream'
                ? await this.mempool.getConfirmations(txid)
                : await this.blockstream.getConfirmations(txid);
        }
    }
    async isTransactionAnchored(txid, requiredDepth = 6) {
        const confirmations = await this.getConfirmations(txid);
        return confirmations >= requiredDepth;
    }
    async getCurrentBlockHeight() {
        try {
            return this.primaryProvider === 'blockstream'
                ? await this.blockstream.getCurrentBlockHeight()
                : await this.mempool.getCurrentBlockHeight();
        }
        catch (error) {
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
    async broadcastTransaction(txHex) {
        return this.mempool.broadcastTransaction(txHex);
    }
}
