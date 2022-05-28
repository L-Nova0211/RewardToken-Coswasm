use super::*;
use cosmwasm_std::{from_binary, Uint128, Addr};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};

use crate::contract::{execute, instantiate};
use IMasonry::msg::{QueryMsg, ExecuteMsg, InstantiateMsg };

use crate::mock_querier::mock_dependencies;

#[test]
fn workflow(){
    let mut deps = mock_dependencies(&[]);
 

}

