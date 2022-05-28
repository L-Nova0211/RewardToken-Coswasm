use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};    
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub ether: Uint128,
    pub day: Uint128,
    
    pub FARMING_POOL_REWARD_ALLOCATION: Uint128,
    pub COMMUNITY_FUND_POOL_ALLOCATION: Uint128,
    
    pub DEV_FUND_POOL_ALLOCATION: Uint128,
    pub VESTING_DURATION: Uint128,
}
pub const startTime: Item<Uint128> = Item::new("startTime");
pub const endTime: Item<Uint128> = Item::new("endTime");

pub const communityFundRewardRate: Item<Uint128> = Item::new("communityFundRewardRate");
pub const devFundRewardRate: Item<Uint128> = Item::new("devFundRewardRate");

pub const communityFund: Item<Addr> = Item::new("communityFund");
pub const devFund: Item<Addr> = Item::new("devFund");

pub const communityFundLastClaimed: Item<Uint128> = Item::new("communityFundLastClaimed");
pub const devFundLastClaimed: Item<Uint128> = Item::new("devFundLastClaimed");

pub const rewardPoolDistributed: Item<bool> = Item::new("rewardPoolDistributed");

