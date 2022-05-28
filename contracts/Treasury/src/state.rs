use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map, U128Key};

pub const OPERATOR: Item<Addr> = Item::new("operator");
pub const INITIALIZED: Item<bool> = Item::new("initialized");

    // epoch
pub const START_TIME: Item<Uint128> = Item::new("starttime");
pub const EPOCH: Item<Uint128> = Item::new("epoch");
pub const EPOCH_SUPPLY_CONTRACTION_LEFT: Item<Uint128> = Item::new("epoch supply contract left");

    // exclusions from total supply
pub const EXCLUDED_FROM_TOTALSUPPLY: Item<Vec<Addr>> = Item::new("excluded from total supply");

    // core components
pub const TOMB: Item<Addr> = Item::new("tomb");
pub const TBOND: Item<Addr> = Item::new("tbond");
pub const TSHARE: Item<Addr> = Item::new("share");

pub const MASONRY: Item<Addr> = Item::new("masonry");
pub const BOND_TREASURY: Item<Addr> = Item::new("bond treasury");
pub const TOMB_ORACLE: Item<Addr> = Item::new("tomb oracle");

    // price
pub const TOMB_PRICE_ONE: Item<Uint128> = Item::new("tomb price one");
pub const TOMB_PRICE_CEILING: Item<Uint128> = Item::new("tomb price ceiling");

pub const SEIGNIORAGE_SAVED: Item<Uint128> = Item::new("seigniorage saved");

pub const SUPPLY_TIERS: Item<Vec<Uint128>> = Item::new("supply tiers");
pub const MAX_EXPANSION_TIERS: Item<Vec<Uint128>> = Item::new("max expansion tiers");

pub const MAX_SUPPLY_EXPANSION_PERCENT: Item<Uint128> = Item::new("max supply expansion percent");
pub const BOND_DEPLETION_FLOOR_PERCENT: Item<Uint128> = Item::new("bond depletion floor percent");
pub const SEIGNIORAGE_EXPANSION_FLOOR_PERCENT: Item<Uint128> = Item::new("seigniorage expansion floor percent");
pub const MAX_SUPPLY_CONTRACTION_PERCENT: Item<Uint128> = Item::new("max supply contraction percent");
pub const MAX_DEBT_RATIO_PERCENT: Item<Uint128> = Item::new("max debt ratio percent");

pub const BOND_SUPPLY_EXPANSION_PERCENT: Item<Uint128> = Item::new("bond supply expansion percent");

// 28 first epochs (1 week) with 4.5% expansion regardless of TOMB price
pub const BOOTSTRAP_EPOCHS: Item<Uint128> = Item::new("bootstrap epochs");
pub const BOOTSTRAP_SUPPLY_EXPANSION_PERCENT: Item<Uint128> = Item::new("bootstrap supply expansion percent");

    /* =================== Added variables =================== */
pub const PREVIOUS_EPOCH_TOMB_PRICE: Item<Uint128> = Item::new("previous epoch tomb price");
pub const MAX_DISCOUNT_RATE: Item<Uint128> = Item::new("max discount rate");
pub const MAX_PREMIUM_RATE: Item<Uint128> = Item::new("max premium rate");
pub const DISCOUNT_PERCENT: Item<Uint128> = Item::new("discount percent");
pub const PREMIUM_THRESHOLD: Item<Uint128> = Item::new("premium threshold");
pub const PREMIUM_PERCENT: Item<Uint128> = Item::new("premium percent");
pub const MINTING_FACTOR_FOR_PAYING_DEBT: Item<Uint128> = Item::new("minting factor for paying debt");

pub const DAOFUND: Item<Addr> = Item::new("daofund");
pub const DAOFUND_SHARED_PERCENT: Item<Uint128> = Item::new("daofund shared percent");
pub const DEVFUND: Item<Addr> = Item::new("dev fund");
pub const DEVFUND_SHARED_PERCENT: Item<Uint128> = Item::new("devfund shared percent");