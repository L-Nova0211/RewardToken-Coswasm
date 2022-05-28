use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item};
use terraswap::asset::{AssetInfo};

//---epoch-----
pub const OPERATOR: Item<Addr> = Item::new("operator");

pub const PERIOD: Item<Uint128> = Item::new("period");
pub const START_TIME: Item<Uint128> = Item::new("start_time");
pub const LAST_EPOCH_TIME: Item<Uint128> = Item::new("last_epoch_time");
pub const EPOCH: Item<Uint128> = Item::new("epoch");

//------------

pub const TOKEN0: Item<AssetInfo> = Item::new("token0");
pub const TOKEN1: Item<AssetInfo> = Item::new("token1");
pub const PAIR: Item<Addr> = Item::new("pair");
pub const BLOCKTIMESTAMP_LAST: Item<Uint128> = Item::new("blocktimestamp_last");
pub const PRICE0: Item<Uint128> = Item::new("price0");
pub const PRICE1: Item<Uint128> = Item::new("price1");