// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "./IQubeClasses.sol";

/**
 * @title IQubeInstances
 * @dev ERC721 for iQube instances (actual data containers)
 */
contract IQubeInstances is ERC721, Ownable {
    using Counters for Counters.Counter;
    
    Counters.Counter private _tokenIds;
    IQubeClasses public immutable classesContract;
    
    struct IQubeInstance {
        uint256 classId;
        string blakQubeDataHash; // IPFS hash or encrypted data reference
        string btcAnchorTxId;
        uint256 anchorBlockHeight;
        bool isDualLocked;
        bool isEncrypted;
        address dataSubject;
        uint256 createdAt;
        string metadataURI;
    }
    
    mapping(uint256 => IQubeInstance) public instances;
    mapping(address => bool) public authorizedMinters;
    
    event InstanceMinted(
        uint256 indexed instanceId,
        uint256 indexed classId,
        address indexed owner,
        address dataSubject
    );
    event InstanceAnchored(uint256 indexed instanceId, string btcTxId, uint256 blockHeight);
    event InstanceDecrypted(uint256 indexed instanceId);
    
    constructor(address _classesContract) ERC721("iQube Instances", "IQINST") {
        classesContract = IQubeClasses(_classesContract);
    }
    
    modifier onlyAuthorized() {
        require(authorizedMinters[msg.sender] || msg.sender == owner(), "Not authorized to mint");
        _;
    }
    
    function setAuthorizedMinter(address minter, bool authorized) external onlyOwner {
        authorizedMinters[minter] = authorized;
    }
    
    function mintInstance(
        uint256 classId,
        address to,
        address dataSubject,
        string memory blakQubeDataHash,
        bool isEncrypted,
        string memory metadataURI
    ) external onlyAuthorized returns (uint256) {
        // Verify class exists and is active
        IQubeClasses.IQubeClass memory class = classesContract.getClass(classId);
        require(class.isActive, "Class is not active");
        
        _tokenIds.increment();
        uint256 newInstanceId = _tokenIds.current();
        
        instances[newInstanceId] = IQubeInstance({
            classId: classId,
            blakQubeDataHash: blakQubeDataHash,
            btcAnchorTxId: "",
            anchorBlockHeight: 0,
            isDualLocked: false,
            isEncrypted: isEncrypted,
            dataSubject: dataSubject,
            createdAt: block.timestamp,
            metadataURI: metadataURI
        });
        
        _mint(to, newInstanceId);
        
        // Increment instance count in class
        classesContract.incrementInstanceCount(classId);
        
        emit InstanceMinted(newInstanceId, classId, to, dataSubject);
        
        return newInstanceId;
    }
    
    function anchorToBitcoin(
        uint256 instanceId,
        string memory btcTxId,
        uint256 blockHeight
    ) external onlyAuthorized {
        require(_exists(instanceId), "Instance does not exist");
        
        IQubeInstance storage instance = instances[instanceId];
        instance.btcAnchorTxId = btcTxId;
        instance.anchorBlockHeight = blockHeight;
        instance.isDualLocked = true;
        
        emit InstanceAnchored(instanceId, btcTxId, blockHeight);
    }
    
    function decryptInstance(uint256 instanceId) external {
        require(_exists(instanceId), "Instance does not exist");
        require(ownerOf(instanceId) == msg.sender, "Not instance owner");
        
        IQubeInstance storage instance = instances[instanceId];
        require(instance.isEncrypted, "Instance is not encrypted");
        
        instance.isEncrypted = false;
        emit InstanceDecrypted(instanceId);
    }
    
    function updateBlakQubeData(uint256 instanceId, string memory newDataHash) external {
        require(_exists(instanceId), "Instance does not exist");
        require(ownerOf(instanceId) == msg.sender, "Not instance owner");
        
        instances[instanceId].blakQubeDataHash = newDataHash;
    }
    
    function getInstance(uint256 instanceId) external view returns (IQubeInstance memory) {
        require(_exists(instanceId), "Instance does not exist");
        return instances[instanceId];
    }
    
    function getInstancesByClass(uint256 classId) external view returns (uint256[] memory) {
        uint256 totalSupply = _tokenIds.current();
        uint256[] memory result = new uint256[](totalSupply);
        uint256 resultIndex = 0;
        
        for (uint256 i = 1; i <= totalSupply; i++) {
            if (instances[i].classId == classId) {
                result[resultIndex] = i;
                resultIndex++;
            }
        }
        
        // Resize array to actual length
        uint256[] memory finalResult = new uint256[](resultIndex);
        for (uint256 i = 0; i < resultIndex; i++) {
            finalResult[i] = result[i];
        }
        
        return finalResult;
    }
    
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(_exists(tokenId), "URI query for nonexistent token");
        return instances[tokenId].metadataURI;
    }
    
    function totalInstances() external view returns (uint256) {
        return _tokenIds.current();
    }
}
