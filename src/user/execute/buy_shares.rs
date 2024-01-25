use crate::{
    state::{SHARES_BALANCE, SHARES_HOLDERS, SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
    ContractError,
};
use cosmwasm_std::{coins, Addr, BankMsg, Event, StdResult, Uint128};
use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cw_utils::must_pay;

const OUT_DENOM: &str = "inj";

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
    let validated_referral_address = deps.api.addr_validate(&referral.to_string())?;
    let validated_shares_subject_address = deps.api.addr_validate(&shares_subject.to_string())?;

    let state = STATE.load(deps.storage)?;

    let shares_supply = Uint128::new(1)
        + SHARES_SUPPLY
            .may_load(deps.storage, &validated_shares_subject_address)?
            .unwrap_or_default();

    let shares_balance = SHARES_BALANCE
        .may_load(
            deps.storage,
            (&info.sender, &validated_shares_subject_address),
        )?
        .unwrap_or_default();

    let shares_holders = SHARES_HOLDERS
        .may_load(deps.storage, &validated_shares_subject_address)?
        .unwrap_or_default();

    let price = get_price(shares_supply);

    let protocol_fee = calculate_fee(price, state.protocol_buy_fee_percent);
    let subject_fee = calculate_fee(price, state.subject_buy_fee_percent);
    let referral_fee = calculate_fee(price, state.referral_buy_fee_percent);
    let total = price + protocol_fee + subject_fee + referral_fee;

    must_pay(&info, OUT_DENOM).map_err(|_| ContractError::InvalidTokenSentPayment {})?;
    assert!(info.funds[0].amount >= total, "Insufficient payment");
    SHARES_BALANCE.update(
        deps.storage,
        (&info.sender, &validated_shares_subject_address),
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + Uint128::new(1))
        },
    )?;

    SHARES_SUPPLY.update(
        deps.storage,
        &validated_shares_subject_address,
        |supply: Option<Uint128>| -> StdResult<_> {
            Ok(supply.unwrap_or_default() + Uint128::new(1))
        },
    )?;

    // If is first buy, add as a holder
    if shares_balance.is_zero() {
        increment_share_holders(deps, validated_shares_subject_address.clone())?;
    }

    let mut msgs: Vec<BankMsg> = Vec::new();

    if protocol_fee > Uint128::zero() {
        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(),
            amount: coins(protocol_fee.into(), OUT_DENOM),
        };
        msgs.push(protocol_fee_result);
    }

    if subject_fee > Uint128::zero() {
        let subject_fee_result = BankMsg::Send {
            to_address: validated_shares_subject_address.to_string(),
            amount: coins(subject_fee.into(), OUT_DENOM),
        };
        msgs.push(subject_fee_result);
    }

    if referral_fee > Uint128::zero() {
        let referral_fee_result = BankMsg::Send {
            to_address: validated_referral_address.to_string(),
            amount: coins(referral_fee.into(), OUT_DENOM),
        };
        msgs.push(referral_fee_result);
    }

    let shares_balance_new: Uint128 = shares_balance + Uint128::new(1);

    let return_payment = info.funds[0].amount - total;
    if return_payment > Uint128::zero() {
        let return_payment_result = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(return_payment.into(), OUT_DENOM),
        };
        msgs.push(return_payment_result);
    }
    let response = Response::new()
        .add_event(
            Event::new("buy_shares")
                .add_attribute("sender", info.sender)
                .add_attribute("shares_subject", validated_shares_subject_address)
                .add_attribute("amount", Uint128::new(1))
                .add_attribute("shares_balance_new", shares_balance_new)
                .add_attribute("shares_supply_new", shares_supply + Uint128::new(1))
                .add_attribute("subject_fees", subject_fee)
                .add_attribute("referral_fees", referral_fee)
                .add_attribute("referral", validated_referral_address)
                .add_attribute("total", total)
                .add_attribute("funds", info.funds[0].amount),
        )
        .add_messages(msgs);
    return Ok(response);
}
