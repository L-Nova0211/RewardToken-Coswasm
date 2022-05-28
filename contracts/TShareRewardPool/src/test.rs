use super::*;
use cosmwasm_std::{from_binary, Uint128, Addr};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};

use crate::contract::{execute, instantiate};
use crate::query::{query};
use crate::msg::{QueryMsg, ExecuteMsg, InstantiateMsg, PoolInfo, UserInfo};

use crate::mock_querier::mock_dependencies;

#[test]
fn workflow(){
    let mut deps = mock_dependencies(&[]);
    
    let msg = InstantiateMsg{
        TSHARE: "tomb".to_string(),
        POOLSTARTTIME: Uint128::from(mock_env().block.time.seconds() + 1000)
    };
//instantiate
    let info = mock_info("admin", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

//add 
    let msg = ExecuteMsg::Add{
        alloc_point: Uint128::from(1u128),
        token: Addr::unchecked("token1"),
        with_update: false,
        last_reward_time: Uint128::zero()
    };

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Add Pool{:?}", res);
//add
    let msg = ExecuteMsg::Add{
        alloc_point: Uint128::from(1u128),
        token: Addr::unchecked("token2"),
        with_update: false,
        last_reward_time: Uint128::zero()
    };
    
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Add Pool{:?}", res);
//set
    let msg = ExecuteMsg::Set{
        pid: Uint128::zero(),
        alloc_point: Uint128::from(200u64)
    };

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Set {:?}", res);
//deposit
    let msg = ExecuteMsg::Deposit{
        pid: Uint128::zero(),
        amount: Uint128::from(10_000u64)
    };

    let info = mock_info("user1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Deposit {:?}", res);
//withdraw
    let msg = ExecuteMsg::Withdraw{
        pid: Uint128::zero(),
        amount: Uint128::from(3_000u64)
    };

    let info = mock_info("user1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Withdraw {:?}", res);
//emergency withdraw
    let msg = ExecuteMsg::EmergencyWithdraw{
        pid: Uint128::zero(),
    };

    let info = mock_info("user1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Emergency Withdraw {:?}", res);
//set operator
    let msg = ExecuteMsg::SetOperator{
        operator: Addr::unchecked("user1".to_string()),
    };

    let info = mock_info("admin", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Set Operator {:?}", res);
//Governance Recover Unsupported
    let msg = ExecuteMsg::GovernanceRecoverUnsupported{
        token: Addr::unchecked("token3".to_string()),
        amount: Uint128::from(10_000u128),
        to: Addr::unchecked("user3".to_string())
    };

    let info = mock_info("user1", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    println!("Governance Recover Unsupported {:?}", res);
// -Get Pool Info-----------------
    let msg = QueryMsg::GetPoolInfo{};
    let pool_info = query(deps.as_ref(), mock_env(), msg).unwrap();

    let res: Vec<PoolInfo> = from_binary(&pool_info).unwrap();
    println!("Pool Info {:?}", res );
    
//get user info
    let msg = QueryMsg::GetUserInfo{
        pid: Uint128::zero(),
        user: Addr::unchecked("user1".to_string())
    };
    let user_info = query(deps.as_ref(), mock_env(), msg).unwrap();

    let res: UserInfo = from_binary(&user_info).unwrap();
    println!("User Info {:?}", res );

}

