#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, QuerierWrapper,
    Uint128, CosmosMsg, WasmMsg, Storage
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, UserInfo, PoolInfo};
use crate::state::{OPERATOR, TOMB, POOLINFO, USERINFO, TOTALALLOCPOINT, 
    POOLSTARTTIME, EPOCHENDTIMES, EPOCHTOMBPERSECOND};

// version info for migration info
const CONTRACT_NAME: &str = "TombGenesisRewardPool";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const EPOCHTOTALREWARDS:[u128;2] = [80_000_000_000_000_000_000_000u128, 
                                        60_000_000_000_000_000_000_000u128];
pub const ETHER: u128 = 1_000_000_000_000_000_000u128;
pub const DAY: u128 = 86_400; //1 day

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.POOLSTARTTIME < Uint128::from(env.block.time.seconds()){
        return Err(ContractError::Late{})
    }
    
    TOMB.save(deps.storage, &deps.api.addr_validate(msg.TOMB.as_str())?)?;
    POOLSTARTTIME.save(deps.storage, &msg.POOLSTARTTIME)?;

    let epoch_end_time = vec![
        msg.POOLSTARTTIME + Uint128::from(4 * DAY),
        msg.POOLSTARTTIME + Uint128::from(9 * DAY)
    ];
    EPOCHENDTIMES.save(deps.storage, &epoch_end_time)?;

    let epoch_tomb_per_second = vec![
        Uint128::from(EPOCHTOTALREWARDS[0] / (4 * DAY)),
        Uint128::from(EPOCHTOTALREWARDS[1] / (4 * DAY)),
        Uint128::zero()
    ];
    EPOCHTOMBPERSECOND.save(deps.storage, &epoch_tomb_per_second)?;

    OPERATOR.save(deps.storage, &info.sender)?;

    POOLINFO.save(deps.storage, &Vec::new())?;
    TOTALALLOCPOINT.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

pub fn balance_of(querier: QuerierWrapper, _token: &Addr, _address: &Addr) -> u128 {
    let token_balance: Cw20BalanceResponse = querier.query_wasm_smart(
        _token,
        &Cw20QueryMsg::Balance{
            address: _address.to_string(),
        }
    ).unwrap();
    token_balance.balance.u128()
}
fn check_pool_duplicate(deps: &DepsMut, _token: Addr) -> bool {
    let pool_info: Vec<PoolInfo> = POOLINFO.load(deps.storage).unwrap();
    let length = pool_info.len();

    if length > 0{
        for pid in 0 .. length  {
            if pool_info[pid].token == _token{
                return true;
            }
        }
    }
    false
}
fn mass_update_pools(deps: DepsMut, env: Env) {
    let pool_info: Vec<PoolInfo> = POOLINFO.load(deps.storage).unwrap();
    let length = pool_info.len();

    let mut _deps = deps;
    for pid in 0 .. length {
        update_pool(_deps.branch(), &env, pid);
    }
}
fn update_pool(deps: DepsMut, env: &Env, _pid: usize) {
    let mut pool_info: Vec<PoolInfo> = POOLINFO.load(deps.storage).unwrap();
    let mut pool = &mut pool_info[_pid];
    let blocktime = Uint128::from(env.block.time.seconds());

    if blocktime <= pool.lastRewardTime {
        return;
    }

    let token_supply: u128 = balance_of(deps.querier, &pool.token, &env.contract.address);
    if token_supply == 0 {
        pool.lastRewardTime = blocktime;
        return;
    }

    let mut total_alloc_point = TOTALALLOCPOINT.load(deps.storage).unwrap();
    if !pool.isStarted {
        pool.isStarted = true;
        total_alloc_point = total_alloc_point + pool.allocPoint;
        TOTALALLOCPOINT.save(deps.storage, &total_alloc_point).unwrap();
    }

    if total_alloc_point > Uint128::zero() {
        let generated_reward = get_generated_reward(deps.storage, pool.lastRewardTime, blocktime);
        let tomb_reward = Uint128::from(generated_reward) * pool.allocPoint / total_alloc_point;
        pool.accTombPerShare += tomb_reward * Uint128::from(ETHER) / Uint128::from(token_supply);
    }
    pool.lastRewardTime = blocktime;
    POOLINFO.save(deps.storage, &pool_info).unwrap();
}
pub fn get_generated_reward(storage: &dyn Storage, from_time: Uint128, to_time: Uint128) -> u128 {
    let epoch_end_time = EPOCHENDTIMES.load(storage).unwrap();
    let epoch_tomb_per_second = EPOCHTOMBPERSECOND.load(storage).unwrap();

    for epoch_id in (1..=2).rev() {
        if to_time >= epoch_end_time[epoch_id - 1] {
            if from_time >= epoch_end_time[epoch_id - 1] {
                return ((to_time - from_time) * (epoch_tomb_per_second[epoch_id])).u128();
            }

            let mut generated_reward = (to_time - epoch_end_time[epoch_id - 1]) * epoch_tomb_per_second[epoch_id];
            if epoch_id == 1 {
                return (generated_reward + ((epoch_end_time[0]-from_time) * epoch_tomb_per_second[0])).u128();
            }
            for epoch_id in (1..=epoch_id-1).rev(){
                if from_time >= epoch_end_time[epoch_id - 1] {
                    return (generated_reward + ((epoch_end_time[epoch_id]-from_time) * epoch_tomb_per_second[epoch_id])).u128();
                }
                generated_reward +=  
                    (epoch_end_time[epoch_id] - epoch_end_time[epoch_id - 1]) * epoch_tomb_per_second[epoch_id];
            }
            return (generated_reward + (epoch_end_time[0]-from_time) * epoch_tomb_per_second[0]).u128();
        }
        return ((to_time - from_time) * epoch_tomb_per_second[0]).u128();
    }
    0u128
}

