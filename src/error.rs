use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Trading is disabled")]
    TradingIsDisabled {},

    #[error("Invalid token sent")]
    InvalidTokenSentPayment {},

    #[error("The tradingstate is the same")]
    TradingStateTheSame {},
}