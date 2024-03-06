dharitri_sc::imports!();

use dharitri_wmoax_swap_sc::ProxyTrait as _;

#[dharitri_sc::module]
pub trait WmoaxWrapModule {
    fn wrap_moax(&self, payment: MoaxOrDctTokenPayment) -> MoaxOrDctTokenPayment {
        require!(
            payment.token_identifier.is_moax(),
            "Payment token is not MOAX!"
        );

        let wrap_moax_addr = self.wrap_moax_addr().get();

        let ((), back_transfers) = self
            .wrap_moax_proxy(wrap_moax_addr)
            .wrap_moax()
            .with_moax_transfer(payment.amount)
            .execute_on_dest_context_with_back_transfers();

        let returned_wrapped_moax = back_transfers.dct_payments;
        require!(
            returned_wrapped_moax.len() == 1,
            "wrap_moax should output only 1 payment"
        );

        MoaxOrDctTokenPayment::from(returned_wrapped_moax.get(0))
    }

    fn unwrap_moax(&self, payment: MoaxOrDctTokenPayment) -> MoaxOrDctTokenPayment {
        let wrap_moax_addr = self.wrap_moax_addr().get();

        let ((), back_transfers) = self
            .wrap_moax_proxy(wrap_moax_addr)
            .unwrap_moax()
            .with_dct_transfer(payment.unwrap_dct())
            .execute_on_dest_context_with_back_transfers();

        let returned_moax = back_transfers.total_moax_amount;

        MoaxOrDctTokenPayment::new(MoaxOrDctTokenIdentifier::moax(), 0, returned_moax)
    }

    #[proxy]
    fn wrap_moax_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> dharitri_wmoax_swap_sc::Proxy<Self::Api>;

    #[storage_mapper("wrapMoaxAddr")]
    fn wrap_moax_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
