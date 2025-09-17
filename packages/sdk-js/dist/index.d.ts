import type { AnchorStatus, DualLockStatus } from './types/iqube';
export declare function getAnchorStatus(iQubeId: string): Promise<AnchorStatus>;
export declare function getDualLockStatus(iQubeId: string): Promise<DualLockStatus>;
export declare function submitForAnchoring(data: string, metadata: string): Promise<{
    receiptId: string;
    batchId?: string;
}>;
export declare function getEVMTransactionStatus(chainId: number, txHash: string): Promise<{
    confirmed: boolean;
    blockNumber?: number;
    gasUsed?: number;
}>;
export declare function getOrdinalPresence(iqubeId: string): Promise<boolean>;
export declare function initializeEVMRPC(): Promise<void>;
export declare function getSupportedChains(): Promise<any[]>;
export declare function mintIQuBeOnICP(dataHash: string): Promise<{
    receiptId: string;
}>;
export declare function anchorBatchesNow(): Promise<{
    batchRoot?: string;
    anchorTxId?: string | null;
}>;
export declare function getBitcoinAddress(derivationPath?: number[][]): Promise<{
    address: string;
    public_key?: number[];
}>;
export declare function submitCrossChainMessage(sourceChainId: number, destChainId: number, payload: string): Promise<string>;
export declare function attestDVNMessage(messageId: string, validators?: string[]): Promise<void>;
export declare function getCrossChainMessageStatus(messageId: string): Promise<{
    attestations: number;
    ready: boolean;
}>;
export declare function setBurnState(receiptId: string, messageId: string, burned: boolean): Promise<string | null>;
export declare function getBurnState(receiptId: string): Promise<any | null>;
export declare function getPendingDVNMessages(): Promise<any[]>;
export declare function getReadyDVNMessages(): Promise<any[]>;
export declare function createAnchorTransaction(dataRoot: string, utxos: any[], amountSats: bigint): Promise<any>;
export declare function signTransaction(unsignedTx: any, derivationPath?: number[][]): Promise<any>;
export declare function broadcastTransaction(rawTx: string): Promise<string>;
export type TestnetUtxo = {
    txid: string;
    vout: number;
    value: number;
    scriptpubkey: string;
};
export declare function fetchTestnetUtxos(address: string): Promise<TestnetUtxo[]>;
export declare function toCanisterUtxos(utxos: TestnetUtxo[]): any[];
export declare function createSignBroadcastAnchor(dataRoot: string, utxos: TestnetUtxo[], amountSats: bigint, derivationPath?: number[][]): Promise<string>;
