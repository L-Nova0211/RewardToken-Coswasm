use cosmwasm_std::{StdResult, StdError, Addr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Operator{
    operator: Addr
}

impl Operator {
    pub fn new(_operator: Addr) -> Operator{
        Operator{
            operator: _operator
        }
    }
    pub fn isOperator(&self, sender: Addr) -> StdResult<bool>{
        if self.operator != sender {
            return Err(StdError::GenericErr{
                msg: "Not Authorized".to_string()
            });
        }
        Ok(true)
    }
}