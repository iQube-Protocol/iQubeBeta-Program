// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/QriptoCent.sol";

contract DeployQriptoCent is Script {
    function run() external {
        // Get deployment parameters from environment
        string memory name = vm.envString("QCT_NAME");
        string memory symbol = vm.envString("QCT_SYMBOL");
        uint256 initialSupply = vm.envUint("QCT_INITIAL_SUPPLY");
        uint256 maxSupply = vm.envUint("QCT_MAX_SUPPLY");
        address admin = vm.envAddress("QCT_ADMIN");
        string memory contractURI = vm.envString("QCT_CONTRACT_URI");
        string memory tokenURI = vm.envString("QCT_TOKEN_URI");

        // Start broadcasting transactions
        vm.startBroadcast();

        // Deploy QriptoCent contract
        QriptoCent qct = new QriptoCent(
            name,
            symbol,
            initialSupply,
            maxSupply,
            admin,
            contractURI,
            tokenURI
        );

        // Set up supported chains (all major testnets)
        qct.setChainSupport(11155111, true); // Sepolia
        qct.setChainSupport(80002, true);    // Polygon Amoy
        qct.setChainSupport(421614, true);   // Arbitrum Sepolia
        qct.setChainSupport(11155420, true); // Optimism Sepolia
        qct.setChainSupport(84532, true);    // Base Sepolia

        vm.stopBroadcast();

        // Log deployment details
        console.log("QriptoCent deployed at:", address(qct));
        console.log("Name:", name);
        console.log("Symbol:", symbol);
        console.log("Initial Supply:", initialSupply);
        console.log("Max Supply:", maxSupply);
        console.log("Admin:", admin);
        console.log("Contract URI:", contractURI);
        console.log("Token URI:", tokenURI);
    }
}
