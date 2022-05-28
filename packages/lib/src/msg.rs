use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub TOMB: String,
    pub SHIBA: String,
    pub POOLSTARTTIME: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Add {
        alloc_point: Uint128,
        token: Addr,
        with_update: bool,
        last_reward_time: Uint128
    },
    Set {
        pid: Uint128,
        alloc_point: Uint128,
    },
    MassUpdatePools{},
    UpdatePool{
        pid: Uint128
    },
    Deposit{
        pid: Uint128,
        amount: Uint128
    },
    Withdraw{
        pid: Uint128,
        amount: Uint128
    },
    EmergencyWithdraw{
        pid: Uint128
    },
    SetOperator{
        operator: Addr
    },
    GovernanceRecoverUnsupported{
        token: Addr,
        amount: Uint128,
        to: Addr
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetOwner{ },
    GetGeneratedReward{
        from_time: Uint128,
        to_time: Uint128
    },
    PendingTomb{
        pid: Uint128,
        user: Addr
    },
    GetPoolInfo{ },
    GetUserInfo{ 
        pid: Uint128,
        user: Addr
    }
}

// Info of each user.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    pub amount: Uint128, // How many tokens the user has provided.
    pub rewardDebt: Uint128, // Reward debt. See explanation below.
}

// Info of each pool.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfo {
    pub token: Addr, // Address of LP token contract.
    pub allocPoint: Uint128, // How many allocation points assigned to this pool. TOMB to distribute.
    pub lastRewardTime: Uint128, // Last time that TOMB distribution occurs.
    pub accTombPerShare: Uint128, // Accumulated TOMB per share, times 1e18. See below.
    pub isStarted: bool, // if lastRewardBlock has passed
}