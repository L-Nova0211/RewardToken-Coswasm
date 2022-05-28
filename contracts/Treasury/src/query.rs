#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, Env, StdResult, Addr,
    Uint128
};

use crate::msg::{QueryMsg,};
use crate::state::{EPOCH};
use crate::util::{is_initialized, next_epoch_point, get_tomb_price,
    get_tomb_updated_price, get_reserve, get_burnable_tomb_left,
    get_redeemable_bonds, get_bond_discount_rate,
    get_bond_premium_rate};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsInitialized{ } => {
            to_binary(&is_initialized(deps.storage)?)
        },

        QueryMsg::NextEpochPoint{} => {
            to_binary(&next_epoch_point(deps.storage)?)
        },

        QueryMsg::GetTombPrice{} => {
            to_binary(&get_tomb_price(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetTombUpdatedPrice{} => {
            to_binary(&get_tomb_updated_price(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetReserve{} => {
            to_binary(&get_reserve(deps.storage)?)
        },

        QueryMsg::GetBurnableTombLeft{} => {
            to_binary(&get_burnable_tomb_left(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetRedeemableBonds{} => {
            to_binary(&get_redeemable_bonds(deps.storage, &deps.querier, env)?)
        },

        QueryMsg::GetBondDiscountRate{} => {
            to_binary(&get_bond_discount_rate(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetBondPremiumRate{} => {
            to_binary(&get_bond_premium_rate(deps.storage, &deps.querier)?)
        },

        QueryMsg::Epoch{ } => {
            to_binary( & EPOCH.load(deps.storage)?)
        }
    }
}