fn safe_tomb_transfer( deps: DepsMut, env: Env, _to: Addr, _amount: Uint128) -> Option<CosmosMsg> {
    let tomb = TOMB.load(deps.storage).unwrap();
    let tomb_balance = balance_of(deps.querier, &tomb, &env.contract.address);
    
    if tomb_balance > 0 {
        let mut amount = _amount;
        if _amount > Uint128::from(tomb_balance) {
            amount = Uint128::from(tomb_balance);
        }

        let msg_transfer = WasmMsg::Execute {
            contract_addr: tomb.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Transfer {
                    recipient: _to.to_string(),
                    amount: amount
                }
            ).unwrap(),
            funds: vec![]
        };
        return Some(CosmosMsg::Wasm(msg_transfer));
    }

    None
}
fn get_user(storage: &mut dyn Storage, pid: Uint128, sender: Addr, can_register: bool) 
    -> Result<UserInfo, ContractError>
{
    let user: UserInfo;
    let key = (pid.u128().into(), &sender);
    let res = USERINFO.may_load(storage, key.clone());

    if res == Ok(None){ //not exist
        user = UserInfo{
            amount: Uint128::zero(),
            rewardDebt: Uint128::zero()
        };

        if can_register == true {
            USERINFO.save(storage, key, &user)?;
        }
        else{
            return Err(ContractError::UserNotExist{ });
        }
    } else{
        user = USERINFO.load(storage, key)?;
    }
    Ok(user)
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Add{ alloc_point, token, with_update, last_reward_time}
            => try_add(deps, env, info, alloc_point, token, with_update, last_reward_time ),

        ExecuteMsg::Set{ pid, alloc_point}
            => try_set(deps, env, pid, alloc_point ),

        ExecuteMsg::MassUpdatePools{ }
            => { 
                mass_update_pools(deps, env);
                Ok(Response::new())
            },

        ExecuteMsg::UpdatePool{ pid }
            => {
                update_pool(deps, &env, pid.u128() as usize);
                Ok(Response::new())
            },
        
        ExecuteMsg::Deposit{ pid, amount }
            => try_deposit(deps, env, info, pid, amount),

        ExecuteMsg::Withdraw{ pid, amount }
            => try_withdraw(deps, env, info, pid, amount),

        ExecuteMsg::EmergencyWithdraw{ pid }
            => try_emergency_withdraw(deps, info, pid),

        ExecuteMsg::SetOperator{ operator }
            => try_setoperator(deps, info, operator),

        ExecuteMsg::GovernanceRecoverUnsupported{ token, amount, to }
            => try_governance_recover_unsupported(deps, env, info, token, amount, to),
    }
}
pub fn try_governance_recover_unsupported(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    amount: Uint128,
    to: Addr
)
    -> Result<Response, ContractError>
{
    let operator = OPERATOR.load(deps.storage)?;
    if operator != info.sender {
        return Err(ContractError::Unauthorized{ });
    }

    let epoch_end_times = EPOCHENDTIMES.load(deps.storage)?;
    
    if Uint128::from(env.block.time.seconds()) < epoch_end_times[1] + Uint128::from(90 * DAY) {
        // do not allow to drain core token (TOMB or lps) if less than 90 days after pool ends
        let tomb = TOMB.load(deps.storage)?;
        if token == tomb {
            return Err(ContractError::Tomb{ });
        }

        let pool_info = POOLINFO.load(deps.storage)?;
        let length = pool_info.len();
        for pid in 0 .. length {
            let pool = &pool_info[pid];
            if token == pool.token{
                return Err(ContractError::PoolToken{ })
            }
        }
    }

    let msg_transfer = WasmMsg::Execute {
        contract_addr: token.to_string(),
        msg: to_binary(
            &Cw20ExecuteMsg::Transfer {
                recipient: to.to_string(),
                amount: amount
            }
        ).unwrap(),
        funds: vec![]
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(msg_transfer))
        .add_attribute("action", "Governance recover unsupported"))
}
pub fn try_setoperator(
    deps: DepsMut,
    info: MessageInfo,
    operator: Addr
)
    -> Result<Response, ContractError>
{
    let _operator = OPERATOR.load(deps.storage)?;
    if _operator != info.sender {
        return Err(ContractError::Unauthorized{ });
    }
    OPERATOR.save(deps.storage, &operator)?;
    Ok(Response::new()
        .add_attribute("action", "Set Operator"))
}
pub fn try_emergency_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    pid: Uint128,
)
    -> Result<Response, ContractError>
{
    let _sender = info.sender;
    let pool_info = &mut POOLINFO.load(deps.storage)?;
    let pool = &mut pool_info[pid.u128() as usize];

    let mut user = get_user(deps.storage, pid.u128().into(), _sender.clone(), false)?;

    let amount: Uint128 = user.amount;
    user.amount = Uint128::zero();
    user.rewardDebt = Uint128::zero();

    let mut msgs: Vec<CosmosMsg> = vec![];
    if amount > Uint128::zero() {
        let msg_transfer_from = WasmMsg::Execute {
            contract_addr: pool.token.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Transfer {
                    recipient: _sender.to_string(),
                    amount: amount
                }
            ).unwrap(),
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(msg_transfer_from));
    }

    USERINFO.save(deps.storage, (pid.u128().into(), &_sender), &user)?;
    Ok(Response::new()
        .add_attribute("action", "emergency withdraw"))
}
pub fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pid: Uint128,
    amount: Uint128
)
    -> Result<Response, ContractError>
{
    let _sender = info.sender;
    let pool_info = &mut POOLINFO.load(deps.storage)?;
    let pool = &mut pool_info[pid.u128() as usize];

    let mut user = get_user(deps.storage, pid, _sender.clone(), false)?;
    if user.amount < amount {
        return Err(ContractError::WithdrawFail{})
    }
    let mut _deps = deps;
    update_pool(_deps.branch(), &env, pid.u128() as usize);

    let mut msgs: Vec<CosmosMsg> = vec![];
    let _pending = user.amount * pool.accTombPerShare / Uint128::from(ETHER) - user.rewardDebt;
    if _pending > Uint128::zero() {
        let msg = safe_tomb_transfer(_deps.branch(), env.clone(), _sender.clone(), _pending);
        if msg != None {
            msgs.push(msg.unwrap());
        }
    }

    if amount > Uint128::zero() {
        user.amount = user.amount - amount;
        
        let msg_transfer_from = WasmMsg::Execute {
            contract_addr: pool.token.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::Transfer {
                    recipient: _sender.to_string(),
                    amount: amount
                }
            ).unwrap(),
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(msg_transfer_from));
    }

    user.rewardDebt = user.amount * pool.accTombPerShare / Uint128::from(ETHER);
    
    USERINFO.save(_deps.storage, (pid.u128().into(), &_sender), &user)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "withdraw"))
}

