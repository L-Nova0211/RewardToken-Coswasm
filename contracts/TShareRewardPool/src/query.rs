#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, Env, StdResult, Addr,
    Uint128
};

use crate::msg::{QueryMsg};
use crate::state::{OPERATOR, POOLINFO, USERINFO, TOTALALLOCPOINT};
use crate::contract::{get_generated_reward, balance_of, ETHER};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner{ } => {
            let owner = OPERATOR.load(deps.storage).unwrap();
            to_binary(&owner)
        }

        QueryMsg::GetGeneratedReward{ from_time, to_time } => {
            let res: u128 = get_generated_reward(deps.storage, from_time, to_time);
            to_binary(&res)
        }

        QueryMsg::PendingShare{ pid, user } => {
            to_binary(&get_pending_share(deps, env, pid, user)?)
        }

        QueryMsg::GetPoolInfo{ } => {
            let pool_info = POOLINFO.load(deps.storage).unwrap();
            to_binary(&pool_info)
        }

        QueryMsg::GetUserInfo{ pid, user } => {
            let user_info = USERINFO.load(deps.storage, (pid.u128().into(), &user)).unwrap();
            to_binary(&user_info)
        }
    }
}

fn get_pending_share(deps: Deps, env: Env, pid: Uint128, user: Addr) -> StdResult<Uint128>
{
    let pool_info = &mut POOLINFO.load(deps.storage)?;
    let pool = &pool_info[pid.u128() as usize];

    let _user = USERINFO.load(deps.storage, (pid.u128().into(), &user))?;
    
    let mut acc_tshare_per_share = pool.accTSharePerShare;
    let token_supply = balance_of(deps.querier, &pool.token, &env.contract.address);
    let blocktime = Uint128::from(env.block.time.seconds());
    
    if blocktime > pool.lastRewardTime && token_supply != 0 {
        let generated_reward = get_generated_reward(deps.storage, pool.lastRewardTime, blocktime);
        let total_alloc_point = TOTALALLOCPOINT.load(deps.storage)?;
        let tshare_reward = Uint128::from(generated_reward) * pool.allocPoint / total_alloc_point;
        acc_tshare_per_share = acc_tshare_per_share + tshare_reward * Uint128::from(ETHER) / Uint128::from(token_supply);
    }

    Ok(_user.amount * acc_tshare_per_share / Uint128::from(ETHER) - _user.rewardDebt)
}