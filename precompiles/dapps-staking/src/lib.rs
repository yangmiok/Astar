//! Astar dApps staking interface.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use fp_evm::{Context, ExitError, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Get;
use pallet_evm::{AddressMapping, GasWeightMapping, Precompile};
use sp_core::H160;
use sp_runtime::traits::{SaturatedConversion, Zero};
use sp_std::convert::TryInto;
use sp_std::{marker::PhantomData, vec::Vec};
extern crate alloc;

mod utils;
pub use utils::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub struct DappsStakingWrapper<R>(PhantomData<R>);

impl<R> DappsStakingWrapper<R>
where
    R: pallet_evm::Config + pallet_dapps_staking::Config,
    R::Call: From<pallet_dapps_staking::Call<R>>,
{
    /// Fetch current era from CurrentEra storage map
    fn current_era() -> Result<PrecompileOutput, ExitError> {
        let current_era = pallet_dapps_staking::CurrentEra::<R>::get();
        let gas_used = R::GasWeightMapping::weight_to_gas(R::DbWeight::get().read);
        println!(
            "--- precompile DappsStaking response: current_era era={:?} gas_used={:?}",
            current_era, gas_used
        );

        let output = utils::argument_from_u32(current_era);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gas_used,
            output,
            logs: Default::default(),
        })
    }

    /// Fetch reward and stake from EraRewardsAndStakes storage map
    fn era_reward_and_stake(input: EvmInArg) -> Result<PrecompileOutput, ExitError> {
        input.expecting_arguments(1).map_err(|e| exit_error(e))?;
        let era = input.to_u256(1).low_u32();
        let reward_and_stake = pallet_dapps_staking::EraRewardsAndStakes::<R>::get(era);
        let (reward, staked) = if let Some(r) = reward_and_stake {
            (r.rewards, r.staked)
        } else {
            (Zero::zero(), Zero::zero())
        };
        let gas_used = R::GasWeightMapping::weight_to_gas(R::DbWeight::get().read);
        println!(
            "--- precompile DappsStaking response: era={:?}, reward={:?} staked ={:?} gas_used={:?}",
            era, reward, staked, gas_used
        );

        let reward = TryInto::<u128>::try_into(reward).unwrap_or(0);
        let mut output = utils::argument_from_u128(reward);

        let staked = TryInto::<u128>::try_into(staked).unwrap_or(0);
        let mut staked_vec = utils::argument_from_u128(staked);
        output.append(&mut staked_vec);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gas_used,
            output,
            logs: Default::default(),
        })
    }

    /// Fetch registered contract from RegisteredDevelopers storage map
    fn registered_contract(input: EvmInArg) -> Result<PrecompileOutput, ExitError> {
        println!("--- precompile registered_contract() {:?}", input.len());
        let developer_h160 = input.to_h160(1);
        let developer = R::AddressMapping::into_account_id(developer_h160);
        println!("--- precompile developer_h160 {:?}", developer_h160);
        println!("--- precompile developer public key {:?}", developer);

        let smart_contract = pallet_dapps_staking::RegisteredDevelopers::<R>::get(&developer);
        let gas_used = R::GasWeightMapping::weight_to_gas(R::DbWeight::get().read);

        println!(
            "--- precompile developer {:?}, contract {:?}",
            developer, smart_contract
        );
        let output = argument_from_h160_vec(smart_contract.unwrap_or_default().encode());

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gas_used,
            output,
            logs: Default::default(),
        })
    }

    /// Fetch amount staked and stakers for a contract per era
    fn contract_era_stake(input: EvmInArg) -> Result<PrecompileOutput, ExitError> {
        println!("--- precompile contract_era_stake() {:?}", input.len());
        input.expecting_arguments(2).map_err(|e| exit_error(e))?;

        // parse input parameters for pallet-dapps-staking call
        let contract_h160 = input.to_h160(1);
        let contract_id = Self::decode_smart_contract(contract_h160)?;
        let era = input.to_u256(2).low_u32();
        println!(
            "--- precompile contract_id={:?}, era={:?}",
            contract_id, era
        );

        // call pallet-dapps-staking
        let staking_info = pallet_dapps_staking::Pallet::<R>::staking_info(&contract_id, era);
        let gas_used = R::GasWeightMapping::weight_to_gas(R::DbWeight::get().read);

        // encode output with total and rewards
        let total = TryInto::<u128>::try_into(staking_info.total).unwrap_or(0);
        let mut output = utils::argument_from_u128(total);
        println!("output-- {:x?}", output);
        let claimed_rewards = TryInto::<u128>::try_into(staking_info.claimed_rewards).unwrap_or(0);
        let mut claimed_rewards_vec = utils::argument_from_u128(claimed_rewards);
        println!("output-- {:x?}", claimed_rewards_vec);
        output.append(&mut claimed_rewards_vec);

        // Encode number of elements of the array
        // since we encode array uint256 as [staker1, amount1, staker2, amount2],
        // the number of elements is double the size of entries in the stakers map
        // see https://docs.soliditylang.org/en/v0.8.10/abi-spec.html#mapping-solidity-to-abi-types
        let mut offset = utils::argument_from_u32(0x60_u32);
        println!("output-- {:?}", offset);
        output.append(&mut offset);
        let mut num_elements = utils::argument_from_u32((staking_info.stakers.len() * 2) as u32);
        println!("output-- {:?}", num_elements);
        output.append(&mut num_elements);

        // encode output for all pairs of staker:amount
        for staker_amount_pair in staking_info.stakers.clone() {
            let mut address = Self::argument_from_account_id(&staker_amount_pair.0);
            let mut amount =
                argument_from_u128(TryInto::<u128>::try_into(staker_amount_pair.1).unwrap_or(0));
            println!("output-- {:?}", address);
            output.append(&mut address);
            println!("output-- {:?}", amount);
            output.append(&mut amount);
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gas_used,
            output,
            logs: Default::default(),
        })
    }

    pub fn argument_from_account_id(account: &R::AccountId) -> Vec<u8> {
        let mut account_encoded = R::AccountId::encode(account);
        let encoded_len = account_encoded.len();
        let mut buffer = sp_std::vec![0; ARG_SIZE_BYTES - encoded_len];
        buffer.append(&mut account_encoded);
        buffer
    }

    /// Register contract with the dapp-staking pallet
    fn register(input: EvmInArg) -> Result<R::Call, ExitError> {
        println!("--- precompile register() {:?}", input.len());
        input.expecting_arguments(1).map_err(|e| exit_error(e))?;
        // parse contract's address
        let contract_h160 = input.to_h160(1);
        // println!("contract_h160 {:?}", contract_h160);

        let contract_id = Self::decode_smart_contract(contract_h160)?;

        Ok(pallet_dapps_staking::Call::<R>::register { contract_id }.into())
    }

    /// Lock up and stake balance of the origin account.
    fn bond_and_stake(input: EvmInArg) -> Result<R::Call, ExitError> {
        input.expecting_arguments(2).map_err(|e| exit_error(e))?;

        // parse contract's address
        let contract_h160 = input.to_h160(1);
        // println!("contract_h160 {:?}", contract_h160);
        let contract_id = Self::decode_smart_contract(contract_h160)?;

        // parse balance to be staked
        let value = input.to_u256(2).low_u128().saturated_into();
        println!("--- precompile bond value {:?}", value);

        Ok(pallet_dapps_staking::Call::<R>::bond_and_stake { contract_id, value }.into())
    }
    /// Helper method to decode type SmartContract enum
    fn decode_smart_contract(
        contract_h160: H160,
    ) -> Result<<R as pallet_dapps_staking::Config>::SmartContract, ExitError> {
        // Encode contract address to fit SmartContract enum.
        // Since the SmartContract enum type can't be accessed from this pecompile,
        // use locally defined enum clone (see Contract enum)
        let contract_enum_encoded = Contract::<H160>::Evm(contract_h160).encode();

        // encoded enum will add one byte before the contract's address
        // therefore we need to decode len(H160) + 1 byte = 21
        let smart_contract = <R as pallet_dapps_staking::Config>::SmartContract::decode(
            &mut &contract_enum_encoded[..21],
        )
        .map_err(|_| exit_error("Error while decoding SmartContract"))?;
        println!("--- precompile smart_contract decoded {:?}", smart_contract);

        Ok(smart_contract)
    }
}

