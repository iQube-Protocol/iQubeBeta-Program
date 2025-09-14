import { getAnchorStatus, getDualLockStatus } from '@iqube/sdk-js'

export default function HomePage() {
  return (
    <main className="container mx-auto px-4 py-8">
      <h1 className="text-4xl font-bold mb-8">21 Sats Market</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="border rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4">iQube Marketplace</h2>
          <p className="text-gray-600 mb-4">
            Buy and sell iQube instances with Bitcoin ordinals and data shards.
          </p>
          <button className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600">
            Browse Market
          </button>
        </div>
        
        <div className="border rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4">Provenance Tracking</h2>
          <p className="text-gray-600 mb-4">
            Verify BTC anchor receipts and dual-lock status for your iQubes.
          </p>
          <button className="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600">
            Check Provenance
          </button>
        </div>
        
        <div className="border rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4">Token Payments</h2>
          <p className="text-gray-600 mb-4">
            Pay with $QOYN or $QCNT for seamless transactions.
          </p>
          <button className="bg-purple-500 text-white px-4 py-2 rounded hover:bg-purple-600">
            Connect Wallet
          </button>
        </div>
      </div>
    </main>
  )
}
