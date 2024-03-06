use core::convert::TryFrom;

use crate::{external_sc_interactions, config::{SWAP_ARGS_LEN, ROUTER_SWAP_ARGS_LEN, SEND_TOKENS_ARGS_LEN}};

dharitri_sc::imports!();
dharitri_sc::derive_imports!();

pub type PaymentsVec<M> = ManagedVec<M, DctTokenPayment<M>>;

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, ManagedVecItem)]
pub enum TaskType {
    WrapMOAX,
    UnwrapMOAX,
    Swap,
    RouterSwap,
    SendMoaxOrDct,
}

#[dharitri_sc::module]
pub trait TaskCall:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wmoax_swap::WmoaxWrapModule
{
    #[payable("*")]
    #[endpoint(composeTasks)]
    fn compose_tasks(
        &self,
        min_expected_token_out: MoaxOrDctTokenPayment,
        tasks: MultiValueEncoded<MultiValue2<TaskType, ManagedVec<ManagedBuffer>>>,
    ) {
        let mut payment_for_next_task = self.call_value().moax_or_single_dct();
        let mut payments_to_return = PaymentsVec::new();

        let mut dest_addr = self.blockchain().get_caller();

        for task in tasks.into_iter() {
            let (task_type, args) = task.into_tuple();

            let payment_for_current_task = payment_for_next_task.clone();

            payment_for_next_task = match task_type {
                TaskType::WrapMOAX => self.wrap_moax(payment_for_current_task),
                TaskType::UnwrapMOAX => self.unwrap_moax(payment_for_current_task),
                TaskType::Swap => self.swap(payment_for_current_task, args),
                TaskType::RouterSwap => {
                    self.router_swap(payment_for_current_task, &mut payments_to_return, args)
                }
                TaskType::SendMoaxOrDct => {
                    require!(args.len() == SEND_TOKENS_ARGS_LEN, "Invalid number of arguments!");
                    let new_destination =
                        ManagedAddress::try_from(args.get(0).clone_value()).unwrap_or_else(|err| sc_panic!(err));

                    dest_addr = new_destination;
                    break;
                }
            };
        }
        self.send_resulted_payments(
            dest_addr,
            min_expected_token_out,
            payment_for_next_task,
            &mut payments_to_return,
        )
    }

    fn swap(
        &self,
        payment_for_current_task: MoaxOrDctTokenPayment,
        args: ManagedVec<ManagedBuffer>,
    ) -> MoaxOrDctTokenPayment {
        require!(
            !payment_for_current_task.token_identifier.is_moax(),
            "MOAX can't be swapped!"
        );
        let payment_in = payment_for_current_task.unwrap_dct();

        require!(args.len() == SWAP_ARGS_LEN, "Incorrect arguments for swap task!");

        let token_out = TokenIdentifier::from(args.get(0).clone_value());
        let min_amount_out = BigUint::from(args.get(1).clone_value());

        self.perform_tokens_swap(
            payment_in.token_identifier,
            payment_in.amount,
            token_out,
            min_amount_out,
        )
        .into()
    }

    fn router_swap(
        &self,
        payment_for_current_task: MoaxOrDctTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
        args: ManagedVec<ManagedBuffer>,
    ) -> MoaxOrDctTokenPayment {
        require!(
            !payment_for_current_task.token_identifier.is_moax(),
            "MOAX can't be swapped!"
        );
        require!(
            args.len() % ROUTER_SWAP_ARGS_LEN == 0,
            "Invalid number of router swap arguments"
        );
        let payment_in = payment_for_current_task.unwrap_dct();
        let mut returned_payments_by_router = self.multi_pair_swap(payment_in, args);

        require!(
            !returned_payments_by_router.is_empty(),
            "Router swap returned 0 payments"
        );

        let last_payment_index = returned_payments_by_router.len() - 1;
        let payment_out = returned_payments_by_router.take(last_payment_index);
        payments_to_return.append_vec(returned_payments_by_router);
        MoaxOrDctTokenPayment::from(payment_out)
    }

    fn send_resulted_payments(
        &self,
        dest_addr: ManagedAddress,
        min_expected_token_out: MoaxOrDctTokenPayment,
        payment_for_current_task: MoaxOrDctTokenPayment,
        payments_to_return: &mut PaymentsVec<Self::Api>,
    ) {
        self.require_min_expected_token(&min_expected_token_out, &payment_for_current_task);
        if payment_for_current_task.token_identifier.is_moax() {
            self.send()
                .direct_moax(&dest_addr, &payment_for_current_task.amount);
        } else {
            payments_to_return.push(DctTokenPayment::new(
                payment_for_current_task.token_identifier.unwrap_dct(),
                payment_for_current_task.token_nonce,
                payment_for_current_task.amount,
            ));
        }
        if !payments_to_return.is_empty() {
            self.send().direct_multi(&dest_addr, payments_to_return);
        }
    }

    fn require_min_expected_token(
        &self,
        expected_token: &MoaxOrDctTokenPayment,
        token_out: &MoaxOrDctTokenPayment,
    ) {
        require!(
            expected_token.token_identifier == token_out.token_identifier
                && expected_token.amount <= token_out.amount,
            "The output token is less than minimum required by user!"
        );
    }
}
