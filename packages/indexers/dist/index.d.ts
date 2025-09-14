export { BlockstreamAPI } from './btc/blockstream';
export { MempoolAPI } from './btc/mempool';
export type { BTCTransaction, BTCBlock } from './btc/blockstream';
export type { MempoolTransaction, MempoolBlock, MempoolFees } from './btc/mempool';
/**
 * Unified BTC indexer that can switch between providers
 */
export declare class BTCIndexer {
    private blockstream;
    private mempool;
    private primaryProvider;
    constructor(network?: 'mainnet' | 'testnet', primaryProvider?: 'blockstream' | 'mempool');
    getTransaction(txid: string): Promise<import("./btc/blockstream").BTCTransaction>;
    getConfirmations(txid: string): Promise<number>;
    isTransactionAnchored(txid: string, requiredDepth?: number): Promise<boolean>;
    getCurrentBlockHeight(): Promise<number>;
    getRecommendedFees(): Promise<import("./btc/mempool").MempoolFees>;
    broadcastTransaction(txHex: string): Promise<string>;
}
