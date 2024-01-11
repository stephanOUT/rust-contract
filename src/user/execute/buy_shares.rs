use crate::{
    state::{SHARES_BALANCE, SHARES_SUPPLY, STATE},
    ContractError,
    util::{calculate_fee, get_price},
};
use cosmwasm_std::{
    coins, Addr, BankMsg, StdError, StdResult,
    Uint128,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn buy_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let shares_supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    let price = get_price(shares_supply, amount);
    println!("Price: {}", price);

    let protocol_fee = calculate_fee(price, state.protocol_fee_percent);
    let subject_fee = calculate_fee(price, state.subject_fee_percent);
    let total = price + protocol_fee + subject_fee;
    println!("subject_fee: {}", subject_fee);
    println!("protocol_fee: {}", protocol_fee);
    println!("total: {}", total);

    // user buying own shares for first time
    if shares_subject == info.sender && shares_supply.is_zero() {
        SHARES_BALANCE.update(
            deps.storage,
            (&info.sender, &shares_subject),
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        SHARES_SUPPLY.update(
            deps.storage,
            &shares_subject,
            |supply: Option<Uint128>| -> StdResult<_> { Ok(supply.unwrap_or_default() + amount) },
        )?;

        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(),
            amount: vec![],
        };

        let subject_fee_result = BankMsg::Send {
            to_address: shares_subject.to_string(),
            amount: vec![],
        };

        let response = Response::new()
            .add_message(protocol_fee_result)
            .add_message(subject_fee_result);
        Ok(response)
    }
    // anyone buying shares
    else if shares_supply > Uint128::zero() {
        assert!(info.funds[0].amount >= total, "Insufficient payment");
        SHARES_BALANCE.update(
            deps.storage,
            (&info.sender, &shares_subject),
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        SHARES_SUPPLY.update(
            deps.storage,
            &shares_subject,
            |supply: Option<Uint128>| -> StdResult<_> { Ok(supply.unwrap_or_default() + amount) },
        )?;

        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(),
            amount: coins(protocol_fee.into(), "inj"),
        };

        let subject_fee_result = BankMsg::Send {
            to_address: shares_subject.to_string(),
            amount: coins(subject_fee.into(), "inj"),
        };

        let response = Response::new()
            .add_message(protocol_fee_result)
            .add_message(subject_fee_result);
        Ok(response)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "buy_shares: supply is zero, user must buy own share first",
        )))
    }
}