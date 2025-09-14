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
export declare class BlockstreamAPI {
    private baseUrl;
    constructor(network?: 'mainnet' | 'testnet');
    getTransaction(txid: string): Promise<BTCTransaction>;
    getTransactionStatus(txid: string): Promise<BTCTransaction['status']>;
    getBlock(blockHash: string): Promise<BTCBlock>;
    getBlockAtHeight(height: number): Promise<BTCBlock>;
    getCurrentBlockHeight(): Promise<number>;
    getConfirmations(txid: string): Promise<number>;
    isTransactionAnchored(txid: string, requiredDepth?: number): Promise<boolean>;
    waitForConfirmations(txid: string, requiredDepth?: number, pollInterval?: number): Promise<number>;
}
