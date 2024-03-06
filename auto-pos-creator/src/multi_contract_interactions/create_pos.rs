dharitri_sc::imports!();
dharitri_sc::derive_imports!();

use common_structs::PaymentsVec;

use crate::{
    common::payments_wrapper::PaymentsWrapper,
    external_sc_interactions::{
        pair_actions::{PairAddLiqArgs, PairTokenPayments},
        router_actions::SwapOperationType,
    },
};

pub type DoubleSwapResult<M> = PairTokenPayments<M>;

pub struct CreateFarmPosArgs<M: ManagedTypeApi> {
    pub caller: ManagedAddress<M>,
    pub first_token_payment: DctTokenPayment<M>,
    pub second_token_payment: DctTokenPayment<M>,
    pub additional_payments: PaymentsVec<M>,
    pub add_liq_first_token_min_amount_out: BigUint<M>,
    pub add_liq_second_token_min_amount_out: BigUint<M>,
    pub pair_address: ManagedAddress<M>,
    pub farm_address: ManagedAddress<M>,
}

pub struct CreateMetastakingPosArgs<M: ManagedTypeApi> {
    pub caller: ManagedAddress<M>,
    pub first_token_payment: DctTokenPayment<M>,
    pub second_token_payment: DctTokenPayment<M>,
    pub additional_payments: PaymentsVec<M>,
    pub add_liq_first_token_min_amount_out: BigUint<M>,
    pub add_liq_second_token_min_amount_out: BigUint<M>,
    pub pair_address: ManagedAddress<M>,
    pub farm_address: ManagedAddress<M>,
    pub metastaking_address: ManagedAddress<M>,
}

