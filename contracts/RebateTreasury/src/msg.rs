use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub tomb: Addr,
    pub tomb_oracle: Addr,
    pub treasury: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bond{token: Addr, amount: Uint128},
    ClaimRewards{},
    SetTomb{tomb: Addr},
    SetTombOracle{tomb_oracle: Addr},
    SetTreasury{treasury: Addr},
    SetAsset{
        token: Addr,
        is_added: bool,
        multiplier: Uint128,
        oracle: Addr,
        is_lp: bool,
        pair: Addr
    },
    SetBondParameter{
        primary_threshold: Uint128,
        primary_factor: Uint128,
        second_threshold: Uint128,
        second_factor: Uint128,
        vesting_period: Uint128
    },
    RedeemAssetsForBuyback{
        tokens: Vec<Addr>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetTombReturn{token: Addr, amount: Uint128},
    GetBondPremium{},
    GetTombPrice{},
    GetTokenPrice{token: Addr}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub is_added: bool,
    pub multiplier: Uint128,
    pub oracle: Addr,
    pub is_lp: bool,
    pub pair: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingSchedule {
    pub amount: Uint128,
    pub period: Uint128,
    pub end: Uint128,
    pub claimed: Uint128,
    pub last_claimed: Uint128
}