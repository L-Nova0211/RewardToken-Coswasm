#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, DepsMut, Env, MessageInfo, Response, Storage,
    Uint128, CosmosMsg, StdResult, StdError, QuerierWrapper
};

use IMasonry::msg::{ExecuteMsg, InstantiateMsg, MasonrySnapshot};
use Treasury::msg::{QueryMsg as TreasuryQuery};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{OPERATOR, TOMB, SHARE, TOTALSUPPLY, INITIALIZED, BALANCES,
    TREASURY, MASONS, MASONRY_HISTORY, WITHDRAW_LOCKUP_EPOCHS, REWARD_LOCKUP_EPOCHS};
use crate::util::{balance_of, check_onlyoperator, check_not_initialized, check_mason_exists,
    safe_share_transferfrom, safe_tomb_transferfrom, safe_transferfrom, update_reward, 
    get_latest_snapshot, check_onlyoneblock};
// version info for migration info
const CONTRACT_NAME: &str = "Masonry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    INITIALIZED.save(deps.storage, &false)?;
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
        ExecuteMsg::Initialize{ tomb, share, treasury }
            => try_initialize(deps, env, info, tomb, share, treasury ),
        
        ExecuteMsg::SetOperator{ operator }
            => try_setoperator(deps, info, operator),
        
        ExecuteMsg::SetLockUp{ withdraw_lockup_epochs,reward_lockup_epochs }
            => try_setlockup(deps, env, info, withdraw_lockup_epochs, reward_lockup_epochs),

        ExecuteMsg::Stake { amount }
            => try_stake(deps, env, info, amount),

        ExecuteMsg::Withdraw { amount }
            => try_withdraw(deps, env, info, amount),

        ExecuteMsg::Exit {  }
            => try_exit(deps, env, info),

        ExecuteMsg::ClaimReward{ }
            => try_claimreward(deps, env, info),

        ExecuteMsg::AllocateSeigniorage{ amount }
            => try_allocate_seigniorage(deps, env, info, amount),

        ExecuteMsg::GovernanceRecoverUnsupported{ token, amount, to }
            =>try_governance_recover_unsupported(deps, env, info, token, amount, to),
    }
}
pub fn try_initialize(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tomb: Addr,
    share: Addr,
    treasury: Addr,
)
    -> Result<Response, ContractError>
{
    check_not_initialized(deps.storage)?;

    TOMB.save(deps.storage, &tomb)?;
    SHARE.save(deps.storage, &share)?;
    TREASURY.save(deps.storage, &treasury)?;

    let genesis_snapshot: MasonrySnapshot = MasonrySnapshot{
        time : Uint128::from(env.block.time.seconds()), 
        reward_received : Uint128::zero(), 
        reward_per_share : Uint128::zero()
    };
    let mut masonry_history: Vec<MasonrySnapshot> = Vec::new();
    masonry_history.push(genesis_snapshot);
    MASONRY_HISTORY.save(deps.storage, &masonry_history)?;

    WITHDRAW_LOCKUP_EPOCHS.save(deps.storage, &Uint128::from(3u128))?;// Lock for 6 epochs (36h) before release withdraw
    REWARD_LOCKUP_EPOCHS.save(deps.storage, &Uint128::from(10u128))?; // Lock for 3 epochs (18h) before release claimReward

    INITIALIZED.save(deps.storage, &true)?;
    OPERATOR.save(deps.storage, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "initialize"))
}
pub fn try_setoperator(
    deps: DepsMut,
    info: MessageInfo,
    operator: Addr,
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    OPERATOR.save(deps.storage, &operator)?;
    Ok(Response::new()
        .add_attribute("action", "set operator"))
}

pub fn try_setlockup(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    withdraw_lockup_epochs: Uint128,
    reward_lockup_epochs: Uint128
)
    -> Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    if withdraw_lockup_epochs < reward_lockup_epochs || withdraw_lockup_epochs > Uint128::from(56u128) {
        return Err(ContractError::OutofRange{});
    }
    WITHDRAW_LOCKUP_EPOCHS.save(deps.storage, &withdraw_lockup_epochs)?;
    REWARD_LOCKUP_EPOCHS.save(deps.storage, &reward_lockup_epochs)?;

    Ok(Response::new()
        .add_attribute("action", "set lock up"))
}

pub fn _stake(
    storage: &mut dyn Storage,
    querier: &QuerierWrapper,
    env: Env,
    sender: Addr,
    amount: Uint128
)
    -> StdResult<CosmosMsg>
{
    let mut total_supply = TOTALSUPPLY.load(storage)?;
    total_supply += amount;
    TOTALSUPPLY.save(storage, &total_supply)?;

    let mut balance = BALANCES.load(storage, sender.clone())?;
    balance += amount;
    BALANCES.save(storage, sender.clone(), &balance)?;
    
    safe_share_transferfrom(storage, querier, sender, env.contract.address, amount)
}

pub fn _withdraw(
    storage: &mut dyn Storage,
    querier: &QuerierWrapper,
    env: Env,
    sender: Addr,
    amount: Uint128
)
    -> StdResult<CosmosMsg>
{
    let mut mason_share = BALANCES.load(storage, sender.clone())?;
    if mason_share < amount {
        return Err(StdError::GenericErr { 
            msg: "Masonry: withdraw request greater than staked amount".to_string() 
        })
    }

    let mut total_supply = TOTALSUPPLY.load(storage)?;
    total_supply -= amount;
    TOTALSUPPLY.save(storage, &total_supply)?;

    mason_share -= amount;
    BALANCES.save(storage, sender.clone(), &mason_share)?;
    
    safe_share_transferfrom(storage, querier, env.contract.address,sender,  amount)
}