pub fn try_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pid: Uint128,
    amount: Uint128
)
    -> Result<Response, ContractError>
{
    let _sender = info.sender;
    let pool_info = &mut POOLINFO.load(deps.storage)?;
    let pool = &mut pool_info[pid.u128() as usize];

    let mut user: UserInfo = get_user(deps.storage, pid, _sender.clone(), true)?;

    let mut _deps = deps;
    update_pool(_deps.branch(), &env, pid.u128() as usize);
    let mut msgs: Vec<CosmosMsg> = vec![];
    if user.amount > Uint128::zero() {
        let _pending = user.amount * pool.accTombPerShare / Uint128::from(ETHER) - user.rewardDebt;
        if _pending > Uint128::zero() {
            let msg = safe_tomb_transfer(_deps.branch(), env.clone(), _sender.clone(), _pending);
            if msg != None {
                msgs.push(msg.unwrap());
            }
        }
    }

    if amount > Uint128::zero() {
        let msg_transfer_from = WasmMsg::Execute {
            contract_addr: pool.token.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::TransferFrom {
                    owner: _sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: amount
                }
            ).unwrap(),
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(msg_transfer_from));

        user.amount += amount;
    }
    user.rewardDebt = user.amount * pool.accTombPerShare / Uint128::from(ETHER);

    USERINFO.save(_deps.storage, (pid.u128().into(), &_sender), &user)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "deposit"))
}
pub fn try_set(
    deps: DepsMut,
    env: Env,
    pid: Uint128,
    alloc_point: Uint128
)
    -> Result<Response, ContractError>
{
    let mut _deps = deps;
    mass_update_pools(_deps.branch(), env);

    let mut pool_info = POOLINFO.load(_deps.storage)?;
    let mut pool = &mut pool_info[pid.u128() as usize];
    if pool.isStarted == true {
        let mut total_alloc_point = TOTALALLOCPOINT.load(_deps.storage)?;
        total_alloc_point = total_alloc_point - pool.allocPoint + alloc_point;
        TOTALALLOCPOINT.save(_deps.storage, &total_alloc_point)?;
    }
    pool.allocPoint = alloc_point;

    POOLINFO.save(_deps.storage, &pool_info)?;
    Ok(Response::new()
        .add_attribute("action", "set"))
}
pub fn try_add(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo,
    alloc_point: Uint128,
    token: Addr,
    with_update: bool,
    last_reward_time: Uint128
) 
    -> Result<Response, ContractError>
{
    let operator = OPERATOR.load(deps.storage)?;
    if operator != info.sender {
        return Err(ContractError::Unauthorized{ });
    }

    if check_pool_duplicate(&deps, token.clone()) == true {
        return Err(ContractError::AlreadyExistingPool {})
    }
    
    let mut _deps = deps; 
    if with_update == true {
        mass_update_pools(_deps.branch(), env.clone());
    }

    let pool_start_time = POOLSTARTTIME.load(_deps.storage)?;
    let mut _last_reward_time = last_reward_time;
    let blocktime = Uint128::from(env.block.time.seconds());

    if blocktime < pool_start_time {
        // chef is sleeping
        if last_reward_time == Uint128::zero() {
            _last_reward_time = pool_start_time;
        } else {
            if last_reward_time < pool_start_time {
                _last_reward_time = pool_start_time;
            }
        }
    } else {
        // chef is cooking
        if last_reward_time == Uint128::zero() || last_reward_time < blocktime {
            _last_reward_time = blocktime;
        }
    }

    let is_started: bool =
            (last_reward_time <= pool_start_time) ||
            (last_reward_time <= blocktime);

    let mut pool_info = POOLINFO.load(_deps.storage)?;
    pool_info.push(PoolInfo{
        token : token,
        allocPoint : alloc_point,
        lastRewardTime : _last_reward_time,
        accTombPerShare : Uint128::zero(),
        isStarted : is_started
        });

    if is_started == true {
        let mut total_alloc_point = TOTALALLOCPOINT.load(_deps.storage)?;
        total_alloc_point = total_alloc_point + alloc_point;
        TOTALALLOCPOINT.save(_deps.storage, &total_alloc_point)?;
    }
    POOLINFO.save(_deps.storage, &pool_info)?;

    Ok(Response::new()
        .add_attribute("action", "add")
    )                                
}

