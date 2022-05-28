use crate::error::ContractError;

use cosmwasm_std::{ Storage, Uint128, Addr, StdResult, StdError, Response, Env, QuerierWrapper, 
    WasmMsg, to_binary, CosmosMsg};
use terraswap::querier;
use crate::state::{
    OWNER, TOMB,  TOMB_ORACLE, TREASURY, ASSETS, VESTING, BOND_THRESHOLD,
    BOND_FACTOR, SECONDARY_THRESHOLD, SECONDARY_FACTOR, BOND_VESTING,
    TOTAL_VESTED, LAST_BUY_BACK, BUYBACK_AMOUNT
};
use crate::contract::{DENOMINATOR, WFTM};
use BasisAsset::msg::{QueryMsg as BasisAssetQuery, ExecuteMsg as BasisAssetMsg};
use IMasonry::msg::{QueryMsg as MasonryQuery};
use Oracle::msg::{QueryMsg as OracleQuery};

use terraswap::querier::{query_token_balance, query_supply};
use terraswap::asset::{AssetInfo};
use terraswap::pair::{QueryMsg as PairQueryMsg, SimulationResponse, PoolResponse};

pub const ETHER: u128 = 1_000_000_000_000_000_000u128;

pub fn check_onlyowner(storage: &dyn Storage, sender: Addr) -> Result<Response, ContractError> {
    let owner = OWNER.load(storage)?;
    if owner != sender {
        return Err(ContractError::Unauthorized{});
    }
    Ok(Response::new())
}

pub fn check_only_asset(storage: &dyn Storage, token: Addr) -> Result<Response, ContractError>  {
    let asset = ASSETS.load(storage, token)?;
    if asset.is_added == false {
        return Err(ContractError::RebateTreasuryError { msg: "token is not a bondable asset".to_string() })
    }
    Ok(Response::new())
}

pub fn claim_vested(
    storage: &mut dyn Storage,
    env: Env,
    account: Addr
) 
    -> StdResult<Option<CosmosMsg>>
{
    let mut schedule = VESTING.load(storage, account.clone())?;
    if schedule.amount != Uint128::zero() && schedule.amount != schedule.claimed {
        return Ok(None);
    }
    let current_time = Uint128::from(env.block.time.seconds());
    if current_time > schedule.last_claimed && schedule.last_claimed < schedule.end {
        return Ok(None);
    }    

    let duration;
    if current_time > schedule.end {
        duration = schedule.end - schedule.last_claimed;
    }else {
        duration = current_time - schedule.last_claimed;
    }

    let claimable = schedule.amount * duration / schedule.period;
    if claimable == Uint128::zero(){
        return Ok(None)
    }
    
    schedule.claimed += claimable;
    if current_time > schedule.end {
        schedule.last_claimed = schedule.end;
    } else {
        schedule.last_claimed = current_time;
    }

    let mut total_vested = TOTAL_VESTED.load(storage)?;
    total_vested -= claimable;

    let msg_transfer = WasmMsg::Execute { 
        contract_addr: TOMB.load(storage)?.to_string(), 
        msg: to_binary(
            &BasisAssetMsg::Transfer {  
                recipient: account.to_string(), 
                amount: claimable 
                }
            )?,
        funds: vec![]
    };
    Ok(Some(CosmosMsg::Wasm(msg_transfer)))
}

pub fn get_tomb_return(storage: &dyn Storage, querier: &QuerierWrapper, token: Addr, amount: Uint128) -> StdResult<Uint128> {
    check_only_asset(storage, token.clone()).unwrap();
    let tomb_price = get_tomb_price(storage, querier)?;
    let token_price = get_token_price(storage, querier, token.clone())?;
    let bond_premium = get_bond_premium(storage, querier)?;
    let asset = ASSETS.load(storage, token)?;
    let denominator = Uint128::from(DENOMINATOR);

    let res = amount * token_price * 
        (bond_premium + denominator * asset.multiplier / 
        (denominator * denominator) / tomb_price);

    Ok(res)
}

