use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, U128Key};
use crate::msg::{UserInfo, PoolInfo};

pub const OPERATOR: Item<Addr> = Item::new("OPERATOR");
pub const TOMB: Item<Addr> = Item::new("TOMB");

// Info of each pool.
pub const POOLINFO: Item<Vec<PoolInfo>> = Item::new("poolinfo");

// Info of each user that stakes LP tokens.
pub const USERINFO: Map<(U128Key, &Addr), UserInfo> = Map::new("userinfo");

// Total allocation points. Must be the sum of all allocation points in all pools.
pub const TOTALALLOCPOINT: Item<Uint128> = Item::new("TOTALALLOCPOINT");

// The time when TOMB mining starts.
pub const POOLSTARTTIME: Item<Uint128> = Item::new("POOLSTARTTIME");

// Time when each epoch ends.
pub const EPOCHENDTIMES: Item<Vec<Uint128>> = Item::new("EPOCHENDTIMES");

// Reward per second for each of 2 epochs (last item is equal to 0 - for sanity).
pub const EPOCHTOMBPERSECOND: Item<Vec<Uint128>> = Item::new("EPOCHTOMBPERSECOND");