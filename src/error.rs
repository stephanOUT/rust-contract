use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Not found")]
    NotFound,
    
    #[error("Serialization")]
    SerializationError,

    #[error("Trading is disabled")]
    TradingIsDisabled {},

    #[error("Buy/Sell quantity limit exceeded")]
    BuySellQuantityLimitExceeded {},

    #[error("Insufficient payment")]
    InsufficientPayment {},
}