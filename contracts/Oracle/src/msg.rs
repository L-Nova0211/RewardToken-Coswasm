use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terraswap::asset::{PairInfoRaw, AssetInfoRaw, AssetInfo, Asset};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pair: Addr,
    pub period: Uint128,
    pub start_time: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Update {},
    SetPeriod{period: Uint128},
    SetEpoch{epoch: Uint128},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Consult{
        token: AssetInfo,
        amount_in: Uint128
    },
    Twap{
        token: AssetInfo,
        amount_in: Uint128
    },
    GetCurrentEpoch{},
    GetPeriod{},
    GetStartTime{},
    GetLastEpochTime{},
    NextEpochPoint{}
}
