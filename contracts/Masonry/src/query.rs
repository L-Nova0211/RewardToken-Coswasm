#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, Env, StdResult, Addr,
    Uint128
};

use IMasonry::msg::{QueryMsg, Masonseat};
use crate::state::{OPERATOR, TOMB, SHARE, TOTALSUPPLY, INITIALIZED, BALANCES,
    TREASURY, MASONS, MASONRY_HISTORY, WITHDRAW_LOCKUP_EPOCHS, REWARD_LOCKUP_EPOCHS};
use crate::util::{get_latest_snapshot, latest_snapshot_index, balance_of, earned};
use Treasury::msg::{QueryMsg as TreasuryQuery};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Operator{ } =>{
            to_binary(&OPERATOR.load(deps.storage)?)
        }

        QueryMsg::LatestSnapshotIndex{ } => {
            to_binary(&latest_snapshot_index(deps.storage)?)
        },

        QueryMsg::GetLastSnapshotIndexOf{ mason } => {
            let _mason = MASONS.load(deps.storage, mason)?;
            to_binary(&_mason.last_snapshot_index)
        },

        QueryMsg::CanWithdraw{ mason } => {
            let _mason = MASONS.load(deps.storage, mason)?;
            let withdraw_lockup_epochs = WITHDRAW_LOCKUP_EPOCHS.load(deps.storage)?;
            let treasury = TREASURY.load(deps.storage)?;

            let epoch: Uint128 = deps.querier.query_wasm_smart(
                treasury, &TreasuryQuery::Epoch {  })?;

            let res = (_mason.epoch_timer_start + withdraw_lockup_epochs) <= epoch;
            to_binary(&res)
        },

        QueryMsg::CanClaimReward{ mason } => {
            let _mason = MASONS.load(deps.storage, mason)?;
            let reward_lockup_epochs = REWARD_LOCKUP_EPOCHS.load(deps.storage)?;
            let treasury = TREASURY.load(deps.storage)?;

            let epoch: Uint128 = deps.querier.query_wasm_smart(
                treasury, &TreasuryQuery::Epoch {  })?;

            let res = (_mason.epoch_timer_start + reward_lockup_epochs) <= epoch;
            to_binary(&res)
        },

        QueryMsg::Epoch{ } => {
            let epoch: Uint128 = deps.querier.query_wasm_smart(
                TREASURY.load(deps.storage)?, &TreasuryQuery::Epoch {  })?;
            to_binary(&epoch)
        },

        QueryMsg::NextEpochPoint{ } => {
            let next_epoch_point: Uint128 = deps.querier.query_wasm_smart(
                TREASURY.load(deps.storage)?, &TreasuryQuery::NextEpochPoint {  })?;
            to_binary(&next_epoch_point)
        },

        QueryMsg::GetTombPrice{ } => {
            let tomb_price: Uint128 = deps.querier.query_wasm_smart(
                TREASURY.load(deps.storage)?, &TreasuryQuery::GetTombPrice {  })?;
            to_binary(&tomb_price)
        },

        QueryMsg::RewardPerShare{ } => {
            to_binary(&(get_latest_snapshot(deps.storage).reward_per_share))
        },

        QueryMsg::Earned{ mason } => {
            to_binary(&earned(deps.storage, mason)?)
        }
    }
}
