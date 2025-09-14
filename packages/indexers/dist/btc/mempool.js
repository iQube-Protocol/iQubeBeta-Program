/**
 * Mempool.space API adapter for Bitcoin testnet/mainnet
 * https://mempool.space/api/
 */
export class MempoolAPI {
    constructor(network = 'testnet') {
        this.baseUrl = network === 'mainnet'
            ? 'https://mempool.space/api'
            : 'https://mempool.space/testnet/api';
    }
    async getTransaction(txid) {
        const response = await fetch(`${this.baseUrl}/tx/${txid}`);
        if (!response.ok) {
            throw new Error(`Failed to fetch transaction: ${response.statusText}`);
        }
        return response.json();
    }
    async getTransactionStatus(txid) {
        const response = await fetch(`${this.baseUrl}/tx/${txid}/status`);
        if (!response.ok) {
            throw new Error(`Failed to fetch transaction status: ${response.statusText}`);
        }
        return response.json();
    }
    async getBlock(blockHash) {
        const response = await fetch(`${this.baseUrl}/block/${blockHash}`);
        if (!response.ok) {
            throw new Error(`Failed to fetch block: ${response.statusText}`);
        }
        return response.json();
    }
    async getBlockAtHeight(height) {
        const response = await fetch(`${this.baseUrl}/block-height/${height}`);
        if (!response.ok) {
            throw new Error(`Failed to fetch block at height: ${response.statusText}`);
        }
        const blockHash = await response.text();
        return this.getBlock(blockHash);
    }
    async getCurrentBlockHeight() {
        const response = await fetch(`${this.baseUrl}/blocks/tip/height`);
        if (!response.ok) {
            throw new Error(`Failed to fetch current block height: ${response.statusText}`);
        }
        return response.json();
    }
    async getRecommendedFees() {
        const response = await fetch(`${this.baseUrl}/v1/fees/recommended`);
        if (!response.ok) {
            throw new Error(`Failed to fetch recommended fees: ${response.statusText}`);
        }
        return response.json();
    }
    async getMempoolInfo() {
        const response = await fetch(`${this.baseUrl}/mempool`);
        if (!response.ok) {
            throw new Error(`Failed to fetch mempool info: ${response.statusText}`);
        }
        return response.json();
    }
    async getConfirmations(txid) {
        const [tx, currentHeight] = await Promise.all([
            this.getTransactionStatus(txid),
            this.getCurrentBlockHeight()
        ]);
        if (!tx.confirmed || !tx.block_height) {
            return 0;
        }
        return currentHeight - tx.block_height + 1;
    }
    async isTransactionAnchored(txid, requiredDepth = 6) {
        const confirmations = await this.getConfirmations(txid);
        return confirmations >= requiredDepth;
    }
    async broadcastTransaction(txHex) {
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
    async getAddressUtxos(address) {
        const response = await fetch(`${this.baseUrl}/address/${address}/utxo`);
        if (!response.ok) {
            throw new Error(`Failed to fetch UTXOs: ${response.statusText}`);
        }
        return response.json();
    }
    async getAddressTransactions(address) {
        const response = await fetch(`${this.baseUrl}/address/${address}/txs`);
        if (!response.ok) {
            throw new Error(`Failed to fetch address transactions: ${response.statusText}`);
        }
        return response.json();
    }
}
