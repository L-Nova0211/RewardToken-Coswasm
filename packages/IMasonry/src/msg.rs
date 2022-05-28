use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub TOMB: String,
    pub POOLSTARTTIME: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Initialize {
        tomb: Addr,
        share: Addr,
        treasury: Addr,
    },
    SetOperator {
        operator: Addr
    },
    SetLockUp {
        withdraw_lockup_epochs: Uint128,
        reward_lockup_epochs: Uint128
    },
    Stake{ amount: Uint128 },
    Withdraw{ amount: Uint128 },
    Exit{ },
    ClaimReward{ },
    AllocateSeigniorage{ amount: Uint128 },
    GovernanceRecoverUnsupported{ 
        token: Addr, 
        amount: Uint128, 
        to: Addr 
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Operator{ },
    
    LatestSnapshotIndex{ },
    GetLastSnapshotIndexOf{ mason: Addr },
    CanWithdraw{ mason: Addr },
    CanClaimReward{ mason: Addr },
    Epoch{ },
    NextEpochPoint{ },
    GetTombPrice{ },

    RewardPerShare{ },
    Earned{ mason: Addr },


}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Masonseat {
    pub last_snapshot_index: Uint128,
    pub reward_earned: Uint128,
    pub epoch_timer_start: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MasonrySnapshot {
    pub time: Uint128,
    pub reward_received: Uint128,
    pub reward_per_share: Uint128
}