impl<R> Precompile for DappsStakingWrapper<R>
where
    R: pallet_evm::Config + pallet_dapps_staking::Config + frame_system::Config,
    <R as frame_system::Config>::Call: From<pallet_dapps_staking::Call<R>>
        + Dispatchable<PostInfo = PostDispatchInfo>
        + GetDispatchInfo,
    <<R as frame_system::Config>::Call as Dispatchable>::Origin:
        From<Option<<R as frame_system::Config>::AccountId>>,
{
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> Result<PrecompileOutput, ExitError> {
        println!(
            "*\n*************** DappsStakingWrapper execute(), len={:?} ************************",
            input.len(),
        );
        println!("--- precompile context.caller={:?}", context.caller);

        let input = EvmInArg::new(&input);
        let selector = input.selector().map_err(|e| exit_error(e))?;
        let call = match selector {
            // storage getters
            [0xd7, 0xbe, 0x38, 0x96] => return Self::current_era(),
            [0xb9, 0xb7, 0x0e, 0x8e] => return Self::era_reward_and_stake(input),
            [0x19, 0x2f, 0xb2, 0x56] => return Self::registered_contract(input),
            [0x3b, 0x41, 0xe1, 0xf4] => return Self::contract_era_stake(input),

            // extrinsic calls
            [0x44, 0x20, 0xe4, 0x86] => Self::register(input)?,
            [0x52, 0xb7, 0x3e, 0x41] => Self::bond_and_stake(input)?,
            _ => {
                println!("!!!!!!!!!!! ERROR selector, selector={:x?}", selector);
                return Err(ExitError::Other("No method at given selector".into()));
            }
        };

        let info = call.get_dispatch_info();
        println!("--- precompile info ={:?}", info);
        if let Some(gas_limit) = target_gas {
            let required_gas = R::GasWeightMapping::weight_to_gas(info.weight);
            println!(
                "--- precompile required_gas={:?}, gas_limit={:?}",
                required_gas, gas_limit
            );
            if required_gas > gas_limit {
                println!("--- precompile !!!!!!! OutOfGas !!!! ");
                return Err(ExitError::OutOfGas);
            }
        }

        let origin = R::AddressMapping::into_account_id(context.caller);
        println!("--> precompile origin = {}", origin);
        let post_info = call.dispatch(Some(origin).into()).map_err(|e| {
            println!("!!!!!!!!!!! ERROR={:x?}", e.error);
            ExitError::Other("Method call via EVM failed".into())
        })?;
        println!("--> precompile post_info ={:?}", post_info);

        let gas_used =
            R::GasWeightMapping::weight_to_gas(post_info.actual_weight.unwrap_or(info.weight));
        println!("--> precompile gas_used ={:?}", gas_used);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost: gas_used,
            output: Default::default(),
            logs: Default::default(),
        })
    }
}
