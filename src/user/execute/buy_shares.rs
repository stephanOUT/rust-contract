use crate::{
    state::{SHARES_BALANCE, SHARES_HOLDERS, SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
    ContractError,
};
use cosmwasm_std::{coins, Addr, BankMsg, Event, StdError, StdResult, Uint128};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

fn increment_share_holders(deps: DepsMut, shares_subject: Addr) -> Result<(), ContractError> {
    SHARES_HOLDERS.update(
        deps.storage,
        &shares_subject,
        |holders: Option<Uint128>| -> StdResult<_> {
            Ok(holders.unwrap_or_default() + Uint128::new(1))
        },
    )?;
    Ok(())
}

pub fn buy_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    referral: Addr,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let shares_supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    let shares_balance = SHARES_BALANCE
        .may_load(deps.storage, (&info.sender, &shares_subject))?
        .unwrap_or_default();

    let shares_holders = SHARES_HOLDERS
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    let price = get_price(shares_supply);

    let protocol_fee = calculate_fee(price, state.protocol_buy_fee_percent);
    let subject_fee = calculate_fee(price, state.subject_buy_fee_percent);
    let referral_fee = calculate_fee(price, state.referral_buy_fee_percent);
    let total = price + protocol_fee + subject_fee + referral_fee;

    // user buying own shares for first time
    if shares_subject == info.sender && shares_supply.is_zero() {
        SHARES_BALANCE.update(
            deps.storage,
            (&info.sender, &shares_subject),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + Uint128::new(1))
            },
        )?;

        SHARES_SUPPLY.update(
            deps.storage,
            &shares_subject,
            |supply: Option<Uint128>| -> StdResult<_> {
                Ok(supply.unwrap_or_default() + Uint128::new(1))
            },
        )?;
        increment_share_holders(deps, shares_subject.clone())?;

        let response = Response::new().add_event(
            Event::new("buy_shares")
                .add_attribute("sender", info.sender)
                .add_attribute("shares_subject", shares_subject)
                .add_attribute("amount", Uint128::new(1))
                .add_attribute("shares_balance_new", shares_balance + Uint128::new(1))
                .add_attribute("shares_supply_new", shares_supply + Uint128::new(1))
                .add_attribute("subject_fees", subject_fee)
                .add_attribute("referral_fees", referral_fee)
                .add_attribute("referral", referral)
                .add_attribute("total", total)
                .add_attribute("funds", Uint128::zero()),
        );
        Ok(response)
    }
    // anyone buying shares
    else if shares_supply > Uint128::zero() {
        assert!(info.funds[0].amount >= total, "Insufficient payment");
        SHARES_BALANCE.update(
            deps.storage,
            (&info.sender, &shares_subject),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default() + Uint128::new(1))
            },
        )?;

        SHARES_SUPPLY.update(
            deps.storage,
            &shares_subject,
            |supply: Option<Uint128>| -> StdResult<_> {
                Ok(supply.unwrap_or_default() + Uint128::new(1))
            },
        )?;

        // If is first buy, add as a holder
        if shares_balance.is_zero() {
            increment_share_holders(deps, shares_subject.clone())?;
        }

        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(),
            amount: coins(protocol_fee.into(), "inj"),
        };

        let subject_fee_result = BankMsg::Send {
            to_address: shares_subject.to_string(),
            amount: coins(subject_fee.into(), "inj"),
        };
        let referral_fee_result = BankMsg::Send {
            to_address: referral.to_string(),
            amount: coins(referral_fee.into(), "inj"),
        };

        let shares_balance_new: Uint128 = shares_balance + Uint128::new(1);

        let return_payment = info.funds[0].amount - total;
        if return_payment > Uint128::zero() {
            let return_payment_result = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(return_payment.into(), "inj"),
            };
            let response = Response::new()
                .add_event(
                    Event::new("buy_shares")
                        .add_attribute("sender", info.sender)
                        .add_attribute("shares_subject", shares_subject)
                        .add_attribute("amount", Uint128::new(1))
                        .add_attribute("shares_balance_new", shares_balance_new)
                        .add_attribute("shares_supply_new", shares_supply + Uint128::new(1))
                        .add_attribute("subject_fees", subject_fee)
                        .add_attribute("referral_fees", referral_fee)
                        .add_attribute("referral", referral)
                        .add_attribute("total", total)
                        .add_attribute("funds", info.funds[0].amount),
                )
                .add_messages([
                    protocol_fee_result,
                    subject_fee_result,
                    referral_fee_result,
                    return_payment_result,
                ]);
            return Ok(response);
        }
        let response = Response::new()
            .add_event(
                Event::new("buy_shares")
                    .add_attribute("sender", info.sender)
                    .add_attribute("shares_subject", shares_subject)
                    .add_attribute("amount", Uint128::new(1))
                    .add_attribute("shares_balance_new", shares_balance_new)
                    .add_attribute("shares_supply_new", shares_supply + Uint128::new(1))
                    .add_attribute("subject_fees", subject_fee)
                    .add_attribute("referral_fees", referral_fee)
                    .add_attribute("referral", referral)
                    .add_attribute("total", total)
                    .add_attribute("funds", info.funds[0].amount),
            )
            .add_messages([protocol_fee_result, subject_fee_result, referral_fee_result]);
        return Ok(response);
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "buy_shares: supply is zero, user must buy own share first",
        )))
    }
}
