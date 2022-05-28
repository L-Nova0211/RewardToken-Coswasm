use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Late")]
    Late {},

    #[error("TombGenesisPool: existing pool?")]
    AlreadyExistingPool {},

    #[error("Withdraw: not good")]
    WithdrawFail {},

    #[error("Token should not be tomb")]
    Tomb {},

    #[error("Token should not be pool token")]
    PoolToken {},

    #[error("User not exist")]
    UserNotExist {},
    
    #[error("Error:{e1}")]
    Testing{e1: String}
}
