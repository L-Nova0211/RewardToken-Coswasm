#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, Env, StdResult, Addr,
    Uint128
};

use crate::msg::{QueryMsg,};

use crate::util::{get_tomb_price, get_tomb_return, get_bond_premium,
    get_token_price
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetTombReturn { token, amount } => {
            to_binary(&get_tomb_return(deps.storage, &deps.querier, token, amount)?)
        },

        QueryMsg::GetBondPremium {  } => {
            to_binary(&get_bond_premium(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetTombPrice {  } => {
            to_binary(&get_tomb_price(deps.storage, &deps.querier)?)
        },

        QueryMsg::GetTokenPrice { token } => {
            to_binary(&get_token_price(deps.storage, &deps.querier, token)?)
        }
    }
}
