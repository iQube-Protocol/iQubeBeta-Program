export declare function getSolAddress(): Promise<string>;
export declare function getSolBalance(address: string): Promise<bigint>;
export declare function requestAirdrop(address: string, lamports: bigint): Promise<string>;
export declare function transferSol(to: string, lamports: bigint): Promise<string>;
export declare function getSolTx(signature: string): Promise<{
    slot: bigint;
    signature: string;
} | null>;
export declare function getLatestBlockhash(): Promise<string>;
export declare function sendRawTxBase64(base64: string): Promise<string>;
