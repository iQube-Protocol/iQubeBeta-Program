// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Counters.sol";

/**
 * @title IQubeClasses
 * @dev ERC721 for iQube class templates (MetaQube definitions)
 */
contract IQubeClasses is ERC721, Ownable {
    using Counters for Counters.Counter;
    
    Counters.Counter private _tokenIds;
    
    struct IQubeClass {
        string name;
        string description;
        string iQubeType; // DataQube, ContentQube, ToolQube, ModelQube, AigentQube
        string businessModel; // Buy, Sell, Rent, Lease, Subscribe, Stake, License, Donate
        uint256 price;
        uint8 sensitivityScore;
        uint8 accuracyScore;
        uint8 verifiabilityScore;
        uint8 riskScore;
        string metadataURI;
        address creator;
        uint256 instanceCount;
        bool isActive;
    }
    
    mapping(uint256 => IQubeClass) public classes;
    mapping(address => bool) public authorizedMinters;
    
    event ClassCreated(uint256 indexed classId, address indexed creator, string name);
    event ClassUpdated(uint256 indexed classId);
    event ClassDeactivated(uint256 indexed classId);
    event InstanceMinted(uint256 indexed classId, uint256 newInstanceCount);
    
    constructor() ERC721("iQube Classes", "IQCLASS") {}
    
    modifier onlyAuthorized() {
        require(authorizedMinters[msg.sender] || msg.sender == owner(), "Not authorized to mint");
        _;
    }
    
    function setAuthorizedMinter(address minter, bool authorized) external onlyOwner {
        authorizedMinters[minter] = authorized;
    }
    
    function createClass(
        string memory name,
        string memory description,
        string memory iQubeType,
        string memory businessModel,
        uint256 price,
        uint8 sensitivityScore,
        uint8 accuracyScore,
        uint8 verifiabilityScore,
        uint8 riskScore,
        string memory metadataURI
    ) external onlyAuthorized returns (uint256) {
        require(bytes(name).length > 0, "Name cannot be empty");
        require(sensitivityScore <= 10 && accuracyScore <= 10 && verifiabilityScore <= 10 && riskScore <= 10, "Scores must be 0-10");
        
        _tokenIds.increment();
        uint256 newClassId = _tokenIds.current();
        
        classes[newClassId] = IQubeClass({
            name: name,
            description: description,
            iQubeType: iQubeType,
            businessModel: businessModel,
            price: price,
            sensitivityScore: sensitivityScore,
            accuracyScore: accuracyScore,
            verifiabilityScore: verifiabilityScore,
            riskScore: riskScore,
            metadataURI: metadataURI,
            creator: msg.sender,
            instanceCount: 0,
            isActive: true
        });
        
        _mint(msg.sender, newClassId);
        emit ClassCreated(newClassId, msg.sender, name);
        
        return newClassId;
    }
    
    function updateClass(
        uint256 classId,
        string memory name,
        string memory description,
        uint256 price,
        string memory metadataURI
    ) external {
        require(_exists(classId), "Class does not exist");
        require(ownerOf(classId) == msg.sender, "Not class owner");
        
        IQubeClass storage class = classes[classId];
        class.name = name;
        class.description = description;
        class.price = price;
        class.metadataURI = metadataURI;
        
        emit ClassUpdated(classId);
    }
    
    function deactivateClass(uint256 classId) external {
        require(_exists(classId), "Class does not exist");
        require(ownerOf(classId) == msg.sender || msg.sender == owner(), "Not authorized");
        
        classes[classId].isActive = false;
        emit ClassDeactivated(classId);
    }
    
    function incrementInstanceCount(uint256 classId) external onlyAuthorized {
        require(_exists(classId), "Class does not exist");
        require(classes[classId].isActive, "Class is not active");
        
        classes[classId].instanceCount++;
        emit InstanceMinted(classId, classes[classId].instanceCount);
    }
    
    function getClass(uint256 classId) external view returns (IQubeClass memory) {
        require(_exists(classId), "Class does not exist");
        return classes[classId];
    }
    
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(_exists(tokenId), "URI query for nonexistent token");
        return classes[tokenId].metadataURI;
    }
    
    function totalClasses() external view returns (uint256) {
        return _tokenIds.current();
    }
}
