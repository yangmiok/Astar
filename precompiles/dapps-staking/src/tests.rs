use crate::mock::{
    advance_to_era, default_context, evm_call, exit_error, initialize_first_block,
    precompile_address, Call, EraIndex, ExternalityBuilder, Origin, Precompiles, TestAccount, AST,
    BLOCK_REWARD, UNBONDING_PERIOD, *,
};
use fp_evm::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::{ExitSucceed, PrecompileSet};
use sha3::{Digest, Keccak256};
use sp_core::H160;
use sp_runtime::Perbill;
use std::collections::BTreeMap;

use crate::utils;

#[test]
fn selector_out_of_bounds_nok() {
    ExternalityBuilder::default().build().execute_with(|| {
        initialize_first_block();

        // Use 3 bytes selector. 4 bytes are needed
        let selector_nok = vec![0x01, 0x02, 0x03];

        let expected = Some(Err(exit_error("Selector too short")));

        assert_eq!(
            Precompiles::execute(
                precompile_address(),
                &selector_nok,
                None,
                &default_context(),
            ),
            expected
        );
    });
}
#[test]
fn selector_unknown_nok() {
    ExternalityBuilder::default().build().execute_with(|| {
        initialize_first_block();

        // We use 3 bytes selector. 4 byts are needed
        let selector_nok = vec![0x01, 0x02, 0x03, 0x04];

        let expected = Some(Err(exit_error("No method at given selector")));

        assert_eq!(
            Precompiles::execute(
                precompile_address(),
                &selector_nok,
                None,
                &default_context(),
            ),
            expected
        );
    });
}

#[test]
fn current_era_is_ok() {
    ExternalityBuilder::default().build().execute_with(|| {
        initialize_first_block();

        let selector = &Keccak256::digest(b"current_era()")[0..4];
        let mut expected_era = vec![0u8; 32];
        expected_era[31] = 1;

        let expected = Some(Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: expected_era.clone(),
            cost: Default::default(),
            logs: Default::default(),
        }));

        assert_eq!(
            Precompiles::execute(precompile_address(), &selector, None, &default_context()),
            expected
        );

        // advance to era 5 and check output
        expected_era[31] = 5;
        advance_to_era(5);
        let expected = Some(Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: expected_era,
            cost: Default::default(),
            logs: Default::default(),
        }));
        assert_eq!(
            Precompiles::execute(precompile_address(), &selector, None, &default_context()),
            expected
        );
    });
}

#[test]
fn era_reward_and_stake_is_ok() {
    ExternalityBuilder::default().build().execute_with(|| {
        initialize_first_block();

        // build input for the call
        let selector = &Keccak256::digest(b"era_reward_and_stake(uint32)")[0..4];
        let mut input_data = Vec::<u8>::from([0u8; 36]);
        input_data[0..4].copy_from_slice(&selector);
        let era = [0u8; 32];
        input_data[4..36].copy_from_slice(&era);

        // build expected outcome
        let reward = BLOCK_REWARD;
        let mut expected_output = utils::argument_from_u128(reward);
        let staked = 0;
        let mut staked_vec = utils::argument_from_u128(staked);
        expected_output.append(&mut staked_vec);
        let expected = Some(Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: expected_output,
            cost: Default::default(),
            logs: Default::default(),
        }));

        // verify that argument check is done in era_reward_and_stake()
        assert_eq!(
            Precompiles::execute(precompile_address(), &selector, None, &default_context()),
            Some(Err(exit_error("Too few arguments")))
        );

        // execute and verify era_reward_and_stake() query
        assert_eq!(
            Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
            expected
        );
    });
}

#[test]
fn era_reward_and_stake_too_many_arguments_nok() {
    ExternalityBuilder::default().build().execute_with(|| {
        initialize_first_block();

        // build input for the call
        let selector = &Keccak256::digest(b"era_reward_and_stake(uint32)")[0..4];
        let mut input_data = Vec::<u8>::from([0u8; 37]);
        input_data[0..4].copy_from_slice(&selector);
        let era = [0u8; 33];
        input_data[4..37].copy_from_slice(&era);

        assert_eq!(
            Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
            Some(Err(exit_error("Too many arguments")))
        )
    });
}

