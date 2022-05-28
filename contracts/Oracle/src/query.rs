#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, Deps, Env, StdResult, StdError
};
use crate::state::{TOKEN0, TOKEN1, PAIR, PRICE0, PRICE1, START_TIME, EPOCH, PERIOD,
    LAST_EPOCH_TIME, OPERATOR};
use crate::contract::{get_price, get_next_epoch_point};
use crate::msg::{QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Consult{ token, amount_in} => {
            let token0 = TOKEN0.load(deps.storage)?;
            let token1 = TOKEN1.load(deps.storage)?;
            let price0 = PRICE0.load(deps.storage)?;
            let price1 = PRICE1.load(deps.storage)?;

            let amount_out;
            if token == token0 {
                amount_out = price0 * amount_in;

            } else {
                if token != token1{
                    return Err(StdError::GenericErr{
                        msg: "Invalid Token".to_string()
                    });
                }
                amount_out = price1 * amount_in;
            }
            to_binary(&amount_out)
        }

        QueryMsg::Twap{ token, amount_in } => {
            let token0 = TOKEN0.load(deps.storage)?;
            let token1 = TOKEN1.load(deps.storage)?;
            let pair = PAIR.load(deps.storage)?;

            let price0 = get_price(&deps.querier, pair.clone(), &token0);
            let price1 = get_price(&deps.querier, pair, &token1);

            let amount_out;
            if token == token0 {
                amount_out = price0 * amount_in;

            } else {
                if token != token1{
                    return Err(StdError::GenericErr{
                        msg: "Invalid Token".to_string()
                    });
                }
                amount_out = price1 * amount_in;
            }
            to_binary(&amount_out)
        }

        QueryMsg::GetCurrentEpoch{} => {
            to_binary(&EPOCH.load(deps.storage)?)
        }
    
        QueryMsg::GetPeriod{} => {
            to_binary(&PERIOD.load(deps.storage)?)
        }

        QueryMsg::GetStartTime{} => {
            to_binary(&START_TIME.load(deps.storage)?)
        }

        QueryMsg::GetLastEpochTime{} => {
            to_binary(&LAST_EPOCH_TIME.load(deps.storage)?)
        }

        QueryMsg::NextEpochPoint{} => {
            to_binary(&get_next_epoch_point(deps.storage)?)
        }
    }
}
