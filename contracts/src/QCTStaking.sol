// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/// @title QCTStaking - Staking contract for QriptoCent (QCT) tokens
/// @notice Allows QCT holders to stake tokens and earn rewards
/// @dev Supports multiple staking pools with different reward rates and lock periods
contract QCTStaking is Ownable, ReentrancyGuard, Pausable {
    IERC20 public qctToken;

    struct Stake {
        uint256 amount;
        uint256 stakedAt;
        uint256 rewardDebt;
        uint256 lastRewardClaim;
    }

    struct Pool {
        uint256 totalStaked;
        uint256 rewardRate; // Rewards per second per QCT staked
        uint256 lockPeriod; // Minimum staking period in seconds
        uint256 totalRewardsDistributed;
        bool active;
    }

    mapping(address => Stake) public stakes;
    mapping(uint256 => Pool) public pools;
    mapping(address => uint256) public userStakeCount;

    uint256 public poolCount;
    uint256 public totalStakedGlobal;
    uint256 public rewardPerTokenStored;

    event Staked(address indexed user, uint256 poolId, uint256 amount);
    event Unstaked(address indexed user, uint256 poolId, uint256 amount);
    event RewardsClaimed(address indexed user, uint256 amount);
    event PoolCreated(uint256 poolId, uint256 rewardRate, uint256 lockPeriod);

    constructor(address _qctToken) Ownable(msg.sender) {
        qctToken = IERC20(_qctToken);
        poolCount = 0;
    }

    /// @notice Create a new staking pool
    /// @param rewardRate Rewards per second per QCT staked (in wei)
    /// @param lockPeriod Minimum staking period in seconds
    function createPool(uint256 rewardRate, uint256 lockPeriod) external onlyOwner {
        poolCount++;
        pools[poolCount] = Pool({
            totalStaked: 0,
            rewardRate: rewardRate,
            lockPeriod: lockPeriod,
            totalRewardsDistributed: 0,
            active: true
        });

        emit PoolCreated(poolCount, rewardRate, lockPeriod);
    }

    /// @notice Stake QCT tokens in a pool
    /// @param poolId ID of the pool to stake in
    /// @param amount Amount of QCT to stake
    function stake(uint256 poolId, uint256 amount) external nonReentrant whenNotPaused {
        require(poolId > 0 && poolId <= poolCount, "Invalid pool ID");
        require(pools[poolId].active, "Pool is not active");
        require(amount > 0, "Cannot stake 0 tokens");
        require(qctToken.balanceOf(msg.sender) >= amount, "Insufficient QCT balance");

        Pool storage pool = pools[poolId];
        Stake storage userStake = stakes[msg.sender];

        // Update rewards before staking
        updateRewards(msg.sender);

        // Transfer tokens to contract
        require(qctToken.transferFrom(msg.sender, address(this), amount), "Token transfer failed");

        // Update stake
        userStake.amount += amount;
        userStake.stakedAt = block.timestamp;
        pool.totalStaked += amount;
        totalStakedGlobal += amount;

        // Update reward debt
        userStake.rewardDebt = (userStake.amount * rewardPerTokenStored) / 1e18;

        userStakeCount[msg.sender]++;

        emit Staked(msg.sender, poolId, amount);
    }

    /// @notice Unstake QCT tokens from a pool
    /// @param poolId ID of the pool to unstake from
    /// @param amount Amount of QCT to unstake
    function unstake(uint256 poolId, uint256 amount) external nonReentrant {
        require(poolId > 0 && poolId <= poolCount, "Invalid pool ID");
        require(amount > 0, "Cannot unstake 0 tokens");

        Stake storage userStake = stakes[msg.sender];
        require(userStake.amount >= amount, "Insufficient staked amount");

        Pool storage pool = pools[poolId];

        // Check lock period
        require(block.timestamp >= userStake.stakedAt + pool.lockPeriod, "Tokens are still locked");

        // Update rewards before unstaking
        updateRewards(msg.sender);

        // Update stake
        userStake.amount -= amount;
        pool.totalStaked -= amount;
        totalStakedGlobal -= amount;

        // Update reward debt
        userStake.rewardDebt = (userStake.amount * rewardPerTokenStored) / 1e18;

        // Transfer tokens back to user
        require(qctToken.transfer(msg.sender, amount), "Token transfer failed");

        emit Unstaked(msg.sender, poolId, amount);
    }

    /// @notice Claim accumulated rewards
    function claimRewards() external nonReentrant {
        updateRewards(msg.sender);

        Stake storage userStake = stakes[msg.sender];
        uint256 rewards = userStake.rewardDebt;

        require(rewards > 0, "No rewards to claim");

        userStake.rewardDebt = 0;
        userStake.lastRewardClaim = block.timestamp;

        // Mint new QCT tokens as rewards
        require(qctToken.transfer(msg.sender, rewards), "Reward transfer failed");

        pools[getUserPool(msg.sender)].totalRewardsDistributed += rewards;

        emit RewardsClaimed(msg.sender, rewards);
    }

    /// @notice Update reward calculations for a user
    /// @param account User address to update rewards for
    function updateRewards(address account) public {
        if (stakes[account].amount == 0) return;

        uint256 poolId = getUserPool(account);
        Pool storage pool = pools[poolId];

        // Calculate pending rewards
        uint256 pendingRewards = calculateRewards(account);

        if (pendingRewards > 0) {
            stakes[account].rewardDebt += pendingRewards;
        }

        rewardPerTokenStored += (block.timestamp * pool.rewardRate * 1e18) / totalStakedGlobal;
    }

    /// @notice Calculate pending rewards for a user
    /// @param account User address to calculate rewards for
    function calculateRewards(address account) public view returns (uint256) {
        if (stakes[account].amount == 0) return 0;

        Stake storage userStake = stakes[account];
        uint256 poolId = getUserPool(account);
        Pool storage pool = pools[poolId];

        uint256 timeStaked = block.timestamp - userStake.stakedAt;
        uint256 rewards = (userStake.amount * pool.rewardRate * timeStaked) / 1e18;

        return rewards;
    }

    /// @notice Get user's active pool (assumes user stakes in one pool at a time)
    function getUserPool(address user) public view returns (uint256) {
        // Simplified: assumes user only stakes in one pool
        // In production, you'd want to track multiple stakes per user
        return 1; // Default to first pool
    }

    /// @notice Get staking information for a user
    function getStakeInfo(address user) external view returns (
        uint256 stakedAmount,
        uint256 pendingRewards,
        uint256 stakedAt,
        uint256 lastClaim
    ) {
        Stake storage userStake = stakes[user];
        return (
            userStake.amount,
            calculateRewards(user),
            userStake.stakedAt,
            userStake.lastRewardClaim
        );
    }

    /// @notice Get pool information
    function getPoolInfo(uint256 poolId) external view returns (
        uint256 totalStaked,
        uint256 rewardRate,
        uint256 lockPeriod,
        uint256 totalRewardsDistributed,
        bool active
    ) {
        Pool storage pool = pools[poolId];
        return (
            pool.totalStaked,
            pool.rewardRate,
            pool.lockPeriod,
            pool.totalRewardsDistributed,
            pool.active
        );
    }

    /// @notice Pause staking operations (emergency)
    function pause() external onlyOwner {
        _pause();
    }

    /// @notice Unpause staking operations
    function unpause() external onlyOwner {
        _unpause();
    }

    /// @notice Update pool reward rate (owner only)
    function updatePoolRewardRate(uint256 poolId, uint256 newRate) external onlyOwner {
        require(poolId > 0 && poolId <= poolCount, "Invalid pool ID");
        require(pools[poolId].active, "Pool is not active");

        pools[poolId].rewardRate = newRate;
    }

    /// @notice Deactivate a pool (owner only)
    function deactivatePool(uint256 poolId) external onlyOwner {
        require(poolId > 0 && poolId <= poolCount, "Invalid pool ID");
        pools[poolId].active = false;
    }

    /// @notice Emergency withdraw (owner only, bypasses lock periods)
    function emergencyWithdraw(uint256 amount) external onlyOwner {
        require(amount <= qctToken.balanceOf(address(this)), "Insufficient contract balance");
        require(qctToken.transfer(owner(), amount), "Emergency transfer failed");
    }
}
