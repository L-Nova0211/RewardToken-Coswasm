#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, QuerierWrapper,
    Uint128, CosmosMsg, WasmMsg, Storage, StdResult, StdError, Deps
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{
    OWNER, TOMB,  TOMB_ORACLE, TREASURY, ASSETS, VESTING, BOND_THRESHOLD,
    BOND_FACTOR, SECONDARY_THRESHOLD, SECONDARY_FACTOR, BOND_VESTING,
    TOTAL_VESTED, LAST_BUY_BACK, BUYBACK_AMOUNT
};

use crate::util::{ETHER, check_onlyowner,check_only_asset, get_tomb_price, 
    get_total_supply, get_tomb_return, claim_vested

};
use terraswap::querier::{query_token_balance, self};
use Oracle::msg::{ExecuteMsg as OracleMsg};
use BasisAsset::msg::{ExecuteMsg as BasisAssetMsg};
use IMasonry::msg::{ExecuteMsg as MasonryMsg};
use BondTreasury::msg::{QueryMsg as BondTreasuryQuery};
use Treasury::msg::{QueryMsg as TreasuryQuery};

// version info for migration info
const CONTRACT_NAME: &str = "RebateTreasury";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const WFTM: &str = "0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83";
pub const DENOMINATOR: u128 = 1_000_000;//1e6

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    OWNER.save(deps.storage, &info.sender)?;

    TOMB.save(deps.storage, &msg.tomb)?;
    TOMB_ORACLE.save(deps.storage, &msg.tomb_oracle)?;
    TREASURY.save(deps.storage, &msg.treasury)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bond { token, amount } 
            => try_bond(deps, env, info, token, amount),

        ExecuteMsg::ClaimRewards {  }
            => try_claim_rewards(deps, env, info),

        ExecuteMsg::SetTomb { tomb }
            => try_set_tomb(deps, env, info, tomb),

        ExecuteMsg::SetTombOracle{ tomb_oracle }
            => try_set_tomb_oracle(deps, env, info, tomb_oracle),

        ExecuteMsg::SetTreasury { treasury }
            => try_set_treasury(deps, env, info, treasury),

        ExecuteMsg::SetAsset { token, is_added, multiplier, oracle, is_lp, pair }
            => try_set_asset(deps, env, info, token, is_added, multiplier, oracle, is_lp, pair),

        ExecuteMsg::SetBondParameter { primary_threshold, primary_factor, second_threshold, second_factor, vesting_period }
            => try_set_bond_parameter(deps, env, info, primary_threshold, primary_factor, second_threshold, second_factor, vesting_period),

        ExecuteMsg::RedeemAssetsForBuyback { tokens }
            => try_redeem_assets_for_buyback(deps, env, info, tokens)
    }
}
    // Bond asset for discounted Tomb at bond rate
pub fn try_bond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    amount: Uint128
)
    -> Result<Response, ContractError>
{
    check_only_asset(deps.storage, token.clone())?;
    if amount <= Uint128::zero() {
        return Err(ContractError::RebateTreasuryError { msg: "invalid bond amount".to_string() });
    }
    let tomb_amount = get_tomb_return(deps.storage, &deps.querier, token, amount)?;
    let tomb_balance = query_token_balance(
        &deps.querier, 
        TOMB.load(deps.storage)?, 
        env.contract.address
    )?;
    let total_vested = TOTAL_VESTED.load(deps.storage)?;
    if tomb_amount > tomb_balance - total_vested {
        return Err(ContractError::RebateTreasuryError { msg: "insufficient tomb balance".to_string() });
    }


    // IERC20(token).transferFrom(msg.sender, address(this), amount);
    // _claimVested(msg.sender);

    let bond_vesting = BOND_VESTING.load(deps.storage)?;
    let mut schedule  = VESTING.load(deps.storage, info.sender)?;
    schedule.amount = schedule.amount - schedule.claimed + tomb_amount;
    schedule.period = bond_vesting;
    schedule.end = Uint128::from(env.block.time.seconds()) + bond_vesting;
    schedule.claimed = Uint128::zero();
    schedule.last_claimed = Uint128::from(env.block.time.seconds());

    let mut total_vested = TOTAL_VESTED.load(deps.storage)?;
    total_vested += tomb_amount;
    TOTAL_VESTED.save(deps.storage, &total_vested)?;

    Ok(Response::new())
}

