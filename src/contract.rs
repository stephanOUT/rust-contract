#[cfg(not(feature = "library"))]
use std::collections::HashMap;

use crate::{
    msg::{
        ExecuteMsg, GetBuyPriceAfterFeeResponse, GetBuyPriceResponse, GetPriceResponse,
        GetSellPriceAfterFeeResponse, GetSellPriceResponse, GetShareBalance, QueryMsg,
    },
    state::{State, STATE},
    ContractError,
};
use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, StdError, StdResult, Uint128,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use serde_json;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        subject_fee_percent: Uint128::new(5),
        protocol_fee_percent: Uint128::new(5),
        shares_supply: HashMap::<Addr, Uint128>::new(),
        shares_balance: HashMap::<Addr, HashMap<Addr, Uint128>>::new(),
        protocol_fee_destination: info.sender.clone(), // change later
    };

    // Convert HashMaps to a JSON string
    let shares_supply_json = serde_json::to_string(&state.shares_supply).unwrap();
    let shares_balance_json = serde_json::to_string(&state.shares_balance).unwrap();

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("subject_fee_percent", Uint128::new(5))
        .add_attribute("protocol_fee_percent", Uint128::new(5))
        .add_attribute("shares_supply", shares_supply_json)
        .add_attribute("shares_balance", shares_balance_json))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetFeeDestination { fee_destination } => {
            set_fee_destination(deps, info, fee_destination)
        }
        ExecuteMsg::SetProtocolFeePercent {
            protocol_fee_percent,
        } => set_protocol_fee_percent(deps, info, protocol_fee_percent),
        ExecuteMsg::SetSubjectFeePercent {
            subject_fee_percent,
        } => set_subject_fee_percent(deps, info, subject_fee_percent),
        ExecuteMsg::BuyShares {
            shares_subject,
            amount,
        } => buy_shares(deps, info, shares_subject, amount),
        ExecuteMsg::SellShares {
            shares_subject,
            amount,
        } => sell_shares(deps, info, shares_subject, amount),
    }
}

pub fn set_fee_destination(
    deps: DepsMut,
    info: MessageInfo,
    fee_destination: Addr,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_destination = fee_destination;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_fee_destination"))
}

pub fn set_protocol_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_protocol_fee_percent"))
}

pub fn set_subject_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.subject_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_subject_fee_percent"))
}

pub fn buy_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let supply = state
        .shares_supply
        .get(&shares_subject)
        .cloned()
        .unwrap_or_else(|| Uint128::zero());
    if supply > Uint128::zero() || shares_subject == info.sender {
        let price_response: GetPriceResponse = get_price(supply, amount)?;
        let price: Uint128 = price_response.price;
        let protocol_fee =
            price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        let subject_fee =
            price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        assert!(
            info.funds[0].amount >= price + protocol_fee + subject_fee,
            "Insufficient payment"
        );
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            let subject_map = state
                .shares_balance
                .entry(shares_subject.clone())
                .or_insert_with(|| HashMap::new());
            *subject_map
                .entry(info.sender.clone())
                .or_insert(Uint128::zero()) += amount;
            let supply = state
                .shares_supply
                .entry(shares_subject.clone())
                .or_insert(Uint128::zero());
            *supply += amount;
            Ok(state)
        })?;

        let the_protocol_fee = vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: protocol_fee.into(),
        }];
        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(), // why would they make us convert to string?
            amount: the_protocol_fee,
        };

        let the_subject_fee = vec![Coin {
            denom: info.funds[0].denom.clone(),
            amount: subject_fee.into(),
        }];
        let subject_fee_result = BankMsg::Send {
            to_address: shares_subject.to_string(),
            amount: the_subject_fee,
        };

        if info.funds[0].amount > (price + protocol_fee + subject_fee) {
            let amount_back = info.funds[0].amount - price - protocol_fee - subject_fee;
            let the_amount_back = vec![Coin {
                denom: info.funds[0].denom.clone(),
                amount: amount_back.into(),
            }];
            let amount_back_result = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: the_amount_back,
            };
        }

        Ok(Response::default())
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "buy_shares: supply is zero",
        )))
    }
}

pub fn get_price(supply: Uint128, amount: Uint128) -> StdResult<GetPriceResponse> {
    let sum1 = if supply.is_zero() {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1))
            * supply
            * (Uint128::new(2) * (supply - Uint128::new(1)) + Uint128::new(1))
            / Uint128::new(6)
    };

    let sum2 = if supply == Uint128::zero() && amount == Uint128::new(1) {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1) + amount)
            * (supply + amount)
            * (Uint128::new(2) * (supply - Uint128::new(1) + amount) + Uint128::new(1))
            / Uint128::new(6)
    };

    let summation = sum2 - sum1;

    let the_price = summation * Uint128::new(1_000_000_000_000_000_000) / Uint128::new(16000);
    Ok(GetPriceResponse { price: the_price })
}

