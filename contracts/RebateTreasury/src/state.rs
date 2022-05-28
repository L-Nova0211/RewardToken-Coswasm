use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, U128Key};

use crate::msg::{Asset, VestingSchedule};

pub const OWNER: Item<Addr> = Item::new("owner");

pub const TOMB: Item<Addr> = Item::new("tomb");
pub const TOMB_ORACLE: Item<Addr> = Item::new("tomb oracle");
pub const TREASURY: Item<Addr> = Item::new("treasury");

pub const ASSETS: Map<Addr, Asset> = Map::new("assets");
pub const VESTING: Map<Addr, VestingSchedule> = Map::new("vesting schedule");

pub const BOND_THRESHOLD: Item<Uint128> = Item::new("bondThreshold");
pub const BOND_FACTOR: Item<Uint128> = Item::new("bondFactor");
pub const SECONDARY_THRESHOLD: Item<Uint128> = Item::new("secondaryThreshold");
pub const SECONDARY_FACTOR: Item<Uint128> = Item::new("secondaryFactor");

pub const BOND_VESTING: Item<Uint128> = Item::new("bondVesting");
pub const TOTAL_VESTED: Item<Uint128> = Item::new("totalVested");

pub const LAST_BUY_BACK: Item<Uint128> = Item::new("lastBuyback");
pub const BUYBACK_AMOUNT: Item<Uint128> = Item::new("buybackAmount");
