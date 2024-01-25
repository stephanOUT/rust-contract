use crate::{
    state::{SHARES_BALANCE, SHARES_HOLDERS, SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
    ContractError,
};
use cosmwasm_std::{coins, Addr, BankMsg, Event, StdError, StdResult, Uint128};
use cosmwasm_std::{DepsMut, MessageInfo, Response};

const OUT_DENOM: &str = "inj";

const BASE_SUPPLY: Uint128 = Uint128::new(1);
const TX_AMOUNT_SHARES: Uint128 = Uint128::new(1);

pub fn sell_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    referral: Addr,
) -> Result<Response, ContractError> {
    let validated_referral_address = deps.api.addr_validate(&referral.to_string())?;
    let validated_shares_subject_address = deps.api.addr_validate(&shares_subject.to_string())?;
    let state = STATE.load(deps.storage)?;
    let shares_supply = BASE_SUPPLY
        + SHARES_SUPPLY
            .may_load(deps.storage, &validated_shares_subject_address)?
            .unwrap_or_default();
    let shares_balance = SHARES_BALANCE
        .may_load(
            deps.storage,
            (&info.sender, &validated_shares_subject_address),
        )?
        .unwrap_or_default();
    if shares_supply > BASE_SUPPLY {
        let price = get_price(shares_supply - TX_AMOUNT_SHARES);

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

        if balance >= BASE_SUPPLY {
            SHARES_BALANCE.update(
                deps.storage,
                (&info.sender, &validated_shares_subject_address),
                |balance: Option<Uint128>| -> StdResult<_> {
                    Ok(balance.unwrap_or_default() - TX_AMOUNT_SHARES)
                },
            )?;

            SHARES_SUPPLY.update(
                deps.storage,
                &validated_shares_subject_address,
                |supply: Option<Uint128>| -> StdResult<_> {
                    Ok(supply.unwrap_or_default() - TX_AMOUNT_SHARES)
                },
            )?;

            if balance == BASE_SUPPLY {
                SHARES_HOLDERS.update(
                    deps.storage,
                    &validated_shares_subject_address,
                    |holders: Option<Uint128>| -> StdResult<_> {
                        Ok(holders.unwrap_or_default() - TX_AMOUNT_SHARES)
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
                        .add_attribute("amount", TX_AMOUNT_SHARES)
                        .add_attribute("shares_balance_new", shares_balance - TX_AMOUNT_SHARES)
                        .add_attribute("shares_supply_new", shares_supply - TX_AMOUNT_SHARES)
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
