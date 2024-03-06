#![allow(deprecated)]

use std::{cell::RefCell, rc::Rc};

use crate::{
    pair_setup::PairSetup,
    wmoax_swap_setup::{WmoaxSwapSetup, WMOAX_TOKEN_ID},
};

use composable_tasks::{config::ConfigModule, ComposableTasksContract};
use dharitri_sc::{hex_literal, types::Address};
use dharitri_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use pair::safe_price::SafePriceModule;
use router::factory::{FactoryModule, PairTokens};

pub static FARMING_TOKEN_ID: &[&[u8]] = &[b"LPTOK-123456", b"LPTOK-654321"];
pub static TOKEN_IDS: &[&[u8]] = &[b"FIRST-123456", b"SECOND-123456", WMOAX_TOKEN_ID];
pub static LP_TOKEN_IDS: &[&[u8]] = &[FARMING_TOKEN_ID[0], FARMING_TOKEN_ID[1], b"LPWMOAX-123456"];

pub struct ComposableTasksSetup<
    PairBuilder,
    RouterBuilder,
    WmoaxSwapBuilder,
    ComposableTasksBuilder,
> where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    WmoaxSwapBuilder: 'static + Copy + Fn() -> dharitri_wmoax_swap_sc::ContractObj<DebugApi>,
    ComposableTasksBuilder: 'static + Copy + Fn() -> composable_tasks::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub owner: Address,
    pub first_user: Address,
    pub second_user: Address,
    pub pair_setups: Vec<PairSetup<PairBuilder>>,
    pub wmoax_swap_setup: WmoaxSwapSetup<WmoaxSwapBuilder>,
    pub router_wrapper: ContractObjWrapper<router::ContractObj<DebugApi>, RouterBuilder>,
    pub ct_wrapper:
        ContractObjWrapper<composable_tasks::ContractObj<DebugApi>, ComposableTasksBuilder>,
}

impl<PairBuilder, RouterBuilder, WmoaxSwapBuilder, ComposableTasksBuilder>
    ComposableTasksSetup<PairBuilder, RouterBuilder, WmoaxSwapBuilder, ComposableTasksBuilder>
where
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    RouterBuilder: 'static + Copy + Fn() -> router::ContractObj<DebugApi>,
    WmoaxSwapBuilder: 'static + Copy + Fn() -> dharitri_wmoax_swap_sc::ContractObj<DebugApi>,
    ComposableTasksBuilder: 'static + Copy + Fn() -> composable_tasks::ContractObj<DebugApi>,
{
    pub fn new(
        pair_builder: PairBuilder,
        router_builder: RouterBuilder,
        wmoax_swap_builder: WmoaxSwapBuilder,
        ct_builder: ComposableTasksBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0);
        let b_mock_new = BlockchainStateWrapper::new();
        let b_mock_ref = RefCell::new(b_mock_new);
        let b_mock_rc = Rc::new(b_mock_ref);
        let b_mock = b_mock_rc;

        let owner = b_mock.borrow_mut().create_user_account(&rust_zero);
        let first_user = Address::from(hex_literal::hex!(
            "75736572315F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock
            .borrow_mut()
            .create_user_account_fixed_address(&first_user, &rust_zero);

        // address:user2 from scenarios
        let second_user = Address::from(hex_literal::hex!(
            "75736572325F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F5F"
        ));
        b_mock
            .borrow_mut()
            .create_user_account_fixed_address(&second_user, &rust_zero);

        let router_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_zero,
            Some(&owner),
            router_builder,
            "router",
        );

        // setup composable tasks sc
        let ct_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            ct_builder,
            "composable tasks",
        );

        let wmoax_swap_setup = WmoaxSwapSetup::new(b_mock.clone(), wmoax_swap_builder, &owner);

        let mut first_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[1],
            LP_TOKEN_IDS[0],
        );
        let mut second_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[1],
        );
        let mut third_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[1],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[2],
        );

        let first_token_total_amount = 2_000_000_000u64;
        let second_token_total_amount = 3_000_000_000u64;
        let third_token_total_amount = 9_000_000_000u64;

        b_mock.borrow_mut().set_dct_balance(
            &owner,
            TOKEN_IDS[0],
            &rust_biguint!(first_token_total_amount),
        );
        b_mock.borrow_mut().set_dct_balance(
            &owner,
            TOKEN_IDS[1],
            &rust_biguint!(second_token_total_amount),
        );
        b_mock.borrow_mut().set_dct_balance(
            &owner,
            TOKEN_IDS[2],
            &rust_biguint!(third_token_total_amount),
        );

        let mut block_round: u64 = 1;
        b_mock.borrow_mut().set_block_round(block_round);

        // add initial liquidity
        first_pair_setup.add_liquidity(&owner, 1_000_000_000, 2_000_000_000);
        second_pair_setup.add_liquidity(&owner, 1_000_000_000, 1_000_000_000);
        third_pair_setup.add_liquidity(&owner, 1_000_000_000, 3_000_000_000);

        // setup price observations
        for _i in 1usize..=20 {
            block_round += 1;
            b_mock.borrow_mut().set_block_round(block_round);

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &first_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(2_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &second_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(1_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &third_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(3_000_000_000),
                        );
                    },
                )
                .assert_ok();
        }

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &router_wrapper, &rust_zero, |sc| {
                // sc.init(OptionalValue::None);
                sc.init_factory(Option::None);

                sc.pair_map().insert(
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[0]),
                        second_token_id: managed_token_id!(TOKEN_IDS[1]),
                    },
                    managed_address!(first_pair_setup.pair_wrapper.address_ref()),
                );

                sc.address_pair_map().insert(
                    managed_address!(first_pair_setup.pair_wrapper.address_ref()),
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[0]),
                        second_token_id: managed_token_id!(TOKEN_IDS[1]),
                    },
                );

                sc.pair_map().insert(
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[0]),
                        second_token_id: managed_token_id!(TOKEN_IDS[2]),
                    },
                    managed_address!(second_pair_setup.pair_wrapper.address_ref()),
                );

                sc.address_pair_map().insert(
                    managed_address!(second_pair_setup.pair_wrapper.address_ref()),
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[0]),
                        second_token_id: managed_token_id!(TOKEN_IDS[2]),
                    },
                );
                sc.pair_map().insert(
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[1]),
                        second_token_id: managed_token_id!(TOKEN_IDS[2]),
                    },
                    managed_address!(third_pair_setup.pair_wrapper.address_ref()),
                );

                sc.address_pair_map().insert(
                    managed_address!(third_pair_setup.pair_wrapper.address_ref()),
                    PairTokens {
                        first_token_id: managed_token_id!(TOKEN_IDS[1]),
                        second_token_id: managed_token_id!(TOKEN_IDS[2]),
                    },
                );
            })
            .assert_ok();

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &ct_wrapper, &rust_biguint!(0), |sc| {
                sc.init();

                let wmoax_swap_addr: &Address = wmoax_swap_setup.wmoax_swap_wrapper.address_ref();
                sc.set_wrap_moax_address(managed_address!(wmoax_swap_addr));
                sc.set_router_address(managed_address!(router_wrapper.address_ref()));
            })
            .assert_ok();

        let pair_setups = vec![first_pair_setup, second_pair_setup, third_pair_setup];

        ComposableTasksSetup {
            b_mock,
            owner,
            first_user,
            second_user,
            pair_setups,
            wmoax_swap_setup,
            router_wrapper,
            ct_wrapper,
        }
    }
}