pub fn try_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
)
    -> Result<Response, ContractError>
{
    let msg = claim_vested(deps.storage, env, info.sender)?;
    match msg{
        Some(msg) =>     
            Ok(Response::new()
            .add_attribute("action", "claim_rewards")
            .add_message(msg)),
        None => 
            Ok(Response::new()
            .add_attribute("action", "claim_rewards"))
    }
}

pub fn try_set_tomb(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyowner(deps.storage, info.sender)?;
    TOMB.save(deps.storage, &tomb)?;
    Ok(Response::new()
        .add_attribute("action", "set tomb"))
}

pub fn try_set_tomb_oracle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb_oracle: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyowner(deps.storage, info.sender)?;
    TOMB_ORACLE.save(deps.storage, &tomb_oracle)?;
    Ok(Response::new()
        .add_attribute("action", "set tomb oracle"))
}

pub fn try_set_treasury(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    treasury: Addr
)
    -> Result<Response, ContractError>
{
    check_onlyowner(deps.storage, info.sender)?;
    TREASURY.save(deps.storage, &treasury)?;
    Ok(Response::new()
        .add_attribute("action", "set treasury"))
}

  // Set bonding parameters of token
pub fn try_set_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    is_added: bool,
    multiplier: Uint128,
    oracle: Addr,
    is_lp: bool,
    pair: Addr
)
    -> Result<Response, ContractError>
{
    let mut asset = ASSETS.load(deps.storage, token.clone())?;
    asset.is_added = is_added;
    asset.multiplier = multiplier;
    asset.oracle = oracle;
    asset.is_lp = is_lp;
    asset.pair = pair;
    ASSETS.save(deps.storage, token, &asset);
    Ok(Response::new()
        .add_attribute("action", "set asset"))
}

// Set bond pricing parameters
pub fn try_set_bond_parameter(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    primary_threshold: Uint128,
    primary_factor: Uint128,
    second_threshold: Uint128,
    second_factor: Uint128,
    vesting_period: Uint128
)
    -> Result<Response, ContractError>
{
    BOND_THRESHOLD.save(deps.storage, &primary_threshold)?;
    BOND_FACTOR.save(deps.storage, &primary_factor)?;
    SECONDARY_FACTOR.save(deps.storage, &second_threshold)?;
    SECONDARY_FACTOR.save(deps.storage, &second_factor)?;
    BOND_VESTING.save(deps.storage, &vesting_period)?;
    Ok(Response::new()
        .add_attribute("action", "set_asset"))
}

    // Redeem assets for buyback under peg
pub fn try_redeem_assets_for_buyback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tokens: Vec<Addr>
)
    -> Result<Response, ContractError>
{
    check_onlyowner(deps.storage, info.sender)?;
    if get_tomb_price(deps.storage, &deps.querier)? < Uint128::from(ETHER) {
        return Err(ContractError::RebateTreasuryError { msg: "unable to buy back".to_string() });
    }
    let epoch: Uint128 = deps.querier.query_wasm_smart(
        TREASURY.load(deps.storage)?, 
        &TreasuryQuery::Epoch {  }
    )?;
    if LAST_BUY_BACK.load(deps.storage)? == epoch {
        return Err(ContractError::RebateTreasuryError { msg: "already bought back".to_string() });
    }
    LAST_BUY_BACK.save(deps.storage, &epoch)?;

    let mut msgs = Vec::new();
    for token in tokens{
        let asset = ASSETS.load(deps.storage, token.clone())?;
        if asset.is_added == false {
            return Err(ContractError::RebateTreasuryError { msg: "invalid token".to_string() });
        }
        let token_balance = query_token_balance(
            &deps.querier, 
            token.clone(), 
            env.contract.address.clone()
        )?;
        let msg_transfer = WasmMsg::Execute { 
            contract_addr: token.to_string(), 
            msg: to_binary(
                &BasisAssetMsg::Transfer {  
                    recipient: OWNER.load(deps.storage)?.to_string(), 
                    amount: token_balance * BUYBACK_AMOUNT.load(deps.storage)? / Uint128::from(DENOMINATOR) 
                    }
                )?,
            funds: vec![]
        };
        msgs.push(msg_transfer);
    }
    Ok(Response::new()
        .add_attribute("action", "redeem assets for buyback")
        .add_messages(msgs)
    )
}
