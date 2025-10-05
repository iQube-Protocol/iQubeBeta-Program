# QriptoCent (QCT) Contract Deployment Guide
# This file provides instructions for deploying QCT contracts to testnets

## Prerequisites
1. Install Foundry: https://book.getfoundry.sh/getting-started/installation
2. Set up your wallet with testnet funds
3. Configure your environment variables

## Deployment Instructions

### 1. Set Environment Variables
Create a `.env` file in the contracts directory with:

```bash
# Deployer Configuration
DEPLOYER_ADDRESS=0xYourWalletAddress
PRIVATE_KEY=your_private_key_without_0x

# Ethereum Sepolia
ETH_RPC_URL_SEPOLIA=https://sepolia.infura.io/v3/YOUR_INFURA_KEY
ETHERSCAN_API_KEY=YOUR_ETHERSCAN_API_KEY

# Polygon Amoy
POLYGON_AMOY_RPC_URL=https://rpc-amoy.polygon.technology
POLYGONSCAN_API_KEY=YOUR_POLYGONSCAN_API_KEY

# Arbitrum Sepolia
ARBITRUM_SEPOLIA_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
ARBISCAN_API_KEY=YOUR_ARBISCAN_API_KEY

# Base Sepolia
BASE_SEPOLIA_RPC_URL=https://sepolia.base.org
BASESCAN_API_KEY=YOUR_BASESCAN_API_KEY

# Optimism Sepolia
OPTIMISM_SEPOLIA_RPC_URL=https://sepolia.optimism.io
OPTIMISM_API_KEY=YOUR_OPTIMISM_API_KEY
```

### 2. Deploy to Sepolia
```bash
cd contracts
forge script script/DeployQriptoCentMultiChain.s.sol \
  --rpc-url sepolia \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  --sig "run()" \
  --private-key $PRIVATE_KEY
```

### 3. Deploy to Polygon Amoy
```bash
forge script script/DeployQriptoCentMultiChain.s.sol \
  --rpc-url polygon_amoy \
  --broadcast \
  --verify \
  --etherscan-api-key $POLYGONSCAN_API_KEY \
  --sig "run()" \
  --private-key $PRIVATE_KEY
```

### 4. Deploy to Other Testnets
Repeat the process for:
- Arbitrum Sepolia
- Base Sepolia
- Optimism Sepolia

### 5. Update Environment Variables
After deployment, update your main `.env` file:

```bash
# Replace 0x000... with actual deployed addresses
NEXT_PUBLIC_QCT_CONTRACT_ETHEREUM_SEPOLIA=0xDeployedSepoliaAddress
NEXT_PUBLIC_QCT_CONTRACT_POLYGON_AMOY=0xDeployedAmoyAddress
# ... etc for all chains
```

## Contract Features
- ✅ ERC-20 with supply limits
- ✅ Multi-chain support
- ✅ DVN role-based access control
- ✅ Cross-chain minting/burning
- ✅ Pause/unpause functionality
- ✅ Event logging

## Post-Deployment Steps
1. Verify contracts on block explorers
2. Test minting/burning functions
3. Update canister configurations
4. Test QCT trading interface with real balances

## Security Notes
- Use a dedicated deployment wallet
- Verify all transactions before broadcasting
- Test on testnets before mainnet deployment
- Keep private keys secure and never commit them