pub fn get_bond_premium(storage: &dyn Storage, querier: &QuerierWrapper)
    -> StdResult<Uint128>
{
    let tomb_price = get_tomb_price(storage, querier)?;
    if tomb_price < Uint128::from(ETHER) {
        return Ok(Uint128::zero());
    }
    let denominator = Uint128::from(DENOMINATOR);

    let tomb_premium = tomb_price * denominator / Uint128::from(ETHER) - denominator;

    let bond_threshold = BOND_THRESHOLD.load(storage)?;
    let bond_factor = BOND_FACTOR.load(storage)?;

    if tomb_premium < bond_threshold {
        return Ok(Uint128::zero());
    }

    if tomb_premium <= SECONDARY_THRESHOLD.load(storage)? {
        return Ok((tomb_premium-bond_threshold) * bond_factor / denominator);
    } else {
        let secondary_threshold = SECONDARY_THRESHOLD.load(storage)?;
        let secondary_factor = SECONDARY_FACTOR.load(storage)?;

        let primary_premium = (secondary_threshold - bond_threshold) * bond_factor / denominator;
        return Ok(primary_premium + (tomb_premium-secondary_threshold) * secondary_factor/denominator);
    }
}

pub fn oracle_consult(querier: &QuerierWrapper, oracle: Addr, token: Addr, amount: Uint128) -> StdResult<Uint128> {
    let token_asset = AssetInfo::Token { contract_addr: token.to_string()};
    let price: Uint128 = querier.query_wasm_smart(
        oracle,
        &OracleQuery::Consult { 
                token: token_asset, 
                amount_in: amount
            }
    )?;

    Ok(price)
}
// oracle
pub fn get_tomb_price(storage: &dyn Storage, querier: &QuerierWrapper) -> StdResult<Uint128> {
    let tomb = TOMB.load(storage)?;    

    let tomb_asset = AssetInfo::Token { contract_addr: tomb.to_string()};
    let price: Uint128 = querier.query_wasm_smart(
        TOMB_ORACLE.load(storage)?,
        &OracleQuery::Consult { 
                token: tomb_asset, 
                amount_in: Uint128::from(ETHER)
            }
    )?;

    Ok(price)
}

pub fn get_token_price(storage: &dyn Storage, querier: &QuerierWrapper, token: Addr) 
    -> StdResult<Uint128> 
{
    check_only_asset(storage, token.clone()).unwrap();
    let mut asset = ASSETS.load(storage, token.clone())?;
    let oracle = asset.oracle;
    if asset.is_lp == false {
        return Ok(oracle_consult(querier, oracle, token, Uint128::from(ETHER))?);
    }
    let pair = asset.pair;
    let total_pair_supply = query_supply(querier, pair.clone())?;

    let pair_info: PoolResponse = querier.query_wasm_smart(
        pair.clone(),
        &PairQueryMsg::Pool{}
    ).unwrap();

    let token0 = Addr::unchecked(pair_info.assets[0].info.clone().to_string());
    let token1 = Addr::unchecked(pair_info.assets[0].info.clone().to_string());

    let reserve0 = pair_info.assets[0].amount;
    let reserve1 = pair_info.assets[1].amount;

    let wftm_asset = Addr::unchecked(WFTM.to_string());
        
    if token1 == wftm_asset {
        let token_price = oracle_consult(querier, oracle, token0, Uint128::from(ETHER))?;
        return Ok(token_price * reserve0 / total_pair_supply + reserve1 * Uint128::from(ETHER) / total_pair_supply);
    } else {
        let token_price = oracle_consult(querier, oracle, token1, Uint128::from(ETHER))?;
        return Ok(token_price * reserve1 / total_pair_supply + reserve0 * Uint128::from(ETHER) / total_pair_supply);
    }
}


pub fn get_total_supply(querier: &QuerierWrapper, token: Addr) -> StdResult<Uint128>{
    let total_supply: Uint128 = querier.query_wasm_smart(
        token, 
        &BasisAssetQuery::TotalSupply {  }
    )?;
    Ok(total_supply)
}


// Get claimable vested Tomb for account
pub fn claimable_tomb(storage: &dyn Storage, env: Env, querier: &QuerierWrapper, account: Addr)
    -> StdResult<Uint128>
{
    let schedule = VESTING.load(storage, account)?;
    let current_time = Uint128::from(env.block.time.seconds() as u128);

    if current_time <= schedule.last_claimed || schedule.last_claimed >= schedule.end {
        return Ok(Uint128::zero());
    }
    let duration;
    if current_time > schedule.end {
        duration = schedule.end - schedule.last_claimed;
    } else {
        duration = current_time - schedule.last_claimed;
    }
    Ok(schedule.amount * duration / schedule.period)
}