#[test]
fn register_is_ok() {
    ExternalityBuilder::default()
        .with_balances(vec![(TestAccount::Alex, 200 * AST)])
        .build()
        .execute_with(|| {
            initialize_first_block();
            let developer = TestAccount::Alex;
            let contract_array = H160::repeat_byte(0x09).to_fixed_bytes();
            register_and_verify(developer, contract_array.clone());
        });
}

#[test]
fn bond_and_stake_is_ok() {
    ExternalityBuilder::default()
        .with_balances(vec![
            (TestAccount::Alex, 200 * AST),
            (TestAccount::Bobo, 200 * AST),
            (TestAccount::Dino, 100 * AST),
        ])
        .build()
        .execute_with(|| {
            initialize_first_block();

            // register new contract by Alex
            let developer = TestAccount::Alex;
            let contract_array = H160::repeat_byte(0x09).to_fixed_bytes();
            register_and_verify(developer, contract_array.clone());

            let amount_staked_bobo = 100 * AST;
            bond_stake_and_verify(TestAccount::Bobo, contract_array, amount_staked_bobo);

            let amount_staked_dino = 50 * AST;
            bond_stake_and_verify(TestAccount::Dino, contract_array, amount_staked_dino);

            let mut stakers_map = BTreeMap::new();
            stakers_map.insert(TestAccount::Bobo, amount_staked_bobo);
            stakers_map.insert(TestAccount::Dino, amount_staked_dino);
            staking_info_verify(
                contract_array,
                amount_staked_bobo + amount_staked_dino,
                1,
                stakers_map,
            );
        });
}

#[test]
fn unbond_and_unstake_is_ok() {
    ExternalityBuilder::default()
        .with_balances(vec![
            (TestAccount::Alex, 200 * AST),
            (TestAccount::Bobo, 200 * AST),
            (TestAccount::Dino, 100 * AST),
        ])
        .build()
        .execute_with(|| {
            initialize_first_block();

            // register new contract by Alex
            let developer = TestAccount::Alex;
            let contract_array = H160::repeat_byte(0x09).to_fixed_bytes();
            register_and_verify(developer, contract_array.clone());

            let amount_staked_bobo = 100 * AST;
            bond_stake_and_verify(TestAccount::Bobo, contract_array, amount_staked_bobo);
            let amount_staked_dino = 50 * AST;
            bond_stake_and_verify(TestAccount::Dino, contract_array, amount_staked_dino);

            // Bobo unstakes all
            let era = 2;
            advance_to_era(era);
            unbond_unstake_and_verify(TestAccount::Bobo, contract_array, amount_staked_bobo);

            let mut stakers_map = BTreeMap::new();
            stakers_map.insert(TestAccount::Dino, amount_staked_dino);
            staking_info_verify(contract_array, amount_staked_dino, era, stakers_map);

            // withdraw unbonded funds
            advance_to_era(era + UNBONDING_PERIOD + 1);
            withdraw_unbonded_verify(TestAccount::Bobo);
        });
}

