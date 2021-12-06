// SPDX-License-Identifier: BSD-3-Clause

pragma solidity >=0.7.0;

/// Interface to the precompiled contract on Shibuya/Shiden/Astar
/// Predeployed on the address 0x0000000000000000000000000000000000005001
interface DappsStaking {

    // Storage getters

    /// @dev Get current era.
    /// Selector: d7be3896
    /// @return The current era
    function current_era() external view returns (uint256);

    /// @dev Get Reward and Stakeed amount for given era.
    /// Selector: b9b70e8e
    /// @return The Reward and Staked amount for given era
    function era_reward_and_stake(uint32 era) external view returns (uint128, uint128);

    /// @dev Get Bonded amount for the staker (H160).
    /// Selector: fbfa941f
    /// @return Bonded amount for the staker
    function ledger(address staker) external view returns (uint128);

    /// @dev Get Developer's address for the provided H160 contract.
    /// Selector: b4fecd4f
    /// @return address as uint256, not as H160
    function registered_developer(address contract_id) external view returns (uint256);

    /// @dev Get amount staked and stakers for a contract per era
    /// Selector: 3b41e1f4
    /// The array of 2 stakers is encoded like: [staker1, amount1, staker2, amount2]
    /// @return amount staked on contract, amount of claimed rewards, array of stakers
    function contract_era_stake(address contract_id, uint32 era) external view returns (uint128, uint128, uint256[] memory);

    /// @dev Get Contract's address for the provided H160 accountId.
    /// Selector: 192fb256
    /// @return address as uint256, not as H160
    function registered_contract(address developer) external view returns (uint256);

    // Extrinsic calls

    /// @dev Register provided contract.
    /// Selector: 4420e486
    function register(address) external;

    /// @dev Stake provided amount on the contract.
    /// Selector: 52b73e41
    function bond_and_stake(address, uint128) external;

    /// @dev Start unbonding process and unstake balance from the contract.
    /// Selector: c7841dd2
    function unbond_and_unstake(address, uint128) external;

    /// @dev Withdraw all funds that have completed the unbonding process.
    /// Selector: 77a0fe02
    function withdraw_unbonded() external;

    /// @dev Claim contract's rewards.
    /// Selector: c13f4af7
    function claim(address, uint128) external;
}
