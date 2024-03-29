#![allow(deprecated)]

pub mod growth_program_setup;
use growth_program::{
    rewards::{
        claim::LockOption,
        common_rewards::{CommonRewardsModule, RewardsInfo},
        deposit::DepositRewardsModule,
        energy::EnergyModule,
    },
    DEFAULT_MIN_REWARDS_PERIOD,
};
use growth_program_setup::*;
use dharitri_sc_scenario::{
    managed_biguint, managed_token_id, managed_token_id_wrapped, rust_biguint, DebugApi,
};
use simple_lock::locked_token::LockedTokenAttributes;

#[test]
fn setup_test() {
    let _ = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );
}

#[test]
fn add_projects_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
}

#[test]
fn deposit_too_few_rewards_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();

    setup
        .b_mock
        .execute_dct_transfer(
            &setup.first_project_owner,
            &setup.gp_wrapper,
            FIRST_PROJ_TOKEN,
            0,
            &rust_biguint!(10),
            |sc| {
                sc.deposit_initial_rewards(1, 2, 28, managed_biguint!(1));
            },
        )
        .assert_user_error("Too few rewards");
}

#[test]
fn deposit_wrong_week_amount_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();

    let amount = StaticMethods::get_first_token_full_amount();

    setup
        .b_mock
        .execute_dct_transfer(
            &setup.first_project_owner,
            &setup.gp_wrapper,
            FIRST_PROJ_TOKEN,
            0,
            &amount,
            |sc| {
                sc.deposit_initial_rewards(1, 2, 5, managed_biguint!(1));
            },
        )
        .assert_user_error("Too few reward weeks");
}

#[test]
fn deposit_rewards_ok_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
    setup.deposit_rewards();

    setup
        .b_mock
        .execute_query(&setup.gp_wrapper, |sc| {
            let rewards_per_week_amount = sc.rewards_total_amount(1, 2).get();
            assert_eq!(
                rewards_per_week_amount,
                StaticMethods::get_first_token_full_amount_managed::<DebugApi>()
                    / DEFAULT_MIN_REWARDS_PERIOD as u32
            );
        })
        .assert_ok();
}

#[test]
fn deposit_additional_rewards_ok_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
    setup.deposit_rewards();

    let required_balance = StaticMethods::get_first_token_full_amount();
    setup.b_mock.set_dct_balance(
        &setup.first_project_owner,
        FIRST_PROJ_TOKEN,
        &required_balance,
    );

    setup
        .b_mock
        .execute_dct_transfer(
            &setup.first_project_owner,
            &setup.gp_wrapper,
            FIRST_PROJ_TOKEN,
            0,
            &required_balance,
            |sc| {
                sc.deposit_additional_rewards(1, 4, 4 + DEFAULT_MIN_REWARDS_PERIOD);
            },
        )
        .assert_ok();

    setup
        .b_mock
        .execute_query(&setup.gp_wrapper, |sc| {
            let rewards_week_two = sc.rewards_total_amount(1, 2).get();
            assert_eq!(
                rewards_week_two,
                StaticMethods::get_first_token_full_amount_managed::<DebugApi>()
                    / DEFAULT_MIN_REWARDS_PERIOD as u32
            );

            let rewards_week_four = sc.rewards_total_amount(1, 4).get();
            assert_eq!(
                rewards_week_four,
                StaticMethods::get_first_token_full_amount_managed::<DebugApi>() * 2u32
                    / DEFAULT_MIN_REWARDS_PERIOD as u32
            );

            let rewards_week_twenty_nine = sc.rewards_total_amount(1, 29).get();
            assert_eq!(
                rewards_week_twenty_nine,
                StaticMethods::get_first_token_full_amount_managed::<DebugApi>()
                    / DEFAULT_MIN_REWARDS_PERIOD as u32
            );

            let rewards_info = sc.rewards_info(1).get();
            let expected_rewards_info = RewardsInfo {
                reward_token_id: managed_token_id!(FIRST_PROJ_TOKEN),
                undistributed_rewards: managed_biguint!(0),
                start_week: 2,
                end_week: 30,
            };
            assert_eq!(rewards_info, expected_rewards_info);
        })
        .assert_ok();
}