#[test]
fn claim_is_ok() {
    ExternalityBuilder::default()
        .with_balances(vec![
            (TestAccount::Alex, 200 * AST),
            (TestAccount::Bobo, 200 * AST),
            (TestAccount::Dino, 200 * AST),
        ])
        .build()
        .execute_with(|| {
            initialize_first_block();

            // register new contract by Alex
            let developer = TestAccount::Alex;
            let contract_array = H160::repeat_byte(0x09).to_fixed_bytes();
            register_and_verify(developer, contract_array.clone());

            let stake_amount_total = 300 * AST;
            let ratio_bobo = Perbill::from_rational(3u32, 5u32);
            let ratio_dino = Perbill::from_rational(2u32, 5u32);
            let amount_staked_bobo = ratio_bobo * stake_amount_total;
            bond_stake_and_verify(TestAccount::Bobo, contract_array, amount_staked_bobo);

            let amount_staked_dino = ratio_dino * stake_amount_total;
            bond_stake_and_verify(TestAccount::Dino, contract_array, amount_staked_dino);

            // advance era and claim reward
            let era = 5;
            advance_to_era(era);
            claim_and_verify(contract_array, era - 1);

            //check that the reward is payed out to the stakers and the developer
            let developer_reward = Perbill::from_percent(DEVELOPER_REWARD_PERCENTAGE)
                * BLOCK_REWARD
                * BLOCKS_PER_ERA as u128
                - REGISTER_DEPOSIT;
            let stakers_reward = Perbill::from_percent(100 - DEVELOPER_REWARD_PERCENTAGE)
                * BLOCK_REWARD
                * BLOCKS_PER_ERA as u128;
            let bobo_reward = ratio_bobo * stakers_reward;
            let dino_reward = ratio_dino * stakers_reward;
            assert_eq!(
                <TestRuntime as pallet_evm::Config>::Currency::free_balance(TestAccount::Alex),
                (200 * AST) + developer_reward
            );
            assert_eq!(
                <TestRuntime as pallet_evm::Config>::Currency::free_balance(TestAccount::Bobo),
                (200 * AST) + bobo_reward
            );
            assert_eq!(
                <TestRuntime as pallet_evm::Config>::Currency::free_balance(TestAccount::Dino),
                (200 * AST) + dino_reward
            );
        });
}

// ****************************************************************************************************
// Helper functions
// ****************************************************************************************************

