use cosmwasm_std::{Storage, Response, Addr, Uint128, DepsMut, StdResult, WasmMsg, StdError,
        CosmosMsg, to_binary, QuerierWrapper};
use IMasonry::msg::{Masonseat, MasonrySnapshot};
use terraswap::querier::{query_token_balance};
use cw20::{Cw20ExecuteMsg};

use crate::error::ContractError;
use crate::state::{OPERATOR, TOMB, SHARE, TOTALSUPPLY, INITIALIZED, BALANCES, STATUS,
    TREASURY, MASONS, MASONRY_HISTORY, WITHDRAW_LOCKUP_EPOCHS, REWARD_LOCKUP_EPOCHS};
    
pub fn balance_of(storage: &dyn Storage, sender: Addr) -> Uint128{
    BALANCES.load(storage, sender.clone()).unwrap()
}

pub fn check_onlyoperator(storage: &dyn Storage, sender: Addr) -> Result<Response, ContractError> {
    let operator = OPERATOR.load(storage)?;
    if operator != sender {
        return Err(ContractError::Unauthorized{});
    }
    Ok(Response::new())
}
pub fn check_onlyoneblock(storage: &mut dyn Storage, height: Uint128, sender: Addr)
    -> Result<Response, ContractError> 
{
    let status = STATUS.load(storage, (height.u128().into(), sender.clone()))?;
    if status == true {
        return Err(ContractError::ContractGuard{ });
    }

    STATUS.save(storage, (height.u128().into(), sender), &true)?;
    Ok(Response::new())
}

pub fn check_mason_exists(storage: &dyn Storage, sender: Addr) -> Result<Response, ContractError> {
    if balance_of(storage, sender) <= Uint128::zero() {
        return Err(ContractError::MasonryNotExist{})
    }
    Ok(Response::new())
}

pub fn update_reward(storage: &mut dyn Storage, mason: Addr) -> Result<Response, ContractError> {
    if mason != Addr::unchecked("".to_string()) {
        let mut seat: Masonseat = MASONS.load(storage, mason.clone())?;
        seat.reward_earned = earned(storage, mason.clone())?;
        seat.last_snapshot_index = latest_snapshot_index(storage)?;
        MASONS.save(storage, mason, &seat)?;
    }
    Ok(Response::new())
}
pub fn latest_snapshot_index(storage: &dyn Storage) -> StdResult<Uint128>{
    let masonry_history = MASONRY_HISTORY.load(storage)?;
    Ok(Uint128::from((masonry_history.len()-1) as u128))
}
pub fn earned(storage: &dyn Storage, mason: Addr) -> StdResult<Uint128>{
    let latest_rps = get_latest_snapshot(storage).reward_per_share;
    let stored_rps = get_last_snapshot_of(storage, mason.clone()).reward_per_share;
    let balance = balance_of(storage, mason.clone());
    let mason = MASONS.load(storage, mason).unwrap();
    let res = balance * (latest_rps-stored_rps) / Uint128::from((10u64).pow(18u32)) + mason.reward_earned;
    Ok(res)
}
pub fn check_not_initialized(storage: &dyn Storage) -> Result<Response, ContractError> {
    let initialized = INITIALIZED.load(storage)?;
    if initialized {
        return Err(ContractError::AlreadyInitialized{})
    }
    Ok(Response::new())
}
pub fn get_latest_snapshot(storage: &dyn Storage) -> MasonrySnapshot {
    let masonry_history = MASONRY_HISTORY.load(storage).unwrap();
    let len = masonry_history.len();

    masonry_history[len-1].clone()
}

pub fn get_last_snapshot_of(storage: &dyn Storage, mason: Addr) -> MasonrySnapshot {
    let mason = MASONS.load(storage, mason).unwrap();
    let masonry_history = MASONRY_HISTORY.load(storage).unwrap();
    masonry_history[(mason.last_snapshot_index.u128() as usize)].clone()
}
pub fn safe_transferfrom( storage: &dyn Storage, querier: &QuerierWrapper, token: Addr, _from: Addr, _to: Addr, _amount: Uint128) -> StdResult<CosmosMsg> {
    let token_balance = query_token_balance(querier, token.clone(), _from.clone()).unwrap();
    
    if token_balance > Uint128::zero() {
        let mut amount = _amount;
        if _amount > token_balance {
            amount = token_balance;
        }

        let msg_transfer = WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_binary(
                &Cw20ExecuteMsg::TransferFrom {
                    owner: _from.to_string(),
                    recipient: _to.to_string(),
                    amount: amount
                }
            ).unwrap(),
            funds: vec![]
        };
        return Ok(CosmosMsg::Wasm(msg_transfer));
    }

    Err(StdError::GenericErr{
        msg: "transfer failed".to_string()
    })
}
pub fn safe_share_transferfrom( storage: &dyn Storage, querier: &QuerierWrapper, _from: Addr, _to: Addr, _amount: Uint128) -> StdResult<CosmosMsg> {
    let share = SHARE.load(storage).unwrap();

    safe_transferfrom(storage, querier, share, _from, _to, _amount)
}
pub fn safe_tomb_transferfrom(storage: &dyn Storage, querier: &QuerierWrapper, _from: Addr, _to: Addr, _amount: Uint128) -> StdResult<CosmosMsg> {
    let tomb = TOMB.load(storage).unwrap();

    safe_transferfrom(storage, querier, tomb, _from, _to, _amount)
}

