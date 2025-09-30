// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/QriptoCent.sol";

contract DeployQriptoCentMultiChain is Script {
    // Deployment parameters - customize these for your deployment
    string constant NAME = "QriptoCent";
    string constant SYMBOL = "QCT";
    uint256 constant INITIAL_SUPPLY = 1_000 * 1e18; // 1,000 QCT
    uint256 constant MAX_SUPPLY = 100_000 * 1e18; // 100,000 QCT
    string constant CONTRACT_URI = "https://iqb-protocol.github.io/qct-metadata/contract.json";
    string constant TOKEN_URI = "https://iqb-protocol.github.io/qct-metadata/token.json";

    // Supported testnet chain IDs
    uint256[] public testnetChains = [
        11155111, // Sepolia
        80002,    // Polygon Amoy
        421614,   // Arbitrum Sepolia
        84532,    // Base Sepolia
        11155420  // Optimism Sepolia
    ];

    function run() external {
        // Get deployer address from environment or use default
        address admin = vm.envOr("DEPLOYER_ADDRESS", msg.sender);

        console.log("Deploying QriptoCent to multiple testnets...");
        console.log("Admin address:", admin);

        // Deploy to each testnet
        for (uint i = 0; i < testnetChains.length; i++) {
            uint256 chainId = testnetChains[i];
            console.log("\n--- Deploying to chain ID:", chainId, "---");

            // Skip if not on the target chain (for multi-chain deployment)
            if (block.chainid != chainId) {
                console.log("Skipping chain", chainId, "- not on target network");
                continue;
            }

            vm.startBroadcast();

            // Deploy QriptoCent contract
            QriptoCent qct = new QriptoCent(
                NAME,
                SYMBOL,
                INITIAL_SUPPLY,
                MAX_SUPPLY,
                admin,
                CONTRACT_URI,
                TOKEN_URI
            );

            // Set up supported chains
            for (uint j = 0; j < testnetChains.length; j++) {
                qct.setChainSupport(testnetChains[j], true);
            }

            vm.stopBroadcast();

            // Log deployment details
            console.log("✅ QriptoCent deployed at:", address(qct));
            console.log("Chain ID:", chainId);
            console.log("Contract address:", address(qct));
            console.log("Initial supply:", INITIAL_SUPPLY);
            console.log("Max supply:", MAX_SUPPLY);

            // Verify deployment
            require(qct.name() == NAME, "Name mismatch");
            require(qct.symbol() == SYMBOL, "Symbol mismatch");
            require(qct.totalSupply() == INITIAL_SUPPLY, "Supply mismatch");
            require(qct.maxSupply() == MAX_SUPPLY, "Max supply mismatch");

            console.log("✅ Deployment verified successfully!");

            // Export deployment info for use in other scripts
            vm.writeFile(
                string.concat("./deployments/", vm.toString(chainId), ".json"),
                string.concat(
                    '{"address":"', vm.toString(address(qct)), '","chainId":', vm.toString(chainId), '}'
                )
            );
        }
    }
}
