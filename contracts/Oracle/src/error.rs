use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid Token")]
    InvalidToken{},

    #[error("Epoch not started yet")]
    NotStartedYet{},

    #[error("Epoch: only operator allowed for pre-epoch")]
    Unauthorized{},

    #[error("_period: out of range")]
    OutOfRange{},
}