pub fn try_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
)
    ->Result<Response, ContractError>
{
    let sender = info.sender;
    check_onlyoneblock(deps.storage, Uint128::from(env.block.height as u128), sender.clone())?;
    update_reward(deps.storage, sender.clone())?;

    if amount <= Uint128::zero() {
        return Err(ContractError::ZeroStake{ })
    }
    let msg = _stake(deps.storage, &deps.querier, env, sender.clone(), amount)?;
    let epoch: Uint128 = deps.querier.query_wasm_smart(
        TREASURY.load(deps.storage)?, &TreasuryQuery::Epoch {  })?;
    let mut mason = MASONS.load(deps.storage, sender.clone())?;
    mason.epoch_timer_start = epoch;
    MASONS.save(deps.storage, sender.clone(), &mason)?;

    Ok(Response::new()
        .add_message(msg))
}

pub fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
)
    ->Result<Response, ContractError>
{
    let _info = info.clone();
    let _env = env.clone();
    
    let sender = info.sender;

    check_onlyoneblock(deps.storage, Uint128::from(env.block.height as u128), sender.clone())?;
    check_mason_exists(deps.storage, sender.clone())?;
    update_reward(deps.storage, sender.clone())?;
    if amount <= Uint128::zero() {
        return Err(ContractError::ZeroUnstake{ })
    }

    let epoch: Uint128 = deps.querier.query_wasm_smart(
        TREASURY.load(deps.storage)?, &TreasuryQuery::Epoch {  })?;
    let mason = MASONS.load(deps.storage, sender.clone())?;
    let withdraw_lockup_epochs = WITHDRAW_LOCKUP_EPOCHS.load(deps.storage)?;

    if mason.epoch_timer_start + withdraw_lockup_epochs > epoch {
        return Err(ContractError::StillInLockup {  });    
    }
    
    let mut _deps = deps;
    execute(_deps.branch(), env, _info, ExecuteMsg::ClaimReward {  })?;

    let msg = _withdraw(_deps.storage, &_deps.querier, _env, sender, amount)?;
    Ok(Response::new()
        .add_message(msg))
}

pub fn try_exit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
)
    ->Result<Response, ContractError>
{
    let _info = info.clone();
    let balance = balance_of(deps.storage, info.sender);
    try_withdraw(deps, env, _info, balance)
}

pub fn try_claimreward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
)
    ->Result<Response, ContractError>
{
    let sender = info.sender;
    update_reward(deps.storage, sender.clone())?;
    let mut mason = MASONS.load(deps.storage, sender.clone())?;

    let reward = mason.reward_earned;
    if reward > Uint128::zero() {
        let epoch: Uint128 = deps.querier.query_wasm_smart(
            TREASURY.load(deps.storage)?, &TreasuryQuery::Epoch {  })?;
        let reward_lockup_epochs = REWARD_LOCKUP_EPOCHS.load(deps.storage)?;

        if mason.epoch_timer_start + reward_lockup_epochs > epoch {
            return Err(ContractError::StillInLockup {  });
        }
        mason.epoch_timer_start = epoch;
        mason.reward_earned = Uint128::zero();

        let msg = safe_tomb_transferfrom(deps.storage, &deps.querier, env.contract.address, sender, reward)?;
        return Ok(Response::new()
            .add_message(msg));
    }
    Ok(Response::new())
}

pub fn try_allocate_seigniorage(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
)
    ->Result<Response, ContractError>
{
    let sender = info.sender;
    check_onlyoperator(deps.storage, sender.clone())?;

    if amount <= Uint128::zero() {
        return Err(ContractError::ZeroAllocation {  });
    }
    let total_supply = TOTALSUPPLY.load(deps.storage)?;
    if total_supply <= Uint128::zero() {
        return Err(ContractError::ZeroTotalSupply{ })
    }

    let prev_rps = get_latest_snapshot(deps.storage).reward_per_share;
    let next_rps = prev_rps + (amount * Uint128::from((10u64).pow(18u32)) / total_supply);
    // Create & add new snapshot

    let new_snapshot: MasonrySnapshot = MasonrySnapshot{
        time: Uint128::from(env.block.height as u128),
        reward_received: amount,
        reward_per_share: next_rps
    };
    let mut masonry_history = MASONRY_HISTORY.load(deps.storage)?;
    masonry_history.push(new_snapshot);
    MASONRY_HISTORY.save(deps.storage, &masonry_history)?;

    let msg = safe_tomb_transferfrom(deps.storage, &deps.querier, sender, env.contract.address, amount)?;
    Ok(Response::new()
        .add_message(msg))
}

pub fn try_governance_recover_unsupported(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    amount: Uint128,
    to: Addr
)
    ->Result<Response, ContractError>
{
    check_onlyoperator(deps.storage, info.sender)?;

    let tomb = TOMB.load(deps.storage)?;
    let share = SHARE.load(deps.storage)?;
    if token == tomb || token == share {
        return Err(ContractError::InvalidToken{ })
    }

    let msg = safe_transferfrom(deps.storage, &deps.querier, token, env.contract.address, to, amount)?;
    Ok(Response::new()
        .add_message(msg))
}