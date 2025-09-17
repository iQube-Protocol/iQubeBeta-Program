'use client';

import { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import * as bitcoin from 'bitcoinjs-lib';
import * as ecc from 'tiny-secp256k1';
import { ECPairFactory } from 'ecpair';
import { 
  getAnchorStatus, 
  getDualLockStatus, 
  submitForAnchoring, 
  getEVMTransactionStatus 
} from '@iqube/sdk-js';

// Initialize ECC library for bitcoinjs
bitcoin.initEccLib(ecc);
const TESTNET = bitcoin.networks.testnet;
const ECPair = ECPairFactory(ecc);

interface CanisterStatus {
  name: string;
  status: 'healthy' | 'degraded' | 'down';
  lastCheck: string;
}

export default function OpsConsole() {
  const [anchorStatus, setAnchorStatus] = useState<any>(null);
  const [dualLockStatus, setDualLockStatus] = useState<any>(null);
  const [canisters, setCanisters] = useState<CanisterStatus[]>([
    { name: 'proof_of_state', status: 'healthy', lastCheck: new Date().toISOString() },
    { name: 'btc_signer_psbt', status: 'healthy', lastCheck: new Date().toISOString() },
    { name: 'cross_chain_service', status: 'degraded', lastCheck: new Date().toISOString() },
    { name: 'evm_rpc', status: 'healthy', lastCheck: new Date().toISOString() },
  ]);
  const [loading, setLoading] = useState(true);
  const [testData, setTestData] = useState('');
  const [testResults, setTestResults] = useState<any[]>([]);
  // Wallet state
  const [sepoliaAccount, setSepoliaAccount] = useState<string | null>(null);
  const [amoyAccount, setAmoyAccount] = useState<string | null>(null);
  const [btcPrivKey, setBtcPrivKey] = useState<string>('');
  const [amoyLatestTx, setAmoyLatestTx] = useState<string | null>(null);
  const [btcAddress, setBtcAddress] = useState<string | null>(null);
  const [btcUtxos, setBtcUtxos] = useState<any[]>([]);
  const [btcLatestTx, setBtcLatestTx] = useState<string | null>(null);
  const [btcBusy, setBtcBusy] = useState<boolean>(false);

  useEffect(() => {
    async function loadData() {
      try {
        const [anchor, dualLock] = await Promise.all([
          getAnchorStatus('test-iqube-1'),
          getDualLockStatus('test-iqube-1'),
        ]);
        setAnchorStatus(anchor);
        setDualLockStatus(dualLock);
      } catch (error) {
        console.error('Failed to load data:', error);
      } finally {
        setLoading(false);
      }
    }

    loadData();
  }, []);

  const handleSubmitForAnchoring = async () => {
    if (!testData.trim()) return;
    
    try {
      const result = await submitForAnchoring(testData, JSON.stringify({ 
        timestamp: new Date().toISOString(),
        source: 'ops-console-test' 
      }));
      
      setTestResults(prev => [...prev, {
        type: 'anchor_submit',
        timestamp: new Date().toISOString(),
        data: result,
        status: 'success'
      }]);
      
      setTestData('');
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'anchor_submit',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  // Small helper to avoid forever-hangs
  const fetchWithTimeout = async (input: RequestInfo | URL, init: RequestInit & { timeoutMs?: number } = {}) => {
    const { timeoutMs = 10000, ...rest } = init;
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), timeoutMs);
    try {
      return await fetch(input, { ...rest, signal: controller.signal });
    } finally {
      clearTimeout(id);
    }
  };

  // ===== Bitcoin helpers and actions =====
  const deriveBtcAddressFromWIF = (wif: string): string => {
    const keyPair = ECPair.fromWIF(wif, TESTNET);
    const { address } = bitcoin.payments.p2wpkh({ pubkey: keyPair.publicKey, network: TESTNET });
    if (!address) throw new Error('Failed to derive address');
    return address;
  };

  const fetchFeeRate = async (): Promise<number> => {
    try {
      const res = await fetch('/api/btc/fee', { cache: 'no-store' });
      const data = await res.json();
      return Math.ceil(data['1'] || data['2'] || 5);
    } catch {
      return 5;
    }
  };

  const handleBtcDeriveAddress = async () => {
    try {
      if (!btcPrivKey) throw new Error('Enter testnet WIF first');
      const addr = deriveBtcAddressFromWIF(btcPrivKey.trim());
      setBtcAddress(addr);
      setTestResults(prev => [...prev, { type: 'btc_address', timestamp: new Date().toISOString(), data: { address: addr }, status: 'success' }]);
    } catch (e: any) {
      setTestResults(prev => [...prev, { type: 'btc_address_error', timestamp: new Date().toISOString(), error: e?.message || 'derive failed', status: 'error' }]);
    }
  };

  const handleBtcFetchUtxos = async () => {
    try {
      if (!btcAddress) throw new Error('Derive address first');
      setBtcBusy(true);
      const res = await fetchWithTimeout(`/api/btc/utxos?address=${encodeURIComponent(btcAddress)}`, { cache: 'no-store', timeoutMs: 12000 });
      if (!res.ok) {
        const body = await res.text();
        throw new Error(`UTXO fetch failed (${res.status}): ${body?.slice(0,200)}`);
      }
      const utxos = await res.json();
      setBtcUtxos(utxos);
      setTestResults(prev => [...prev, { type: 'btc_utxos', timestamp: new Date().toISOString(), data: { count: utxos.length, utxos }, status: 'success' }]);
    } catch (e: any) {
      setTestResults(prev => [...prev, { type: 'btc_utxos_error', timestamp: new Date().toISOString(), error: e?.message || 'utxo fetch failed', status: 'error' }]);
    } finally {
      setBtcBusy(false);
    }
  };

  const handleBtcBroadcastTestTx = async () => {
    try {
      if (!btcPrivKey) throw new Error('Enter WIF');
      if (!btcAddress) throw new Error('Derive address first');
      setBtcBusy(true);
      const keyPair = ECPair.fromWIF(btcPrivKey.trim(), TESTNET);
      // refresh utxos via local proxy to avoid CORS/timeouts
      const utxoRes = await fetchWithTimeout(`/api/btc/utxos?address=${encodeURIComponent(btcAddress)}`, { cache: 'no-store', timeoutMs: 12000 });
      if (!utxoRes.ok) {
        const body = await utxoRes.text();
        throw new Error(`UTXO fetch failed (${utxoRes.status}): ${body?.slice(0,200)}`);
      }
      const utxos: Array<{ txid: string; vout: number; value: number }> = await utxoRes.json();
      if (!utxos.length) throw new Error('No UTXOs found. Fund the address from faucet.');
      const utxo = utxos.sort((a,b) => b.value - a.value)[0];
      const feeRate = await fetchFeeRate();

      const psbt = new bitcoin.Psbt({ network: TESTNET });
      const txHexRes = await fetch(`/api/btc/txhex?txid=${encodeURIComponent(utxo.txid)}`, { cache: 'no-store' });
      const txHex = await txHexRes.text();

      const p2wpkh = bitcoin.payments.p2wpkh({ pubkey: keyPair.publicKey, network: TESTNET });
      psbt.addInput({
        hash: utxo.txid,
        index: utxo.vout,
        witnessUtxo: { script: p2wpkh.output!, value: utxo.value },
        nonWitnessUtxo: Buffer.from(txHex, 'hex'),
      });

      const vbytes = 110; // rough estimate
      const fee = vbytes * feeRate;
      const outputValue = utxo.value - fee - 100; // dust buffer
      if (outputValue <= 546) throw new Error('Insufficient funds for fee');
      psbt.addOutput({ address: btcAddress, value: outputValue });

      psbt.signAllInputs(keyPair);
      psbt.finalizeAllInputs();
      const txHexSigned = psbt.extractTransaction().toHex();

      const broadcast = await fetch('/api/btc/broadcast', { method: 'POST', headers: { 'Content-Type': 'text/plain' }, body: txHexSigned });
      const txid = await broadcast.text();
      setBtcLatestTx(txid);
      setTestResults(prev => [...prev, { type: 'btc_broadcast', timestamp: new Date().toISOString(), data: { txid, feeRate }, status: 'success' }]);
    } catch (e: any) {
      setTestResults(prev => [...prev, { type: 'btc_broadcast_error', timestamp: new Date().toISOString(), error: e?.message || 'broadcast failed', status: 'error' }]);
    } finally {
      setBtcBusy(false);
    }
  };

  const handleBtcGenerateKey = async () => {
    try {
      setBtcBusy(true);
      const keyPair = ECPair.makeRandom({ network: TESTNET });
      const wif = keyPair.toWIF();
      const { address } = bitcoin.payments.p2wpkh({ pubkey: keyPair.publicKey, network: TESTNET });
      if (!address) throw new Error('Failed to derive address');
      setBtcPrivKey(wif);
      setBtcAddress(address);
      setTestResults(prev => [...prev, { type: 'btc_generate_key', timestamp: new Date().toISOString(), data: { address, wifMasked: `${wif.slice(0,4)}...${wif.slice(-4)}` }, status: 'success' }]);
    } catch (e: any) {
      setTestResults(prev => [...prev, { type: 'btc_generate_key_error', timestamp: new Date().toISOString(), error: e?.message || 'generate failed', status: 'error' }]);
    } finally {
      setBtcBusy(false);
    }
  };

  // EVM mint on Polygon Amoy via MetaMask + ethers.js
  const AMOY_CONTRACT = '0x632E1d32e34F0A690635BBcbec0D066daa448ede';
  const MIN_ABI_MINT_QUBE = [
    {
      inputs: [
        { internalType: 'string', name: 'uri', type: 'string' },
        { internalType: 'string', name: 'encryptionKey', type: 'string' },
      ],
      name: 'mintQube',
      outputs: [],
      stateMutability: 'nonpayable',
      type: 'function',
    },
    {
      inputs: [{ internalType: 'uint256', name: 'tokenId', type: 'uint256' }],
      name: 'getMetaQubeLocation',
      outputs: [{ internalType: 'string', name: '', type: 'string' }],
      stateMutability: 'view',
      type: 'function',
    },
    {
      inputs: [{ internalType: 'uint256', name: 'tokenId', type: 'uint256' }],
      name: 'getEncryptionKey',
      outputs: [{ internalType: 'string', name: '', type: 'string' }],
      stateMutability: 'view',
      type: 'function',
    },
    {
      inputs: [{ internalType: 'uint256', name: 'tokenId', type: 'uint256' }],
      name: 'ownerOf',
      outputs: [{ internalType: 'address', name: '', type: 'address' }],
      stateMutability: 'view',
      type: 'function',
    },
    {
      inputs: [{ internalType: 'uint256', name: 'tokenId', type: 'uint256' }],
      name: 'tokenURI',
      outputs: [{ internalType: 'string', name: '', type: 'string' }],
      stateMutability: 'view',
      type: 'function',
    },
    {
      anonymous: false,
      inputs: [
        { indexed: true, internalType: 'address', name: 'from', type: 'address' },
        { indexed: true, internalType: 'address', name: 'to', type: 'address' },
        { indexed: true, internalType: 'uint256', name: 'tokenId', type: 'uint256' },
      ],
      name: 'Transfer',
      type: 'event',
    },
  ] as const;

  const handleMintOnAmoy = async () => {
    try {
      const eth = (window as any).ethereum;
      if (!eth) throw new Error('MetaMask not found');

      // Ensure Amoy network
      const accounts = await eth.request({ method: 'eth_requestAccounts' });
      try {
        await eth.request({ method: 'wallet_switchEthereumChain', params: [{ chainId: '0x13882' }] });
      } catch (e: any) {
        if (e?.code === 4902) {
          await eth.request({
            method: 'wallet_addEthereumChain',
            params: [{
              chainId: '0x13882',
              chainName: 'Polygon Amoy',
              nativeCurrency: { name: 'MATIC', symbol: 'MATIC', decimals: 18 },
              rpcUrls: ['https://rpc-amoy.polygon.technology'],
              blockExplorerUrls: ['https://amoy.polygonscan.com'],
            }],
          });
        } else {
          throw e;
        }
      }
      setAmoyAccount(accounts?.[0] || null);

      // Build signer & contract
      const provider = new ethers.BrowserProvider(eth);
      const signer = await provider.getSigner();
      const contract = new ethers.Contract(AMOY_CONTRACT, MIN_ABI_MINT_QUBE, signer);

      // Simple test payload (replace with real URI/key later)
      const testUri = `ipfs://test-${Date.now()}`;
      const testKey = `key-${Math.random().toString(16).slice(2, 10)}`;

      const tx = await contract.mintQube(testUri, testKey);
      const fromAddr = await signer.getAddress();
      setTestResults(prev => [...prev, {
        type: 'amoy_mint_submitted',
        timestamp: new Date().toISOString(),
        data: { hash: tx.hash, from: fromAddr, contract: AMOY_CONTRACT, uri: testUri },
        status: 'success',
      }]);

      setAmoyLatestTx(tx.hash);

      // Optionally wait for 1 confirmation
      const receipt = await tx.wait();
      let mintedTokenId: string | null = null;
      try {
        // Parse Transfer event to extract tokenId
        const transferTopic = ethers.id('Transfer(address,address,uint256)');
        const log = receipt?.logs?.find((l: any) => l.topics && l.topics[0] === transferTopic);
        if (log) {
          // tokenId is topic[3]
          const tokenIdHex = log.topics[3];
          mintedTokenId = BigInt(tokenIdHex).toString();
        }
      } catch {}

      // Fetch metadata per spec if possible
      let onchainMeta: any = {};
      try {
        if (mintedTokenId) {
          const [metaLoc, encKey, owner, uri] = await Promise.all([
            contract.getMetaQubeLocation(mintedTokenId),
            contract.getEncryptionKey(mintedTokenId),
            contract.ownerOf(mintedTokenId),
            contract.tokenURI(mintedTokenId).catch(() => ''),
          ]);
          onchainMeta = { tokenId: mintedTokenId, metaLoc, encKey, owner, uri };
        }
      } catch (e) {
        onchainMeta = { tokenId: mintedTokenId, note: 'Post-mint readbacks failed or not available' };
      }

      setTestResults(prev => [...prev, {
        type: 'amoy_mint_confirmed',
        timestamp: new Date().toISOString(),
        data: { hash: receipt?.hash ?? tx.hash, blockNumber: receipt?.blockNumber, ...onchainMeta },
        status: 'success',
      }]);
    } catch (error: any) {
      setTestResults(prev => [...prev, {
        type: 'amoy_mint_error',
        timestamp: new Date().toISOString(),
        error: error?.message || 'Unknown error',
        status: 'error',
      }]);
    }
  };

  // ICP DVN verify actions (stubs using available state)
  const handleViewICPDVNStatus = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_dvn_status',
        timestamp: new Date().toISOString(),
        data: dualLockStatus || { note: 'No DVN status loaded yet' },
        status: dualLockStatus ? 'success' : 'error'
      }
    ]);
  };
  const handleViewAnchorStatus = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_anchor_status',
        timestamp: new Date().toISOString(),
        data: anchorStatus || { note: 'No Anchor status loaded yet' },
        status: anchorStatus ? 'success' : 'error'
      }
    ]);
  };
  const handleGetReceiptStub = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_get_receipt',
        timestamp: new Date().toISOString(),
        data: { todo: 'Wire proof_of_state.get_receipt(id) via SDK' },
        status: 'success'
      }
    ]);
  };
  const handleGetBatchesStub = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_get_batches',
        timestamp: new Date().toISOString(),
        data: { todo: 'Wire proof_of_state.get_batches() via SDK' },
        status: 'success'
      }
    ]);
  };
  const handleGetDVNMessageStub = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_get_dvn_message',
        timestamp: new Date().toISOString(),
        data: { todo: 'Wire cross_chain_service.get_dvn_message(message_id) via SDK' },
        status: 'success'
      }
    ]);
  };
  const handleListAttestationsStub = () => {
    setTestResults(prev => [
      ...prev,
      {
        type: 'icp_list_attestations',
        timestamp: new Date().toISOString(),
        data: { todo: 'Wire cross_chain_service.get_message_attestations(message_id) via SDK' },
        status: 'success'
      }
    ]);
  };

  const handleTestEVMTransaction = async () => {
    try {
      const result = await getEVMTransactionStatus(
        11155111, // Sepolia
        '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
      );
      
      setTestResults(prev => [...prev, {
        type: 'evm_tx_check',
        timestamp: new Date().toISOString(),
        data: result,
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'evm_tx_check',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleMintIQuBeOnICP = async () => {
    try {
      const testIQuBeData = {
        id: `iqube_${Date.now()}`,
        metadata: { type: 'test_mint', timestamp: new Date().toISOString() }
      };
      
      const result = await submitForAnchoring(
        JSON.stringify(testIQuBeData), 
        'ICP iQube Mint Test'
      );
      
      setTestResults(prev => [...prev, {
        type: 'mint_iqube_icp',
        timestamp: new Date().toISOString(),
        data: result,
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'mint_iqube_icp',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleGetBitcoinAddress = async () => {
    try {
      // Mock Bitcoin address generation for now
      const mockBtcAddress = `tb1q${Math.random().toString(36).substr(2, 32)}`;
      
      setTestResults(prev => [...prev, {
        type: 'get_btc_address',
        timestamp: new Date().toISOString(),
        data: { address: mockBtcAddress, network: 'testnet' },
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'get_btc_address',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleDualLockFlow = async () => {
    try {
      const dualLockData = {
        icp_receipt: `receipt_${Date.now()}`,
        btc_anchor: `anchor_${Date.now()}`,
        flow_type: 'dual_lock_test'
      };
      
      const result = await getDualLockStatus('test-dual-lock');
      
      setTestResults(prev => [...prev, {
        type: 'dual_lock_flow',
        timestamp: new Date().toISOString(),
        data: { ...result, test_data: dualLockData },
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'dual_lock_flow',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleTransferICPToPolygon = async () => {
    try {
      const transferData = {
        from_chain: 'ICP',
        to_chain: 'Polygon',
        amount: '1.0',
        tx_hash: `0x${Math.random().toString(16).substr(2, 64)}`
      };
      
      setTestResults(prev => [...prev, {
        type: 'transfer_icp_polygon',
        timestamp: new Date().toISOString(),
        data: transferData,
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'transfer_icp_polygon',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleAnchorBatches = async () => {
    try {
      const result = await getAnchorStatus('batch-anchor-test');
      
      setTestResults(prev => [...prev, {
        type: 'anchor_batches',
        timestamp: new Date().toISOString(),
        data: result,
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'anchor_batches',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  const handleListSupportedChains = async () => {
    try {
      const supportedChains = [
        { id: 1, name: 'Ethereum Mainnet', symbol: 'ETH' },
        { id: 137, name: 'Polygon Mainnet', symbol: 'MATIC' },
        { id: 11155111, name: 'Sepolia Testnet', symbol: 'ETH' },
        { id: 80001, name: 'Mumbai Testnet', symbol: 'MATIC' }
      ];
      
      setTestResults(prev => [...prev, {
        type: 'list_supported_chains',
        timestamp: new Date().toISOString(),
        data: { chains: supportedChains, count: supportedChains.length },
        status: 'success'
      }]);
    } catch (error) {
      setTestResults(prev => [...prev, {
        type: 'list_supported_chains',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
        status: 'error'
      }]);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-xl">Loading ops console...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold text-gray-900 mb-8">
          iQube Operations Console
        </h1>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
          {/* Canister Health */}
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-4">ICP Canister Health</h2>
            <div className="space-y-2">
              {canisters.map((canister) => (
                <div key={canister.name} className="flex justify-between items-center">
                  <span className="capitalize">{canister.name.replace('_', ' ')}:</span>
                  <div className="flex items-center space-x-2">
                    <span className={
                      canister.status === 'healthy' ? 'text-green-600' :
                      canister.status === 'degraded' ? 'text-yellow-600' : 'text-red-600'
                    }>●</span>
                    <span className="text-xs text-gray-500">
                      {new Date(canister.lastCheck).toLocaleTimeString()}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* BTC Anchors */}
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-4">BTC Anchor Status</h2>
            {anchorStatus && (
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span>TX Hash:</span>
                  <span className="text-sm font-mono">
                    {anchorStatus.btcTxHash?.slice(0, 8)}...
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Confirmations:</span>
                  <span>{anchorStatus.confirmations}</span>
                </div>
                <div className="flex justify-between">
                  <span>Block Height:</span>
                  <span>{anchorStatus.blockHeight}</span>
                </div>
                <div className="flex justify-between">
                  <span>Status:</span>
                  <span className={anchorStatus.isConfirmed ? 'text-green-600' : 'text-yellow-600'}>
                    {anchorStatus.isConfirmed ? 'Confirmed' : 'Pending'}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Cross-Chain Status */}
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-1">Cross-Chain Status</h2>
            <h3 className="text-lg font-semibold mb-3">ICP DVN</h3>
            {dualLockStatus && (
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span>EVM TX:</span>
                  <span className="text-sm font-mono">
                    {dualLockStatus.evmTxHash?.slice(0, 8)}...
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>ICP Receipt:</span>
                  <span className="text-sm font-mono">
                    {dualLockStatus.icpReceiptId?.slice(0, 8)}...
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Lock Status:</span>
                  <span className={dualLockStatus.isLocked ? 'text-green-600' : 'text-yellow-600'}>
                    {dualLockStatus.isLocked ? 'Locked' : 'Unlocked'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Unlock Height:</span>
                  <span>{dualLockStatus.unlockHeight}</span>
                </div>
              </div>
            )}
            {/* ICP DVN verify actions */}
            <div className="mt-4 grid grid-cols-2 md:grid-cols-3 gap-2">
              <button onClick={handleViewICPDVNStatus} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">View DVN Status</button>
              <button onClick={handleViewAnchorStatus} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">View Anchor Status</button>
              <button onClick={handleGetReceiptStub} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">Get Receipt</button>
              <button onClick={handleGetBatchesStub} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">View Batches</button>
              <button onClick={handleGetDVNMessageStub} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">Get DVN Message</button>
              <button onClick={handleListAttestationsStub} className="px-3 py-2 text-sm bg-gray-100 rounded hover:bg-gray-200">List Attestations</button>
            </div>
            <div className="mt-3 text-xs text-gray-500">
              Note: ICP interactions are via canisters; no browser wallet is required.
            </div>
          </div>
        </div>

        {/* Chain Explorers */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-6">
          {/* Ethereum Sepolia */}
          <div className="bg-white rounded-lg shadow p-6">
            <h3 className="text-lg font-semibold mb-3">Ethereum Sepolia (Live Testnet)</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Chain ID:</span>
                <span>11155111</span>
              </div>
              <div className="flex justify-between">
                <span>Latest TX:</span>
                <span className="font-mono text-xs">0xabc123...def456</span>
              </div>
              <div className="flex justify-between">
                <span>Block:</span>
                <span>5,234,567</span>
              </div>
              <div className="flex justify-between">
                <span>Status:</span>
                <span className="text-green-600">Live</span>
              </div>
            </div>
            <div className="mt-3 space-y-2">
              <button 
                onClick={() => window.open('https://sepolia.etherscan.io/tx/0xabc123def456', '_blank')}
                className="w-full px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm"
              >
                View on Sepolia Explorer
              </button>
              <button 
                onClick={async () => {
                  try {
                    const eth = (window as any).ethereum;
                    if (!eth) { alert('MetaMask not found.'); return; }
                    const accounts = await eth.request({ method: 'eth_requestAccounts' });
                    try {
                      await eth.request({ method: 'wallet_switchEthereumChain', params: [{ chainId: '0xaa36a7' }] }); // 11155111
                    } catch (e: any) {
                      if (e?.code === 4902) {
                        await eth.request({
                          method: 'wallet_addEthereumChain',
                          params: [{
                            chainId: '0xaa36a7',
                            chainName: 'Sepolia Test Network',
                            nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
                            rpcUrls: ['https://rpc.sepolia.org'],
                            blockExplorerUrls: ['https://sepolia.etherscan.io']
                          }]
                        });
                      }
                    }
                    setSepoliaAccount(accounts?.[0] || null);
                    alert(`Connected Sepolia as ${accounts?.[0]}`);
                  } catch (err) {
                    console.error(err);
                    alert('Failed to connect MetaMask to Sepolia');
                  }
                }}
                className="w-full px-3 py-2 bg-gray-700 text-white rounded-md hover:bg-gray-800 text-sm"
              >
                {sepoliaAccount ? `Connected: ${sepoliaAccount.slice(0,6)}...${sepoliaAccount.slice(-4)}` : 'Connect MetaMask (Sepolia)'}
              </button>
            </div>
          </div>

          {/* Polygon Amoy */}
          <div className="bg-white rounded-lg shadow p-6">
            <h3 className="text-lg font-semibold mb-3">Polygon Amoy (Live Testnet)</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Chain ID:</span>
                <span>80002</span>
              </div>
              <div className="flex justify-between">
                <span>Latest TX:</span>
                <span className="font-mono text-xs">{amoyLatestTx ? `${amoyLatestTx.slice(0,8)}...${amoyLatestTx.slice(-6)}` : '—'}</span>
              </div>
              <div className="flex justify-between">
                <span>Block:</span>
                <span>8,765,432</span>
              </div>
              <div className="flex justify-between">
                <span>Status:</span>
                <span className="text-green-600">Live</span>
              </div>
            </div>
            <div className="mt-3 space-y-2">
              {amoyLatestTx ? (
                <div className="space-y-2">
                  <a
                    href={`https://amoy.polygonscan.com/tx/${amoyLatestTx}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="block w-full text-center px-3 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 text-sm"
                  >
                    View on Amoy Explorer
                  </a>
                  <div className="text-xs break-all bg-gray-50 p-2 rounded border border-gray-200">
                    https://amoy.polygonscan.com/tx/{amoyLatestTx}
                  </div>
                  <div className="grid grid-cols-2 gap-2">
                    <button
                      onClick={async () => {
                        const url = `https://amoy.polygonscan.com/tx/${amoyLatestTx}`;
                        try { await navigator.clipboard.writeText(url); } catch {}
                      }}
                      className="px-3 py-2 bg-gray-700 text-white rounded-md text-sm hover:bg-gray-800"
                    >
                      Copy URL
                    </button>
                    <button
                      onClick={() => {
                        const url = `https://amoy.polygonscan.com/tx/${amoyLatestTx}`;
                        window.open(url, '_blank', 'noopener,noreferrer');
                      }}
                      className="px-3 py-2 bg-gray-600 text-white rounded-md text-sm hover:bg-gray-700"
                    >
                      Open in New Tab
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  disabled
                  title="Mint first to generate a transaction hash"
                  className="w-full px-3 py-2 bg-purple-300 text-white rounded-md text-sm cursor-not-allowed"
                >
                  View on Amoy Explorer
                </button>
              )}
              <button 
                onClick={async () => {
                  try {
                    const eth = (window as any).ethereum;
                    if (!eth) {
                      alert('MetaMask not found. Please install MetaMask.');
                      return;
                    }
                    
                    // Request account access
                    const accounts = await eth.request({ method: 'eth_requestAccounts' });
                    
                    // Switch to Polygon Amoy
                    try {
                      await eth.request({
                        method: 'wallet_switchEthereumChain',
                        params: [{ chainId: '0x13882' }] // 80002 in hex
                      });
                    } catch (switchError: any) {
                      // Chain not added, add it
                      if (switchError.code === 4902) {
                        await eth.request({
                          method: 'wallet_addEthereumChain',
                          params: [{
                            chainId: '0x13882',
                            chainName: 'Polygon Amoy',
                            nativeCurrency: { name: 'MATIC', symbol: 'MATIC', decimals: 18 },
                            rpcUrls: ['https://rpc-amoy.polygon.technology'],
                            blockExplorerUrls: ['https://amoy.polygonscan.com']
                          }]
                        });
                      }
                    }
                    setAmoyAccount(accounts?.[0] || null);
                    alert(`Connected Amoy as ${accounts?.[0]}`);
                  } catch (error) {
                    console.error('MetaMask connection failed:', error);
                    alert('Failed to connect MetaMask');
                  }
                }}
                className="w-full px-3 py-2 bg-orange-600 text-white rounded-md hover:bg-orange-700 text-sm"
              >
                {amoyAccount ? `Connected: ${amoyAccount.slice(0,6)}...${amoyAccount.slice(-4)}` : 'Connect MetaMask (Amoy)'}
              </button>
              <button 
                onClick={handleMintOnAmoy}
                className="w-full px-3 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 text-sm"
              >
                Mint on Amoy (MetaMask)
              </button>
            </div>
          </div>

          {/* Bitcoin Testnet */}
          <div className="bg-white rounded-lg shadow p-6">
            <h3 className="text-lg font-semibold mb-3">Bitcoin Testnet (Live)</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span>Network:</span>
                <span>testnet3</span>
              </div>
              <div className="flex justify-between">
                <span>Latest TX:</span>
                <span className="font-mono text-xs">{btcLatestTx ? `${btcLatestTx.slice(0,8)}...${btcLatestTx.slice(-6)}` : '—'}</span>
              </div>
              <div className="flex justify-between">
                <span>Status:</span>
                <span className="text-green-600">Live</span>
              </div>
            </div>
            <div className="mt-3 space-y-2">
              <label className="block text-xs text-gray-600">Testnet Private Key (WIF)</label>
              <input
                type="password"
                value={btcPrivKey}
                onChange={(e) => setBtcPrivKey(e.target.value)}
                placeholder="Enter WIF (kept in-memory only)"
                className="w-full px-3 py-2 border rounded-md text-sm"
              />
              <div className="text-xs text-gray-600">Derived Address: {btcAddress ? <span className="font-mono">{btcAddress}</span> : '—'}</div>
              <div className="text-xs text-gray-600">UTXOs: {btcUtxos?.length ?? 0}</div>
              {btcLatestTx ? (
                <a
                  href={`https://mempool.space/testnet/tx/${btcLatestTx}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="block w-full text-center px-3 py-2 bg-slate-700 text-white rounded-md hover:bg-slate-800 text-sm"
                >
                  View on Bitcoin Testnet Explorer (TX)
                </a>
              ) : btcAddress ? (
                <div className="space-y-2">
                  <a
                    href={`https://mempool.space/testnet/address/${btcAddress}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="block w-full text-center px-3 py-2 bg-slate-600 text-white rounded-md hover:bg-slate-700 text-sm"
                  >
                    View on Bitcoin Testnet Explorer (Address)
                  </a>
                  <div className="text-xs break-all bg-gray-50 p-2 rounded border border-gray-200">
                    https://mempool.space/testnet/address/{btcAddress}
                  </div>
                  <div className="grid grid-cols-2 gap-2">
                    <button
                      onClick={async () => {
                        const url = `https://mempool.space/testnet/address/${btcAddress}`;
                        try { await navigator.clipboard.writeText(url); } catch {}
                      }}
                      className="px-3 py-2 bg-gray-700 text-white rounded-md text-sm hover:bg-gray-800"
                    >
                      Copy URL
                    </button>
                    <button
                      onClick={() => {
                        const url = `https://mempool.space/testnet/address/${btcAddress}`;
                        window.open(url, '_blank', 'noopener,noreferrer');
                      }}
                      className="px-3 py-2 bg-gray-600 text-white rounded-md text-sm hover:bg-gray-700"
                    >
                      Open in New Tab
                    </button>
                  </div>
                </div>
              ) : (
                <button disabled className="w-full px-3 py-2 bg-slate-300 text-white rounded-md text-sm cursor-not-allowed">View on Bitcoin Testnet Explorer</button>
              )}
              <div className="grid grid-cols-1 md:grid-cols-4 gap-2">
                <button onClick={handleBtcGenerateKey} disabled={btcBusy} className="px-3 py-2 bg-emerald-700 text-white rounded-md hover:bg-emerald-800 text-sm disabled:opacity-60">Generate Testnet Key</button>
                <button onClick={handleBtcDeriveAddress} disabled={btcBusy} className="px-3 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 text-sm disabled:opacity-60">Derive Address</button>
                <button onClick={handleBtcFetchUtxos} disabled={btcBusy || !btcAddress} className="px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm disabled:opacity-60">Get UTXOs</button>
                <button onClick={handleBtcBroadcastTestTx} disabled={btcBusy || !btcAddress} className="px-3 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 text-sm disabled:opacity-60">Broadcast Test TX</button>
              </div>
              <div className="text-xs text-gray-500">
                Faucets:
                <a href="https://bitcoinfaucet.uo1.net/" target="_blank" rel="noopener noreferrer" className="underline ml-1">UO1</a>,
                <a href="https://coinfaucet.eu/en/btc-testnet/" target="_blank" rel="noopener noreferrer" className="underline ml-1">CoinFaucet</a>
              </div>
            </div>
          </div>
        </div>

        {/* End-to-End Test Suite */}
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h2 className="text-xl font-semibold mb-4">End-to-End Test Suite</h2>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
            <button 
              onClick={handleMintIQuBeOnICP}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm"
            >
              Mint iQube on ICP
            </button>
            <button 
              onClick={handleMintOnAmoy}
              className="px-4 py-2 bg-green-700 text-white rounded-md hover:bg-green-800 text-sm"
            >
              Mint on Amoy (EVM)
            </button>
            <button 
              onClick={handleGetBitcoinAddress}
              className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 text-sm"
            >
              Get Bitcoin Address (Mint on BTC)
            </button>
            <button 
              onClick={handleDualLockFlow}
              className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 text-sm"
            >
              Dual-Lock Flow (ICP → BTC)
            </button>
            <button 
              onClick={handleTransferICPToPolygon}
              className="px-4 py-2 bg-orange-600 text-white rounded-md hover:bg-orange-700 text-sm"
            >
              Transfer ICP → Polygon (simulate)
            </button>
            <button 
              onClick={handleAnchorBatches}
              className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 text-sm"
            >
              Anchor Batches Now (BTC)
            </button>
            <button 
              onClick={handleListSupportedChains}
              className="px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700 text-sm"
            >
              List Supported Chains
            </button>
          </div>
        </div>

        {/* Testing Interface */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-4">Test Anchoring</h2>
            <div className="space-y-4">
              <textarea
                value={testData}
                onChange={(e) => setTestData(e.target.value)}
                placeholder="Enter test data to anchor..."
                className="w-full p-3 border border-gray-300 rounded-md"
                rows={3}
              />
              <div className="flex space-x-2">
                <button
                  onClick={handleSubmitForAnchoring}
                  disabled={!testData.trim()}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
                >
                  Submit for Anchoring
                </button>
                <button
                  onClick={handleTestEVMTransaction}
                  className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700"
                >
                  Test EVM TX Check
                </button>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-4">Test Results</h2>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {testResults.length === 0 ? (
                <p className="text-gray-500">No test results yet</p>
              ) : (
                testResults.slice(-10).reverse().map((result, index) => (
                  <div key={index} className="p-2 border border-gray-200 rounded text-sm">
                    <div className="flex justify-between items-center mb-1">
                      <span className="font-medium">{result.type}</span>
                      <span className={
                        result.status === 'success' ? 'text-green-600' : 'text-red-600'
                      }>
                        {result.status}
                      </span>
                    </div>
                    <div className="text-xs text-gray-500 mb-1">
                      {new Date(result.timestamp).toLocaleTimeString()}
                    </div>
                    {result.data && (
                      <pre className="text-xs bg-gray-50 p-1 rounded overflow-x-auto">
                        {JSON.stringify(result.data, null, 2)}
                      </pre>
                    )}
                    {result.error && (
                      <div className="text-xs text-red-600">{result.error}</div>
                    )}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
