// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title QCNT Token
 * @dev Custodian-backed claims token with proof-of-reserves
 */
contract QCNT is ERC20, Ownable, Pausable {
    address public custodian;
    uint256 public reservesAttestation;
    uint256 public lastAttestationTime;
    uint256 public constant ATTESTATION_VALIDITY = 24 hours;
    
    mapping(address => bool) public minters;
    mapping(address => bool) public burners;
    
    event CustodianUpdated(address indexed oldCustodian, address indexed newCustodian);
    event ReservesAttested(uint256 amount, uint256 timestamp);
    event MinterAdded(address indexed minter);
    event MinterRemoved(address indexed minter);
    event BurnerAdded(address indexed burner);
    event BurnerRemoved(address indexed burner);
    
    constructor(address _custodian) ERC20("QCNT", "QCNT") {
        custodian = _custodian;
    }
    
    modifier onlyMinter() {
        require(minters[msg.sender], "QCNT: caller is not a minter");
        _;
    }
    
    modifier onlyBurner() {
        require(burners[msg.sender], "QCNT: caller is not a burner");
        _;
    }
    
    modifier validReserves() {
        require(
            block.timestamp <= lastAttestationTime + ATTESTATION_VALIDITY,
            "QCNT: reserves attestation expired"
        );
        _;
    }
    
    function setCustodian(address _custodian) external onlyOwner {
        address oldCustodian = custodian;
        custodian = _custodian;
        emit CustodianUpdated(oldCustodian, _custodian);
    }
    
    function addMinter(address minter) external onlyOwner {
        minters[minter] = true;
        emit MinterAdded(minter);
    }
    
    function removeMinter(address minter) external onlyOwner {
        minters[minter] = false;
        emit MinterRemoved(minter);
    }
    
    function addBurner(address burner) external onlyOwner {
        burners[burner] = true;
        emit BurnerAdded(burner);
    }
    
    function removeBurner(address burner) external onlyOwner {
        burners[burner] = false;
        emit BurnerRemoved(burner);
    }
    
    function attestReserves(uint256 amount) external {
        require(msg.sender == custodian, "QCNT: only custodian can attest");
        reservesAttestation = amount;
        lastAttestationTime = block.timestamp;
        emit ReservesAttested(amount, block.timestamp);
    }
    
    function mint(address to, uint256 amount) external onlyMinter validReserves whenNotPaused {
        require(totalSupply() + amount <= reservesAttestation, "QCNT: exceeds reserves");
        _mint(to, amount);
    }
    
    function burn(address from, uint256 amount) external onlyBurner whenNotPaused {
        _burn(from, amount);
    }
    
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    function reservesCoverage() external view returns (uint256) {
        if (totalSupply() == 0) return 100;
        return (reservesAttestation * 100) / totalSupply();
    }
    
    function isAttestationValid() external view returns (bool) {
        return block.timestamp <= lastAttestationTime + ATTESTATION_VALIDITY;
    }
}
