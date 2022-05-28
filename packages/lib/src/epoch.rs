use cosmwasm_std::{StdResult, StdError, Addr};
use serde::{Deserialize, Serialize};
use crate::operator::{Operator};

use chrono::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Epoch{
    period: u128,
    start_time: u128,
    last_epoch_time: u128,
    epoch: u128,
    operator: Operator
}

impl Epoch{
    pub fn instantiate(
        mut self,
        _period: u128,
        _start_time: u128,
        _start_epoch: u128,
        _sender: Addr,
    ){
        self.period = _period;
        self.start_time = _start_time;
        self.epoch = _start_epoch;
        self.last_epoch_time = _start_time - _period; 
        self.operator = Operator::new(_sender);
    }

    pub fn checkStartTime(self) -> StdResult<bool> {
        let now: DateTime<Utc> = Utc::now(); 
        if (now.second() as u128) < self.start_time {
            return Err(StdError::GenericErr{
                msg: "Epoch: not started yet".to_string()
            });
        }
        Ok(true)
    }

    pub fn getCurrentEpoch(self) -> u128 {
        return self.epoch;
    }

    pub fn getPeriod(self) -> u128 {
        return self.period;
    }

    pub fn getStartTime(self) -> u128 {
        return self.start_time;
    }

    pub fn getLastEpochTime(self) -> u128 {
        return self.last_epoch_time;
    }

    pub fn nextEpochPoint(self) -> u128 {
        return self.last_epoch_time + self.period;
    }

    /* ========== GOVERNANCE ========== */

    pub fn setPeriod(mut self, _period: u128, sender: Addr) 
        -> StdResult<bool>
    {
        self.operator.isOperator(sender)?;
        if _period < 1 || _period <= 48 * 60 * 60 {
            return Err(StdError::GenericErr{
                msg: "_period: out of range".to_string()
            })
        }
        self.period = _period;
        Ok(true)
    }

    pub fn setEpoch(mut self, _epoch: u128, sender: Addr)
        -> StdResult<bool>
    {
        self.operator.isOperator(sender)?;
        self.epoch = _epoch;
        Ok(true)
    }
}