pub fn sell_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares_subject: Addr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let supply = state
        .shares_supply
        .get(&shares_subject)
        .cloned()
        .unwrap_or_else(|| Uint128::zero());
    if supply > amount {
        let price_response: GetPriceResponse = get_price(supply, amount)?;
        let price: Uint128 = price_response.price;
        let protocol_fee =
            price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        let subject_fee =
            price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
        let balance = state
            .shares_balance
            .get(&shares_subject)
            .and_then(|balances| balances.get(&info.sender).copied())
            .unwrap_or(Uint128::zero());
        if balance >= amount {
            STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                let subject_map = state
                    .shares_balance
                    .entry(shares_subject.clone())
                    .or_insert_with(|| HashMap::new());
                *subject_map
                    .entry(info.sender.clone())
                    .or_insert(Uint128::zero()) -= amount;
                let supply = state
                    .shares_supply
                    .entry(shares_subject.clone())
                    .or_insert(Uint128::zero());
                *supply -= amount;
                Ok(state)
            })?;

            let total_withdrawal = price - protocol_fee - subject_fee;
            let funds = vec![Coin {
                denom: info.funds[0].denom.clone(),
                amount: total_withdrawal,
            }];
            let subject_fee_result = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: funds,
            };

            let the_protocol_fee = vec![Coin {
                denom: info.funds[0].denom.clone(),
                amount: protocol_fee.into(),
            }];
            let protocol_fee_result = BankMsg::Send {
                to_address: state.protocol_fee_destination.to_string(),
                amount: the_protocol_fee,
            };

            let the_subject_fee = vec![Coin {
                denom: info.funds[0].denom.clone(),
                amount: subject_fee.into(),
            }];
            let subject_fee_result = BankMsg::Send {
                to_address: shares_subject.to_string(),
                amount: the_subject_fee,
            };

            Ok(Response::default())
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { supply, amount } => to_binary(&get_price(supply, amount)?),
        QueryMsg::GetBuyPrice {
            shares_subject,
            amount,
        } => to_binary(&get_buy_price(deps, shares_subject, amount)?),
        QueryMsg::GetSellPrice {
            shares_subject,
            amount,
        } => to_binary(&get_sell_price(deps, shares_subject, amount)?),
        QueryMsg::GetBuyPriceAfterFee {
            shares_subject,
            amount,
        } => to_binary(&get_buy_price_after_fee(deps, shares_subject, amount)?),
        QueryMsg::GetSellPriceAfterFee {
            shares_subject,
            amount,
        } => to_binary(&get_sell_price_after_fee(deps, shares_subject, amount)?),
        QueryMsg::GetShareBalance {
            shares_subject,
            my_address,
        } => to_binary(&get_share_balance(deps, my_address)?),
        QueryMsg::GetState {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state)
        }
    }
}

pub fn get_buy_price(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetBuyPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let buy_price_response = get_price(
        state
            .shares_supply
            .get(&shares_subject)
            .copied()
            .unwrap_or_else(|| Uint128::zero()),
        amount,
    )?;
    let buy_price: Uint128 = buy_price_response.price;
    Ok(GetBuyPriceResponse { price: buy_price })
}

pub fn get_sell_price(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetSellPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let sell_price_response: GetPriceResponse = get_price(
        state
            .shares_supply
            .get(&shares_subject)
            .copied()
            .unwrap_or_else(|| Uint128::zero())
            - amount,
        amount,
    )?;
    let sell_price: Uint128 = sell_price_response.price;
    Ok(GetSellPriceResponse { price: sell_price })
}

pub fn get_buy_price_after_fee(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetBuyPriceAfterFeeResponse> {
    let state = STATE.load(deps.storage)?;
    let price_response: GetBuyPriceResponse = get_buy_price(deps, shares_subject, amount)?;
    let price: Uint128 = price_response.price;
    let protocol_fee = price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
    let subject_fee = price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
    let return_price = price + protocol_fee + subject_fee;
    Ok(GetBuyPriceAfterFeeResponse {
        price: return_price,
    })
}

pub fn get_sell_price_after_fee(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetSellPriceAfterFeeResponse> {
    let state = STATE.load(deps.storage)?;
    let price_response: GetSellPriceResponse = get_sell_price(deps, shares_subject, amount)?;
    let price: Uint128 = price_response.price;
    let protocol_fee = price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
    let subject_fee = price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
    let return_price = price - protocol_fee - subject_fee;
    Ok(GetSellPriceAfterFeeResponse {
        price: return_price,
    })
}

pub fn get_share_balance(deps: Deps, my_address: Addr) -> StdResult<GetShareBalance> {
    let state = STATE.load(deps.storage)?;

    Ok(GetShareBalance {
        amount: state
            .shares_balance
            .get(&my_address)
            .map(|balance_map| {
                balance_map
                    .values()
                    .fold(Uint128::zero(), |acc, &balance| acc + balance)
            })
            .unwrap_or_default(),
    })
}
