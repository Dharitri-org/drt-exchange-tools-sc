dharitri_sc::imports!();

#[dharitri_sc::module]
pub trait MoaxWrapperActionsModule {
    fn call_wrap_moax(&self, moax_amount: BigUint) -> DctTokenPayment {
        let wrapper_sc_address = self.moax_wrapper_address().get();
        let ((), back_transfers) = self
            .moax_wrapper_proxy(wrapper_sc_address)
            .wrap_moax()
            .with_moax_transfer(moax_amount)
            .execute_on_dest_context_with_back_transfers();

        let returned_wrapped_moax = back_transfers.dct_payments;
        require!(
            returned_wrapped_moax.len() == 1,
            "wrap_moax should output only 1 payment"
        );

        returned_wrapped_moax.get(0)
    }

    #[storage_mapper("moaxWrapperAddress")]
    fn moax_wrapper_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn moax_wrapper_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> dharitri_wmoax_swap_sc::Proxy<Self::Api>;
}
