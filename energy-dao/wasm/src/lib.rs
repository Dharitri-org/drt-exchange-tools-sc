// Code generated by the dharitri-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           47
// Async Callback:                       1
// Total number of exported functions:  49

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

dharitri_sc_wasm_adapter::allocator!();
dharitri_sc_wasm_adapter::panic_handler!();

dharitri_sc_wasm_adapter::endpoints! {
    energy_dao
    (
        init => init
        upgrade => upgrade
        registerWrappedFarmToken => register_wrapped_farm_token
        registerUnstakeFarmToken => register_unstake_farm_token
        registerWrappedMetastakingToken => register_wrapped_metastaking_token
        registerUnstakeMetastakingToken => register_unstake_metastaking_token
        setExitPenaltyPercent => set_exit_penalty_percent
        addFarms => add_farms
        removeFarms => remove_farms
        addMetastakingAddresses => add_metastaking_addresses
        removeMetastakingAddresses => remove_metastaking_addresses
        getFarmState => get_farm_state
        getFarmingTokenId => get_farming_token
        getFarmTokenId => get_farm_token
        getDivisionSafetyConstant => get_division_safety_constant
        getMinimumFarmingEpoch => get_minimum_farming_epochs
        getDualYieldTokenId => get_dual_yield_token
        getLpFarmTokenId => get_lp_farm_token
        getStakingTokenId => get_staking_token
        getLpFarmAddress => get_lp_farm_address
        getStakingFarmAddress => get_staking_farm_address
        getWrappedFarmTokenId => wrapped_farm_token
        getUnstakeFarmTokenId => unstake_farm_token
        getWrappedMetastakingTokenId => wrapped_metastaking_token
        getUnstakeMetastakingTokenId => unstake_metastaking_token
        getExitPenaltyPercent => exit_penalty_percent
        enterFarm => enter_farm_endpoint
        claimUserRewards => claim_user_rewards
        unstakeFarm => unstake_farm
        unbondFarm => unbond_farm
        enterMetastaking => enter_metastaking_endpoint
        unstakeMetastaking => unstake_metastaking
        unbondMetastaking => unbond_metastaking_endpoint
        claimMetastakingRewards => claim_metastaking_rewards
        lockEnergyTokens => lock_energy_tokens
        extendLockPeriod => extend_lock_period
        getInternalLockedTokens => internal_locked_tokens
        claimFeesCollectorRewards => claim_fees_collector_rewards
        setEnergyFactoryAddress => set_energy_factory_address
        getEnergyFactoryAddress => energy_factory_address
        addAdmin => add_admin_endpoint
        removeAdmin => remove_admin_endpoint
        updateOwnerOrAdmin => update_owner_or_admin_endpoint
        getPermissions => permissions
        issueWrappedToken => issue_wrapped_token
        setTransferRoleWrappedToken => set_transfer_role
        unsetTransferRoleWrappedToken => unset_transfer_role
        getWrappedTokenId => wrapped_token
    )
}

dharitri_sc_wasm_adapter::async_callback! { energy_dao }
