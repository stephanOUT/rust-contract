use crate::{
    state::{SHARES_BALANCE, SHARES_SUPPLY, STATE},
    ContractError, msg::GetPriceResponse, util::get_price,
};
use cosmwasm_std::{
    coins, Addr, BankMsg, StdError, StdResult,
    Uint128, Coin,
};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

pub fn sell_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();
    if supply > amount {
        let price = get_price(supply, amount);
        let protocol_fee =
            price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        let subject_fee =
            price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        let balance = SHARES_BALANCE
            .may_load(deps.storage, (&info.sender, &shares_subject))?
            .unwrap_or_default();
        if balance >= amount {
            SHARES_BALANCE.update(
                deps.storage,
                (&info.sender, &shares_subject),
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default() - amount)
                },
            )?;

            SHARES_SUPPLY.update(
                deps.storage,
                &shares_subject,
                |supply: Option<Uint128>| -> StdResult<_> {
                    Ok(supply.unwrap_or_default() - amount)
                },
            )?;

            let total_withdrawal = price - protocol_fee - subject_fee;
            let funds = vec![Coin {
                denom: "inj".to_string(),
                amount: total_withdrawal,
            }];
            let funds_result = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: funds,
            };

            let the_protocol_fee = vec![Coin {
                denom: "inj".to_string(),
                amount: protocol_fee.into(),
            }];
            let protocol_fee_result = BankMsg::Send {
                to_address: state.protocol_fee_destination.to_string(),
                amount: the_protocol_fee,
            };

            let the_subject_fee = vec![Coin {
                denom: "inj".to_string(),
                amount: subject_fee.into(),
            }];
            let subject_fee_result = BankMsg::Send {
                to_address: shares_subject.to_string(),
                amount: the_subject_fee,
            };

            let response = Response::new()
                .add_message(protocol_fee_result)
                .add_message(subject_fee_result)
                .add_message(funds_result);
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