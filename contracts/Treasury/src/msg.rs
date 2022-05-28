use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Initialize{ 
        tomb: Addr,
        tbond: Addr,
        tshare: Addr,
        tomb_oracle: Addr,
        masonry: Addr,
        genesis_pool: Addr,
        bond_treasury: Addr,
        start_time: Uint128
    },
    SetOperator{
        operator: Addr,
    },
    SetMasonry{
        masonry: Addr
    },
    SetBondTreasury{
        bond_treasury: Addr
    },
    SetTombOracle{
        tomb_oracle: Addr,
    },
    SetTombPriceCeiling{
        tomb_price_ceiling: Uint128
    },
    SetMaxSupplyExpansionPercents{
        max_supply_expansion_percent: Uint128
    },
    SetSupplyTiersEntry{
        index: Uint128,
        value: Uint128
    },
    SetBondDepletionFloorPercent{
        bond_depletion_floor_percent: Uint128
    },
    SetMaxSupplyContractionPercent{
        max_supply_contraction_percent: Uint128
    },
    SetMaxDebtRatioPercent{
        max_debt_ratio_percent: Uint128
    },
    SetBootstrap{
        bootstrap_epochs: Uint128, 
        bootstrap_supply_expansion_percent: Uint128
    },
    SetExtraFunds{
        dao_fund: Addr,
        dao_fund_shared_percent: Uint128,
        dev_fund: Addr,
        dev_fund_shared_percent: Uint128
    },
    SetMaxDiscountRate{
        max_discount_rate: Uint128
    },
    SetMaxPremiumRate{
        max_premium_rate: Uint128
    },
    SetDiscountPercent{
        discount_percent: Uint128
    },
    SetPremiumThreshold{
        premium_threshold: Uint128
    },
    SetPremiumPercent{
        premium_percent: Uint128
    },
    SetMintingFactorForPayingDebt{
        minting_factor_for_paying_debt: Uint128
    },
    SetBondSupplyExpansionPercent{
        bond_supply_expansion_percent: Uint128
    },
    UpdateTombPrice{ },
    BuyBonds{
        tomb_amount: Uint128,
        target_price: Uint128,
    },
    RedeemBonds{
        bond_amount: Uint128,
        target_price: Uint128,
    },
    SendToMasonry{
        amount: Uint128
    },
    SendToBondTreasury{
        amount: Uint128
    },
    AllocateSeigniorage{},
    GovernanceRecoverUnsupported{
        token: Addr,
        amount: Uint128,
        to: Addr
    },
    MasonrySetOperator{
        operator: Addr
    },
    MasonrySetLockup{
        withdraw_lockup_epochs: Uint128,
        reward_lockup_epochs: Uint128,
    },
    MasonryAllocationSeigniorage{
        amount: Uint128
    },
    MasonryGovernanceRecoverUnsupported{
        token: Addr,
        amount: Uint128,
        to: Addr
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsInitialized{},
    NextEpochPoint{},
    GetTombPrice{},
    GetTombUpdatedPrice{},
    GetReserve{},
    GetBurnableTombLeft{},
    GetRedeemableBonds{},
    GetBondDiscountRate{},
    GetBondPremiumRate{},
    Epoch{}
}
