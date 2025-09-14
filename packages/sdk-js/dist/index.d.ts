export * from './types/iqube';
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
