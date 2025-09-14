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
export declare class MempoolAPI {
    private baseUrl;
    constructor(network?: 'mainnet' | 'testnet');
    getTransaction(txid: string): Promise<MempoolTransaction>;
    getTransactionStatus(txid: string): Promise<MempoolTransaction['status']>;
    getBlock(blockHash: string): Promise<MempoolBlock>;
    getBlockAtHeight(height: number): Promise<MempoolBlock>;
    getCurrentBlockHeight(): Promise<number>;
    getRecommendedFees(): Promise<MempoolFees>;
    getMempoolInfo(): Promise<{
        loaded: boolean;
        size: number;
        bytes: number;
        usage: number;
        total_fee: number;
        maxmempool: number;
        mempoolminfee: number;
        minrelaytxfee: number;
    }>;
    getConfirmations(txid: string): Promise<number>;
    isTransactionAnchored(txid: string, requiredDepth?: number): Promise<boolean>;
    broadcastTransaction(txHex: string): Promise<string>;
    getAddressUtxos(address: string): Promise<Array<{
        txid: string;
        vout: number;
        status: {
            confirmed: boolean;
            block_height?: number;
            block_hash?: string;
            block_time?: number;
        };
        value: number;
    }>>;
    getAddressTransactions(address: string): Promise<MempoolTransaction[]>;
}
