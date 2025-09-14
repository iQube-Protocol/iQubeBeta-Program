'use client';

import { useState, useEffect } from 'react';
import { 
  getAnchorStatus, 
  getDualLockStatus, 
  submitForAnchoring, 
  getEVMTransactionStatus 
} from '@iqube/sdk-js';

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
            <h2 className="text-xl font-semibold mb-4">Cross-Chain Status</h2>
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
