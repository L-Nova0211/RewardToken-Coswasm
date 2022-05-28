use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("WithdrawLockupEpochs:out of range")]
    OutofRange {},

    #[error("Masonry doesn't exist")]
    MasonryNotExist {},

    #[error("Already Initialized")]
    AlreadyInitialized {},

    #[error("Can't 0 stake")]
    ZeroStake{ },

    #[error("Can't 0 unstake")]
    ZeroUnstake{ },

    #[error("Can't 0 allocation")]
    ZeroAllocation{},

    #[error("Masonry: Cannot allocate when totalSupply is 0")]
    ZeroTotalSupply{},

    #[error("Invalid token transfer")]
    InvalidToken{},

    #[error("Masonry: still in reward lockup")]
    StillInLockup{},

    #[error("ContractGuard: one block, one function")]
    ContractGuard{}
}
