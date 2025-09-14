// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title QOYN Token
 * @dev ERC20 token with 21M total supply and 21-year emission schedule
 */
contract QOYN is ERC20, Ownable {
    uint256 public constant MAX_SUPPLY = 21_000_000 * 10**18; // 21M tokens
    uint256 public constant EMISSION_DURATION = 21 * 365 days; // 21 years
    
    uint256 public immutable deploymentTime;
    uint256 public totalMinted;
    
    mapping(address => bool) public minters;
    
    event MinterAdded(address indexed minter);
    event MinterRemoved(address indexed minter);
    
    constructor() ERC20("QOYN", "QOYN") {
        deploymentTime = block.timestamp;
    }
    
    modifier onlyMinter() {
        require(minters[msg.sender], "QOYN: caller is not a minter");
        _;
    }
    
    function addMinter(address minter) external onlyOwner {
        minters[minter] = true;
        emit MinterAdded(minter);
    }
    
    function removeMinter(address minter) external onlyOwner {
        minters[minter] = false;
        emit MinterRemoved(minter);
    }
    
    function mint(address to, uint256 amount) external onlyMinter {
        require(totalMinted + amount <= maxMintableAtTime(), "QOYN: exceeds emission schedule");
        require(totalMinted + amount <= MAX_SUPPLY, "QOYN: exceeds max supply");
        
        totalMinted += amount;
        _mint(to, amount);
    }
    
    function maxMintableAtTime() public view returns (uint256) {
        if (block.timestamp >= deploymentTime + EMISSION_DURATION) {
            return MAX_SUPPLY;
        }
        
        uint256 elapsed = block.timestamp - deploymentTime;
        return (MAX_SUPPLY * elapsed) / EMISSION_DURATION;
    }
    
    function remainingMintable() external view returns (uint256) {
        uint256 maxMintable = maxMintableAtTime();
        return maxMintable > totalMinted ? maxMintable - totalMinted : 0;
    }
}
