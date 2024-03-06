#![no_std]

dharitri_sc::imports!();

pub mod common;
pub mod configs;
pub mod external_sc_interactions;
pub mod multi_contract_interactions;

#[dharitri_sc::contract]
pub trait AutoPosCreator:
    utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::farm_staking_actions::FarmStakingActionsModule
    + external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + external_sc_interactions::moax_wrapper_actions::MoaxWrapperActionsModule
    + multi_contract_interactions::create_pos::CreatePosModule
    + multi_contract_interactions::create_pos_endpoints::CreatePosEndpointsModule
    + multi_contract_interactions::exit_pos::ExitPosModule
    + multi_contract_interactions::exit_pos_endpoints::ExitPosEndpointsModule
{
    #[init]
    fn init(&self, moax_wrapper_address: ManagedAddress, router_address: ManagedAddress) {
        self.require_sc_address(&moax_wrapper_address);
        self.require_sc_address(&router_address);
        self.moax_wrapper_address().set(moax_wrapper_address);
        self.router_address().set(router_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
