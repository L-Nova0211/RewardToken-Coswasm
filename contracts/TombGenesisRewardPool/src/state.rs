use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, U128Key};
use crate::msg::{UserInfo, PoolInfo};

pub const OPERATOR: Item<Addr> = Item::new("OPERATOR");
pub const TOMB: Item<Addr> = Item::new("TOMB");
pub const SHIBA: Item<Addr> = Item::new("SHIBA");

// Info of each pool.
pub const POOLINFO: Item<Vec<PoolInfo>> = Item::new("poolinfo");

// Info of each user that stakes LP tokens.
pub const USERINFO: Map<(U128Key, &Addr), UserInfo> = Map::new("userinfo");

// Total allocation points. Must be the sum of all allocation points in all pools.
pub const TOTALALLOCPOINT: Item<Uint128> = Item::new("TOTALALLOCPOINT");

// The time when TOMB mining starts.
pub const POOLSTARTTIME: Item<Uint128> = Item::new("POOLSTARTTIME");

// The time when TOMB mining ends.
pub const POOLENDTIME: Item<Uint128> = Item::new("POOLENDTIME");
