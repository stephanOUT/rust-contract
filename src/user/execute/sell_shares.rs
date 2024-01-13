use crate::{
    msg::GetPriceResponse,
    state::{SHARES_BALANCE, SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
    ContractError,
};
use cosmwasm_std::{coins, Addr, BankMsg, Coin, Event, StdError, StdResult, Uint128};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn sell_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    amount_of_shares_to_sell: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let shares_supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();
    let shares_balance = SHARES_BALANCE
        .may_load(deps.storage, (&info.sender, &shares_subject))?
        .unwrap_or_default();
    if shares_supply > amount_of_shares_to_sell {
        let price = get_price(
            (shares_supply - amount_of_shares_to_sell),
            amount_of_shares_to_sell,
        );
        println!("Price: {}", price);

        let protocol_fee = calculate_fee(price, state.protocol_fee_percent);
        let subject_fee = calculate_fee(price, state.subject_fee_percent);
        let total = price - protocol_fee - subject_fee;
        println!("subject_fee: {}", subject_fee);
        println!("protocol_fee: {}", protocol_fee);
        println!("total: {}", total);

        let balance = SHARES_BALANCE
            .may_load(deps.storage, (&info.sender, &shares_subject))?
            .unwrap_or_default();

        if balance >= amount_of_shares_to_sell {
            SHARES_BALANCE.update(
                deps.storage,
                (&info.sender, &shares_subject),
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default() - amount_of_shares_to_sell)
                },
            )?;

            SHARES_SUPPLY.update(
                deps.storage,
                &shares_subject,
                |supply: Option<Uint128>| -> StdResult<_> {
                    Ok(supply.unwrap_or_default() - amount_of_shares_to_sell)
                },
            )?;

            let funds_result = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(total.into(), "inj"),
            };

            let protocol_fee_result = BankMsg::Send {
                to_address: state.protocol_fee_destination.to_string(),
                amount: coins(protocol_fee.into(), "inj"),
            };

            let subject_fee_result = BankMsg::Send {
                to_address: shares_subject.to_string(),
                amount: coins(subject_fee.into(), "inj"),
            };

            // let response = Response::new()
            //     .add_message(protocol_fee_result)
            //     .add_message(subject_fee_result)
            //     .add_message(funds_result);

            let response = Response::new()
                .add_event(
                    Event::new("sell_shares")
                        .add_attribute("sender", info.sender)
                        .add_attribute("shares_subject", shares_subject)
                        .add_attribute("amount", amount_of_shares_to_sell)
                        .add_attribute("shares_balance", shares_balance)
                        .add_attribute("shares_supply", shares_supply)
                        .add_attribute("total", total),
                )
                .add_messages([funds_result, protocol_fee_result, subject_fee_result]);

            Ok(response)
        } else {
            Err(ContractError::Std(StdError::generic_err(
                "Insufficient shares",
            )))
        }
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "Cannot sell the last share",
        )))
    }
}