#[dharitri_sc::module]
pub trait CreatePosModule:
    utils::UtilsModule
    + crate::configs::pairs_config::PairsConfigModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + crate::external_sc_interactions::router_actions::RouterActionsModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    fn process_payment(
        &self,
        payment: MoaxOrDctTokenPayment,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> DctTokenPayment {
        let dct_payment = self.get_dct_payment(payment);

        if !swap_operations.is_empty() {
            self.call_router_swap(dct_payment, swap_operations)
        } else {
            dct_payment
        }
    }

    fn swap_half_input_payment_if_needed(
        &self,
        first_payment: &mut DctTokenPayment,
        pair_address: ManagedAddress,
    ) -> DctTokenPayment {
        let pair_config = self.get_pair_config(&pair_address);

        let other_token_id = if first_payment.token_identifier == pair_config.first_token_id {
            pair_config.second_token_id.clone()
        } else if first_payment.token_identifier == pair_config.second_token_id {
            pair_config.first_token_id
        } else if first_payment.token_identifier == pair_config.lp_token_id {
            return DctTokenPayment::new(
                first_payment.token_identifier.clone(),
                0,
                BigUint::zero(),
            );
        } else {
            sc_panic!("The output token identifier is not part of the LP")
        };

        let swap_input_payment = DctTokenPayment::new(
            first_payment.token_identifier.clone(),
            0,
            &first_payment.amount / 2u64,
        );
        first_payment.amount -= &swap_input_payment.amount;
        let second_payment =
            self.call_pair_swap(pair_address.clone(), swap_input_payment, other_token_id);

        self.check_router_pair(
            pair_address,
            first_payment.token_identifier.clone(),
            second_payment.token_identifier.clone(),
        );

        // Reverse tokens if needed
        if first_payment.token_identifier == pair_config.second_token_id {
            let reversed_payment = first_payment.clone();
            first_payment.token_identifier = second_payment.token_identifier;
            first_payment.amount = second_payment.amount;
            reversed_payment
        } else {
            second_payment
        }
    }

    fn get_dct_payment(&self, payment: MoaxOrDctTokenPayment) -> DctTokenPayment {
        require!(payment.token_identifier.is_valid(), "Invalid payment");
        if payment.token_identifier.is_moax() {
            self.call_wrap_moax(payment.amount)
        } else {
            let dct_payment = payment.unwrap_dct();
            require!(dct_payment.token_nonce == 0, "Only fungible DCT accepted");
            dct_payment
        }
    }

    fn split_first_payment(&self) -> (MoaxOrDctTokenPayment, PaymentsVec<Self::Api>) {
        let raw_payments = self.call_value().any_payment();
        match raw_payments {
            MoaxOrMultiDctPayment::Moax(moax_amount) => (
                MoaxOrDctTokenPayment::new(MoaxOrDctTokenIdentifier::moax(), 0, moax_amount),
                ManagedVec::new(),
            ),
            MoaxOrMultiDctPayment::MultiDct(mut dct_payments) => {
                require!(!dct_payments.is_empty(), "Invalid payments");
                let first_payment = self.pop_first_payment(&mut dct_payments);

                (MoaxOrDctTokenPayment::from(first_payment), dct_payments)
            }
        }
    }

    fn split_first_two_payments(
        &self,
    ) -> (DctTokenPayment, DctTokenPayment, PaymentsVec<Self::Api>) {
        let mut all_payments = self.call_value().all_dct_transfers().clone_value();
        require!(all_payments.len() > 1, "Invalid payments");
        let first_payment = self.pop_first_payment(&mut all_payments);
        let second_payment = self.pop_first_payment(&mut all_payments);

        (first_payment, second_payment, all_payments)
    }

    fn create_lp_pos(
        &self,
        args: PairAddLiqArgs<Self::Api>,
    ) -> (DctTokenPayment, PaymentsWrapper<Self::Api>) {
        let mut output_payments = PaymentsWrapper::new();
        if args.second_tokens.amount == 0 {
            let lp_token_id = self
                .lp_token_identifier()
                .get_from_address(&args.pair_address);
            require!(
                args.first_tokens.token_identifier == lp_token_id,
                "Wrong LP token"
            );
            return (args.first_tokens, output_payments);
        }

        let add_liq_result = self.call_pair_add_liquidity(args);

        output_payments.push(add_liq_result.first_tokens_remaining);
        output_payments.push(add_liq_result.second_tokens_remaining);

        (add_liq_result.lp_tokens, output_payments)
    }

    fn create_farm_pos(
        &self,
        args: CreateFarmPosArgs<Self::Api>,
    ) -> (DctTokenPayment, PaymentsWrapper<Self::Api>) {
        let pair_args = PairAddLiqArgs {
            pair_address: args.pair_address,
            first_tokens: args.first_token_payment,
            second_tokens: args.second_token_payment,
            first_token_min_amount_out: args.add_liq_first_token_min_amount_out,
            second_token_min_amount_out: args.add_liq_second_token_min_amount_out,
        };
        let (lp_tokens, mut output_payments) = self.create_lp_pos(pair_args);

        let mut payments = PaymentsVec::from_single_item(lp_tokens);
        payments.append_vec(args.additional_payments);
        let enter_result = self.call_enter_farm(args.farm_address, args.caller, payments);
        output_payments.push(enter_result.rewards);

        (enter_result.new_farm_token, output_payments)
    }

    fn create_metastaking_pos(
        &self,
        args: CreateMetastakingPosArgs<Self::Api>,
    ) -> (DctTokenPayment, PaymentsWrapper<Self::Api>) {
        let farm_args = CreateFarmPosArgs {
            caller: args.caller.clone(),
            first_token_payment: args.first_token_payment,
            second_token_payment: args.second_token_payment,
            additional_payments: PaymentsVec::new(),
            add_liq_first_token_min_amount_out: args.add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out: args.add_liq_second_token_min_amount_out,
            pair_address: args.pair_address,
            farm_address: args.farm_address,
        };
        let (farm_tokens, mut output_payments) = self.create_farm_pos(farm_args);

        let mut payments = PaymentsVec::from_single_item(farm_tokens);
        payments.append_vec(args.additional_payments);
        let stake_result =
            self.call_metastaking_stake(args.metastaking_address, args.caller, payments);

        output_payments.push(stake_result.staking_boosted_rewards);
        output_payments.push(stake_result.lp_farm_boosted_rewards);

        (stake_result.dual_yield_tokens, output_payments)
    }
}