/// helper function to register and verify if registration is valid
fn register_and_verify(developer: TestAccount, contract_array: [u8; 20]) {
    let selector = &Keccak256::digest(b"register(address)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 36]);
    input_data[0..4].copy_from_slice(&selector);
    input_data[16..36].copy_from_slice(&contract_array);

    // verify that argument check is done in register()
    assert_ok!(Call::Evm(evm_call(developer.clone(), selector.to_vec())).dispatch(Origin::root()));

    // call register()
    assert_ok!(Call::Evm(evm_call(developer.clone(), input_data)).dispatch(Origin::root()));

    // check the storage after the register
    registered_contract_verify(developer.clone(), contract_array);
    registered_developer_verify(developer, contract_array);

    // check_register_event(developer, contract_h160);
}

/// helper function to read storage with registered contracts
fn registered_contract_verify(developer: TestAccount, contract_array_h160: [u8; 20]) {
    println!(
        "--- registered_contract_verify contract_array_h160({:?}) {:?}",
        contract_array_h160.len(),
        contract_array_h160
    );

    let selector = &Keccak256::digest(b"registered_contract(address)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 36]);
    input_data[0..4].copy_from_slice(&selector);

    let developer_arg = utils::argument_from_h160(developer.to_h160());
    let contract = utils::argument_from_h160_array(contract_array_h160);

    input_data[4..36].copy_from_slice(&developer_arg);

    let expected = Some(Ok(PrecompileOutput {
        exit_status: ExitSucceed::Returned,
        output: contract,
        cost: Default::default(),
        logs: Default::default(),
    }));

    // verify that argument check is done in registered_contract
    assert_eq!(
        Precompiles::execute(precompile_address(), &selector, None, &default_context()),
        Some(Err(exit_error("Too few arguments")))
    );

    assert_eq!(
        Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
        expected
    );
}

/// helper function to read ledger storage item
fn ledger_verify(staker: TestAccount, amount: u128) {
    println!("--- ledger account={:?} amount={:?}", staker, amount);

    let selector = &Keccak256::digest(b"ledger(address)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 36]);
    input_data[0..4].copy_from_slice(&selector);

    let staker_arg = utils::argument_from_h160(staker.to_h160());

    input_data[4..36].copy_from_slice(&staker_arg);

    let expected = Some(Ok(PrecompileOutput {
        exit_status: ExitSucceed::Returned,
        output: utils::argument_from_u128(amount),
        cost: Default::default(),
        logs: Default::default(),
    }));

    // verify that argument check is done in registered_contract
    assert_eq!(
        Precompiles::execute(precompile_address(), &selector, None, &default_context()),
        Some(Err(exit_error("Too few arguments")))
    );

    assert_eq!(
        Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
        expected
    );
}

/// helper function to read storage with registered contracts
fn registered_developer_verify(developer: TestAccount, contract_array_h160: [u8; 20]) {
    println!(
        "--- registered_developer_verify contract_array_h160({:?}) {:?}",
        contract_array_h160.len(),
        contract_array_h160
    );

    let selector = &Keccak256::digest(b"registered_developer(address)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 36]);
    input_data[0..4].copy_from_slice(&selector);

    let contract = utils::argument_from_h160_array(contract_array_h160);

    input_data[4..36].copy_from_slice(&contract);

    let expected = Some(Ok(PrecompileOutput {
        exit_status: ExitSucceed::Returned,
        output: developer.to_argument(),
        cost: Default::default(),
        logs: Default::default(),
    }));

    // verify that argument check is done in registered_developer
    assert_eq!(
        Precompiles::execute(precompile_address(), &selector, None, &default_context()),
        Some(Err(exit_error("Too few arguments")))
    );

    assert_eq!(
        Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
        expected
    );
}

/// helper function to bond, stake and verify if resulet is OK
fn bond_stake_and_verify(staker: TestAccount, contract_array: [u8; 20], amount: u128) {
    println!(
        "--- bond_stake_and_verify contract_array({:?}) {:?}",
        contract_array.len(),
        contract_array
    );
    let selector = &Keccak256::digest(b"bond_and_stake(address,uint128)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 68]);
    input_data[0..4].copy_from_slice(&selector);
    input_data[16..36].copy_from_slice(&contract_array);
    let staking_amount = amount.to_be_bytes();
    input_data[(68 - staking_amount.len())..68].copy_from_slice(&staking_amount);

    // verify that argument check is done in bond_and_stake()
    assert_ok!(Call::Evm(evm_call(staker.clone(), selector.to_vec())).dispatch(Origin::root()));

    // call bond_and_stake()
    assert_ok!(Call::Evm(evm_call(staker.clone(), input_data)).dispatch(Origin::root()));

    ledger_verify(staker, amount);
}

/// helper function to unbond, unstake and verify if resulet is OK
fn unbond_unstake_and_verify(staker: TestAccount, contract_array: [u8; 20], amount: u128) {
    println!(
        "--- unbond_unstake_and_verify contract_array({:?}) {:?}",
        contract_array.len(),
        contract_array
    );
    let selector = &Keccak256::digest(b"unbond_and_unstake(address,uint128)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 68]);
    input_data[0..4].copy_from_slice(&selector);
    input_data[16..36].copy_from_slice(&contract_array);
    let staking_amount = amount.to_be_bytes();
    input_data[(68 - staking_amount.len())..68].copy_from_slice(&staking_amount);

    // verify that argument check is done in unbond_unstake()
    assert_ok!(Call::Evm(evm_call(staker.clone(), selector.to_vec())).dispatch(Origin::root()));

    // call unbond_and_unstake()
    assert_ok!(Call::Evm(evm_call(staker, input_data)).dispatch(Origin::root()));
}

/// helper function to withdraw unstaked funds and verify if resulet is OK
fn withdraw_unbonded_verify(staker: TestAccount) {
    println!("--- withdraw_unbonded_verify");
    let selector = &Keccak256::digest(b"withdraw_unbonded()")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 4]);
    input_data[0..4].copy_from_slice(&selector);

    // call unbond_and_unstake(). Check usable_balance before and after the call
    assert_ne!(
        <TestRuntime as pallet_evm::Config>::Currency::free_balance(&staker),
        <TestRuntime as pallet_evm::Config>::Currency::usable_balance(&staker)
    );
    assert_ok!(Call::Evm(evm_call(staker.clone(), input_data)).dispatch(Origin::root()));
    assert_eq!(
        <TestRuntime as pallet_evm::Config>::Currency::free_balance(&staker),
        <TestRuntime as pallet_evm::Config>::Currency::usable_balance(&staker)
    );
}

/// helper function to bond, stake and verify if resulet is OK
fn claim_and_verify(contract_array: [u8; 20], era: EraIndex) {
    println!(
        "--- claim_and_verify contract_array({:?}) {:?}, era {:?}",
        contract_array.len(),
        contract_array,
        era
    );
    let staker = TestAccount::Bobo;
    let selector = &Keccak256::digest(b"claim(address,uint128)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 68]);
    input_data[0..4].copy_from_slice(&selector);
    input_data[16..36].copy_from_slice(&contract_array);
    let era_array = era.to_be_bytes();
    input_data[(68 - era_array.len())..68].copy_from_slice(&era_array);

    // verify that argument check is done in claim()
    assert_ok!(Call::Evm(evm_call(staker.clone(), selector.to_vec())).dispatch(Origin::root()));

    // call bond_and_stake()
    assert_ok!(Call::Evm(evm_call(staker.clone(), input_data)).dispatch(Origin::root()));
}

/// helper function to check if bonding was successful
fn staking_info_verify(
    contract_array: [u8; 20],
    amount: u128,
    era: EraIndex,
    stakers_map: BTreeMap<TestAccount, u128>,
) {
    // prepare input to read staked amount on the contract
    let selector = &Keccak256::digest(b"contract_era_stake(address,uint32)")[0..4];
    let mut input_data = Vec::<u8>::from([0u8; 68]);
    input_data[0..4].copy_from_slice(&selector);
    input_data[16..36].copy_from_slice(&contract_array);
    let mut era_vec = Vec::<u8>::from([0u8; 32]);
    era_vec[31] = era as u8;
    input_data[(68 - era_vec.len())..68].copy_from_slice(&era_vec);

    // Compose expected outcome: 1. add total and rewards
    let total = amount;
    let claimed_reward = 0;
    let mut expected_output = utils::argument_from_u128(total);
    let mut claimed_reward_vec = utils::argument_from_u128(claimed_reward);
    expected_output.append(&mut claimed_reward_vec);

    // Compose expected outcome: 2. add number of elements of the array
    let mut offset = utils::argument_from_u32(0x60_u32);
    expected_output.append(&mut offset);
    let mut num_elements = utils::argument_from_u32((stakers_map.len() * 2) as u32);
    expected_output.append(&mut num_elements);

    // Compose expected outcome: 3. add stakers map as array [staker1, amount1, staker2, amount2]
    for staker_amount in stakers_map {
        println!("staker_amount_pair {:?}", staker_amount);
        let mut address = staker_amount.0.to_argument();
        let mut amount = utils::argument_from_u128(staker_amount.1);
        println!("address {:?}, \namount {:?}", address, amount);
        expected_output.append(&mut address);
        expected_output.append(&mut amount);
    }

    let expected = Some(Ok(PrecompileOutput {
        exit_status: ExitSucceed::Returned,
        output: expected_output,
        cost: Default::default(),
        logs: Default::default(),
    }));

    // verify that argument check is done in contract_era_stake
    assert_eq!(
        Precompiles::execute(precompile_address(), &selector, None, &default_context()),
        Some(Err(exit_error("Too few arguments")))
    );

    // execute and verify contract_era_stake() query
    assert_eq!(
        Precompiles::execute(precompile_address(), &input_data, None, &default_context()),
        expected
    );
}

// Check Register event
// pub fn check_register_event(developer: H160, contract_id: H160) {
//     System::assert_last_event(Event::DappsStaking(
//         <TestRuntime as pallet_dapps_staking::Config>::Event::NewContract(
//         developer,
//         contract_id,
//     )));
// }
