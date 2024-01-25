use crate::{
    state::{SHARES_BALANCE, SHARES_HOLDERS, SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
    ContractError,
};
use cosmwasm_std::{coins, Addr, BankMsg, Event, StdError, StdResult, Uint128};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

const OUT_DENOM: &str = "inj";

pub fn sell_shares(
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
    if shares_supply > Uint128::new(1) {
        let price = get_price(shares_supply - Uint128::new(1));

        let protocol_fee = calculate_fee(price, state.protocol_sell_fee_percent);
        let subject_fee = calculate_fee(price, state.subject_sell_fee_percent);
        let referral_fee = calculate_fee(price, state.referral_sell_fee_percent);
        let total = price - protocol_fee - subject_fee - referral_fee;

        let balance = SHARES_BALANCE
            .may_load(
                deps.storage,
                (&info.sender, &validated_shares_subject_address),
            )?
            .unwrap_or_default();

        if balance >= Uint128::new(1) {
            SHARES_BALANCE.update(
                deps.storage,
                (&info.sender, &validated_shares_subject_address),
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default() - Uint128::new(1))
                },
            )?;

            SHARES_SUPPLY.update(
                deps.storage,
                &validated_shares_subject_address,
                |supply: Option<Uint128>| -> StdResult<_> {
                    Ok(supply.unwrap_or_default() - Uint128::new(1))
                },
            )?;

            if balance == Uint128::new(1) {
                SHARES_HOLDERS.update(
                    deps.storage,
                    &validated_shares_subject_address,
                    |holders: Option<Uint128>| -> StdResult<_> {
                        Ok(holders.unwrap_or_default() - Uint128::new(1))
                    },
                )?;
            }

            let mut msgs: Vec<BankMsg> = Vec::new();

            if total > Uint128::zero() {
                let funds_result = BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: coins(total.into(), OUT_DENOM),
                };
                msgs.push(funds_result);
            }

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
            let response = Response::new()
                .add_event(
                    Event::new("sell_shares")
                        .add_attribute("sender", info.sender)
                        .add_attribute("shares_subject", validated_shares_subject_address)
                        .add_attribute("amount", Uint128::new(1))
                        .add_attribute("shares_balance_new", shares_balance - Uint128::new(1))
                        .add_attribute("shares_supply_new", shares_supply - Uint128::new(1))
                        .add_attribute("subject_fees", subject_fee)
                        .add_attribute("referral_fees", referral_fee)
                        .add_attribute("referral", validated_referral_address)
                        .add_attribute("total", total),
                )
                .add_messages(msgs);
            return Ok(response);
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
