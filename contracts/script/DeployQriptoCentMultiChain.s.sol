// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/QCTToken.sol";
import "../src/QCTStaking.sol";

contract DeployQriptoCentMultiChain is Script {
    function run() external {
        // Get private key from environment
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        // Start broadcasting transactions
        vm.startBroadcast(deployerPrivateKey);

        // Deploy QCT Token
        QCTToken qctToken = new QCTToken();
        console.log("QCT Token deployed to:", address(qctToken));

        // Deploy QCT Staking contract
        QCTStaking stakingContract = new QCTStaking(address(qctToken));
        console.log("QCT Staking deployed to:", address(stakingContract));

        // Create a staking pool (example configuration)
        stakingContract.createPool(1e15, 30 days); // 0.001 QCT per second per QCT staked, 30 day lock
        console.log("Staking pool created with ID: 1");

        vm.stopBroadcast();

        // Log deployment summary
        console.log("\n=== QCT Deployment Summary ===");
        console.log("Network:", block.chainid);
        console.log("QCT Token:", address(qctToken));
        console.log("Staking Contract:", address(stakingContract));
        console.log("Initial Pool ID: 1");
        console.log("Pool Reward Rate: 0.001 QCT/second per QCT staked");
        console.log("Pool Lock Period: 30 days");
    }
}
