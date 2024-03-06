#![allow(deprecated)]

use std::cell::RefCell;
use std::rc::Rc;

use dharitri_sc::types::{Address, DctLocalRole};
use dharitri_sc_scenario::{
    managed_token_id, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use dharitri_wmoax_swap_sc::MoaxDctSwap;

pub static WMOAX_TOKEN_ID: &[u8] = b"WMOAX-123456";
pub static MOAX_TOKEN_ID: &[u8] = b"MOAX";

pub struct WmoaxSwapSetup<WmoaxSwapObjBuilder>
where
    WmoaxSwapObjBuilder: 'static + Copy + Fn() -> dharitri_wmoax_swap_sc::ContractObj<DebugApi>,
{
    pub b_mock: Rc<RefCell<BlockchainStateWrapper>>,
    pub wmoax_swap_wrapper:
        ContractObjWrapper<dharitri_wmoax_swap_sc::ContractObj<DebugApi>, WmoaxSwapObjBuilder>,
}

impl<WmoaxSwapObjBuilder> WmoaxSwapSetup<WmoaxSwapObjBuilder>
where
    WmoaxSwapObjBuilder: 'static + Copy + Fn() -> dharitri_wmoax_swap_sc::ContractObj<DebugApi>,
{
    pub fn new(
        b_mock: Rc<RefCell<BlockchainStateWrapper>>,
        wmoax_swap_builder: WmoaxSwapObjBuilder,
        owner: &Address,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let wmoax_swap_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_zero,
            Some(owner),
            wmoax_swap_builder,
            "wmoax swap",
        );

        b_mock
            .borrow_mut()
            .execute_tx(owner, &wmoax_swap_wrapper, &rust_zero, |sc| {
                sc.init(managed_token_id!(WMOAX_TOKEN_ID));
            })
            .assert_ok();

        let initial_token_balance = 10_000_000_000u64;
        b_mock.borrow_mut().set_dct_balance(
            wmoax_swap_wrapper.address_ref(),
            WMOAX_TOKEN_ID,
            &rust_biguint!(initial_token_balance),
        );
        b_mock.borrow_mut().set_moax_balance(
            wmoax_swap_wrapper.address_ref(),
            &rust_biguint!(initial_token_balance),
        );

        let wmoax_token_roles = [DctLocalRole::Mint, DctLocalRole::Burn];
        b_mock.borrow_mut().set_dct_local_roles(
            wmoax_swap_wrapper.address_ref(),
            WMOAX_TOKEN_ID,
            &wmoax_token_roles[..],
        );

        WmoaxSwapSetup {
            b_mock,
            wmoax_swap_wrapper,
        }
    }
}
