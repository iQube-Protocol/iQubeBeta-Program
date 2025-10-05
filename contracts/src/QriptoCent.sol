   // SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";

/// @title QriptoCent (QCT) - Cross-Chain Token for iQube Protocol
/// @notice ERC-20 token with mint/burn capabilities for cross-chain operations
/// @dev Supports multiple chains and DVN-controlled minting/burning
contract QriptoCent is ERC20, AccessControl, Pausable, ERC20Burnable {
    bytes32 public constant DVN_ROLE = keccak256("DVN_ROLE");
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant PAUSER_ROLE = keccak256("PAUSER_ROLE");

    // Token metadata
    string private _contractURI;
    string private _tokenURI;

    // Supply limits
    uint256 public maxSupply;
    uint256 public mintedSupply;

    // Chain-specific configurations
    mapping(uint256 => bool) public supportedChains;
    mapping(uint256 => address) public bridgeContracts;

    event Minted(address indexed to, uint256 amount, uint256 chainId);
    event Burned(address indexed from, uint256 amount, uint256 chainId);
    event ChainSupportUpdated(uint256 chainId, bool supported);
    event BridgeContractUpdated(uint256 chainId, address bridgeContract);
    event MaxSupplyUpdated(uint256 newMaxSupply);

    constructor(
        string memory name,
        string memory symbol,
        uint256 initialSupply,
        uint256 maxSupply_,
        address admin,
        string memory contractURI_,
        string memory tokenURI_
    ) ERC20(name, symbol) {
        require(admin != address(0), "QriptoCent: admin cannot be zero address");
        require(maxSupply_ > 0, "QriptoCent: max supply must be greater than 0");

        maxSupply = maxSupply_;
        _contractURI = contractURI_;
        _tokenURI = tokenURI_;

        // Grant roles to admin
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(DVN_ROLE, admin);
        _grantRole(MINTER_ROLE, admin);
        _grantRole(PAUSER_ROLE, admin);

        // Mint initial supply to admin if specified
        if (initialSupply > 0) {
            _mint(admin, initialSupply);
            mintedSupply = initialSupply;
            emit Minted(admin, initialSupply, block.chainid);
        }
    }

    /// @notice Mint tokens (restricted to MINTER_ROLE or DVN_ROLE)
    /// @param to Address to mint tokens to
    /// @param amount Amount of tokens to mint
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        require(to != address(0), "QriptoCent: cannot mint to zero address");
        require(amount > 0, "QriptoCent: amount must be greater than 0");
        require(mintedSupply + amount <= maxSupply, "QriptoCent: max supply exceeded");

        _mint(to, amount);
        mintedSupply += amount;

        emit Minted(to, amount, block.chainid);
    }

    /// @notice Burn tokens (restricted to DVN_ROLE)
    /// @param from Address to burn tokens from
    /// @param amount Amount of tokens to burn
    function burn(address from, uint256 amount) external onlyRole(DVN_ROLE) {
        require(from != address(0), "QriptoCent: cannot burn from zero address");
        require(amount > 0, "QriptoCent: amount must be greater than 0");
        require(balanceOf(from) >= amount, "QriptoCent: insufficient balance");

        _burn(from, amount);
        mintedSupply -= amount;

        emit Burned(from, amount, block.chainid);
    }

    /// @notice Cross-chain mint (called by bridge contracts)
    /// @param to Address to mint tokens to
    /// @param amount Amount of tokens to mint
    /// @param chainId Source chain ID
    function crossChainMint(
        address to,
        uint256 amount,
        uint256 chainId
    ) external {
        require(supportedChains[chainId], "QriptoCent: chain not supported");
        require(bridgeContracts[chainId] == msg.sender || hasRole(DVN_ROLE, msg.sender), "QriptoCent: unauthorized bridge");
        require(to != address(0), "QriptoCent: cannot mint to zero address");
        require(amount > 0, "QriptoCent: amount must be greater than 0");
        require(mintedSupply + amount <= maxSupply, "QriptoCent: max supply exceeded");

        _mint(to, amount);
        mintedSupply += amount;

        emit Minted(to, amount, chainId);
    }

    /// @notice Cross-chain burn (called by bridge contracts)
    /// @param from Address to burn tokens from
    /// @param amount Amount of tokens to burn
    /// @param chainId Destination chain ID
    function crossChainBurn(
        address from,
        uint256 amount,
        uint256 chainId
    ) external {
        require(supportedChains[chainId], "QriptoCent: chain not supported");
        require(bridgeContracts[chainId] == msg.sender || hasRole(DVN_ROLE, msg.sender), "QriptoCent: unauthorized bridge");
        require(from != address(0), "QriptoCent: cannot burn from zero address");
        require(amount > 0, "QriptoCent: amount must be greater than 0");
        require(balanceOf(from) >= amount, "QriptoCent: insufficient balance");

        _burn(from, amount);
        mintedSupply -= amount;

        emit Burned(from, amount, chainId);
    }

    /// @notice Set max supply (admin only)
    /// @param newMaxSupply New maximum supply
    function setMaxSupply(uint256 newMaxSupply) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newMaxSupply >= mintedSupply, "QriptoCent: new max supply below current minted");
        maxSupply = newMaxSupply;
        emit MaxSupplyUpdated(newMaxSupply);
    }

    /// @notice Add or remove chain support (admin only)
    /// @param chainId Chain ID to update
    /// @param supported Whether chain is supported
    function setChainSupport(uint256 chainId, bool supported) external onlyRole(DEFAULT_ADMIN_ROLE) {
        supportedChains[chainId] = supported;
        emit ChainSupportUpdated(chainId, supported);
    }

    /// @notice Set bridge contract for a chain (admin only)
    /// @param chainId Chain ID
    /// @param bridgeContract Bridge contract address
    function setBridgeContract(uint256 chainId, address bridgeContract) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(bridgeContract != address(0), "QriptoCent: bridge contract cannot be zero address");
        bridgeContracts[chainId] = bridgeContract;
        emit BridgeContractUpdated(chainId, bridgeContract);
    }

    /// @notice Pause token transfers (emergency)
    function pause() external onlyRole(PAUSER_ROLE) {
        _pause();
    }

    /// @notice Unpause token transfers
    function unpause() external onlyRole(PAUSER_ROLE) {
        _unpause();
    }

    /// @notice Get contract metadata URI
    function contractURI() external view returns (string memory) {
        return _contractURI;
    }

    /// @notice Get token metadata URI
    function tokenURI() external view returns (string memory) {
        return _tokenURI;
    }

    /// @notice Set contract metadata URI (admin only)
    function setContractURI(string memory contractURI_) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _contractURI = contractURI_;
    }

    /// @notice Set token metadata URI (admin only)
    function setTokenURI(string memory tokenURI_) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _tokenURI = tokenURI_;
    }

    /// @notice Get total minted supply
    function getMintedSupply() external view returns (uint256) {
        return mintedSupply;
    }

    /// @notice Get circulating supply (total - burned)
    function getCirculatingSupply() external view returns (uint256) {
        return mintedSupply;
    }

    /// @notice Check if chain is supported
    function isChainSupported(uint256 chainId) external view returns (bool) {
        return supportedChains[chainId];
    }

    /// @notice Get bridge contract for a chain
    function getBridgeContract(uint256 chainId) external view returns (address) {
        return bridgeContracts[chainId];
    }

    // Override required functions
    function _beforeTokenTransfer(address from, address to, uint256 amount)
        internal
        override
        whenNotPaused
    {
        super._beforeTokenTransfer(from, to, amount);
    }

    // Support for EIP-165 interface detection
    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(AccessControl)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
