/**
 * Blockstream API adapter for Bitcoin testnet/mainnet
 * https://blockstream.info/api/
 */
export class BlockstreamAPI {
    constructor(network = 'testnet') {
        this.baseUrl = network === 'mainnet'
            ? 'https://blockstream.info/api'
            : 'https://blockstream.info/testnet/api';
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
    async waitForConfirmations(txid, requiredDepth = 6, pollInterval = 30000) {
        return new Promise((resolve, reject) => {
            const checkConfirmations = async () => {
                try {
                    const confirmations = await this.getConfirmations(txid);
                    if (confirmations >= requiredDepth) {
                        resolve(confirmations);
                    }
                    else {
                        setTimeout(checkConfirmations, pollInterval);
                    }
                }
                catch (error) {
                    reject(error);
                }
            };
            checkConfirmations();
        });
    }
}