#[test]
fn claim_ok_first_week_unlocked_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
    setup.deposit_rewards();

    // advance to week 2
    setup.advance_week();

    let sig_first_user_week_2 = hex_literal::hex!("7f2a309ed332a516c3dff6634bbf9ce42ea57d6cb5acac606010ae47e1180db4f7ecc70d79998eec961bbbc353c49e469233b6c915179795cd07d903969ce905");
    setup
        .claim(
            &setup.first_user_addr.clone(),
            1,
            LockOption::None,
            0,
            &sig_first_user_week_2,
        )
        .assert_ok();

    setup
        .b_mock
        .execute_query(&setup.gp_wrapper, |sc| {
            let total_energy = sc.total_energy_for_week(1, 2).get();
            assert_eq!(total_energy, managed_biguint!(192307692307690));

            let interested_energy = sc.interested_energy_rewards_claimers(1, 2).get();
            assert_eq!(interested_energy, managed_biguint!(348000) / 4u32); // 25% for full unlocked
        })
        .assert_ok();

    setup.b_mock.check_dct_balance(
        &setup.first_user_addr,
        FIRST_PROJ_TOKEN,
        &rust_biguint!(17400000000000208),
    );

    // first user try claim again
    setup
        .claim(
            &setup.first_user_addr.clone(),
            1,
            LockOption::None,
            0,
            &sig_first_user_week_2,
        )
        .assert_user_error("Already claimed");
}

#[test]
fn claim_ok_first_week_locked_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
    setup.deposit_rewards();

    // advance to week 2
    setup.advance_week();

    let sig_first_user_week_2 = hex_literal::hex!("7f2a309ed332a516c3dff6634bbf9ce42ea57d6cb5acac606010ae47e1180db4f7ecc70d79998eec961bbbc353c49e469233b6c915179795cd07d903969ce905");
    setup
        .claim(
            &setup.first_user_addr.clone(),
            1,
            LockOption::OneWeek,
            0,
            &sig_first_user_week_2,
        )
        .assert_ok();

    setup
        .b_mock
        .execute_query(&setup.gp_wrapper, |sc| {
            let total_energy = sc.total_energy_for_week(1, 2).get();
            assert_eq!(total_energy, managed_biguint!(192307692307690));

            let interested_energy = sc.interested_energy_rewards_claimers(1, 2).get();
            assert_eq!(interested_energy, managed_biguint!(348000) / 2u32); // 50% for one week lock
        })
        .assert_ok();

    DebugApi::dummy();
    setup.b_mock.check_nft_balance(
        &setup.first_user_addr,
        LOCKED_TOKEN_ID,
        1,
        &rust_biguint!(34800000000000417),
        Some(&LockedTokenAttributes::<DebugApi> {
            original_token_id: managed_token_id_wrapped!(FIRST_PROJ_TOKEN),
            original_token_nonce: 0,
            unlock_epoch: 19,
        }),
    );
}

#[test]
fn claim_too_many_rewards_test() {
    let mut setup = GrowthProgramSetup::new(
        growth_program::contract_obj,
        pair_mock::contract_obj,
        router_mock::contract_obj,
        simple_lock::contract_obj,
        energy_factory::contract_obj,
    );

    setup.add_projects();
    setup.deposit_rewards();

    // advance to week 2
    setup.advance_week();

    let sig_first_user_week_2 = hex_literal::hex!("7f2a309ed332a516c3dff6634bbf9ce42ea57d6cb5acac606010ae47e1180db4f7ecc70d79998eec961bbbc353c49e469233b6c915179795cd07d903969ce905");
    setup
        .claim(
            &setup.first_user_addr.clone(),
            1,
            LockOption::None,
            17400000000000209,
            &sig_first_user_week_2,
        )
        .assert_user_error("Too few rewards");
}
