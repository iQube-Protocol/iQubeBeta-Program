// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/// @title QCTDemo - Minimal demo ERC-20 for cross-chain QCT rekey flows (Stage 1A)
/// @notice Mint/Burn restricted to DVN_ROLE for server-side/DVN agent orchestration.
contract QCTDemo is ERC20, AccessControl {
    bytes32 public constant DVN_ROLE = keccak256("DVN_ROLE");

    constructor(string memory name_, string memory symbol_) ERC20(name_, symbol_) {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function mint(address to, uint256 amount) external onlyRole(DVN_ROLE) {
        _mint(to, amount);
    }

    function burn(address from, uint256 amount) external onlyRole(DVN_ROLE) {
        _burn(from, amount);
    }
}
