#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Response, QuerierWrapper, Uint128, Storage, StdResult
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{TOKEN0, TOKEN1, PAIR, PRICE0, PRICE1, START_TIME, EPOCH, PERIOD,
    LAST_EPOCH_TIME, OPERATOR};
use terraswap::asset::{AssetInfo, Asset};
use terraswap::pair::{QueryMsg as PairQueryMsg, SimulationResponse, PoolResponse};
use terraswap::querier::{simulate, query_pair_info};

// version info for migration info
const CONTRACT_NAME: &str = "Oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


pub fn get_price(querier: &QuerierWrapper, _pair: Addr, asset_info: &AssetInfo) -> Uint128 {
    let offer_asset = Asset{
        info: asset_info.clone(),
        amount: Uint128::from(1u128)
    };
    let sim_res: SimulationResponse = simulate( querier, _pair, &offer_asset ).unwrap();

    sim_res.return_amount
}
pub fn check_onlyoperator(storage: &dyn Storage, sender: Addr) -> Result<Response, ContractError> {
    let operator = OPERATOR.load(storage)?;
    if operator != sender {
        return Err(ContractError::Unauthorized{});
    }
    Ok(Response::new())
}

pub fn check_starttime(storage: &dyn Storage, env: Env)
    ->Result<Response, ContractError>
{
    if Uint128::from(env.block.time.seconds() as u128) < START_TIME.load(storage)? {
        return Err(ContractError::NotStartedYet{ });
    }
    Ok(Response::new())
}

pub fn get_next_epoch_point(storage: &dyn Storage)
    ->StdResult<Uint128>
{
    Ok(LAST_EPOCH_TIME.load(storage)? + PERIOD.load(storage)?)
}


pub fn check_epoch(storage: &dyn Storage, env: Env, sender: Addr)
    ->Result<Response, ContractError>
{
    let next_epoch_point = get_next_epoch_point(storage)?;
    if Uint128::from(env.block.time.seconds() as u128) < next_epoch_point {
        if sender != OPERATOR.load(storage)? {
            return Err(ContractError::Unauthorized{ });
        }
    } 
    Ok(Response::new())
}
pub fn check_epoch_after(storage:&mut dyn Storage, env: Env)
    ->Result<Response, ContractError>
{
    let mut next_epoch_point = get_next_epoch_point(storage)?;
    loop {
        LAST_EPOCH_TIME.save(storage, &next_epoch_point)?;
        let mut epoch = EPOCH.load(storage)?;
        epoch += Uint128::from(1u128);
        next_epoch_point = get_next_epoch_point(storage)?;
        if Uint128::from(env.block.time.seconds() as u128) < next_epoch_point{
            break;
        }
    }
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    //-----------epoch----------------
    OPERATOR.save(deps.storage, &info.sender)?;

    PERIOD.save(deps.storage, &msg.period)?;
    START_TIME.save(deps.storage, &msg.start_time)?;
    EPOCH.save(deps.storage, &Uint128::zero())?;
    LAST_EPOCH_TIME.save(deps.storage, &(msg.start_time - msg.period))?;
    
    //----------------------------------
    let pair = msg.pair;
    PAIR.save(deps.storage, &pair)?;

    let pair_info: PoolResponse = deps.querier.query_wasm_smart(
        pair.clone(),
        &PairQueryMsg::Pool{}
    ).unwrap();

    let token0 = pair_info.assets[0].info.clone();
    let token1 = pair_info.assets[0].info.clone();
    TOKEN0.save( deps.storage, &token0)?;
    TOKEN1.save( deps.storage, &token1)?;

    PRICE0.save(deps.storage, &get_price(&deps.querier, pair.clone(), &token0))?;
    PRICE1.save(deps.storage, &get_price(&deps.querier, pair, &token1))?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Update {  } => try_update(deps, env, info),
        ExecuteMsg::SetPeriod{ period } => try_setperiod(deps, info, period),
        ExecuteMsg::SetEpoch{ epoch } => try_setepoch(deps, info, epoch),
    }
}

pub fn try_update(deps:DepsMut, env:Env, info:MessageInfo) 
    -> Result<Response, ContractError>
{
    check_epoch(deps.storage, env.clone(), info.sender.clone())?;

    let token0 = TOKEN0.load(deps.storage)?;
    let token1 = TOKEN1.load(deps.storage)?;
    let pair = PAIR.load(deps.storage)?;

    TOKEN0.save( deps.storage, &token0)?;
    TOKEN1.save( deps.storage, &token1)?;

    PRICE0.save(deps.storage, &get_price(&deps.querier, pair.clone(), &token0))?;
    PRICE1.save(deps.storage, &get_price(&deps.querier, pair, &token1))?;

    check_epoch_after(deps.storage, env.clone())?;
    Ok(Response::new())
}

pub fn try_setperiod(deps: DepsMut, info: MessageInfo, period: Uint128) 
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let hour = Uint128::from(3600u128);

    if period < Uint128::from(1u128) * hour || period > Uint128::from(48u128) * hour {
        return Err(ContractError::OutOfRange{ });
    }
    PERIOD.save(deps.storage, &period)?;

    Ok(Response::new())
}

pub fn try_setepoch(deps: DepsMut, info: MessageInfo, epoch: Uint128) 
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;
    EPOCH.save(deps.storage, &epoch)?;

    Ok(Response::new())
}
