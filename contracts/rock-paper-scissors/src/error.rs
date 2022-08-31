use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Must include wager amount")]
    MissingWagerAmount {},

    #[error("Host and opponent addresses cannot be the same: {val:?}")]
    HostAndOpponentCannotBeTheSame { val: String },

    #[error("Host and opponent can only have one active game at a time")]
    ActiveGameAlreadyExists {},

    #[error("Your wager amount must match the host's wager amount")]
    InsufficientWagerAmount {},

    #[error("Game between host and opponent could not be found")]
    GameNotFound {},
}
