#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::collections::HashMap;
use cosmwasm_std::{BankMsg, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128, Event, StdError, Coin};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetBuyPriceResponse, GetSellPriceResponse, GetPriceResponse, GetBuyPriceAfterFeeResponse, GetSellPriceAfterFeeResponse, GetShareBalance};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /*let mut state = State {
        owner: info.sender.clone(),
        subjectFeePercent: Uint128::new(5),
        protocolFeePercent: Uint128::new(5),
        protocolFeeDestination: info.sender.clone(),
        shares_balance: HashMap::new(),
        sharesSupply: HashMap::new(),
    };
    let subject_addr = Addr::unchecked("subject_address");
    state.shares_balance.entry(subject_addr.clone()).or_insert_with(HashMap::new);
    state.owner = info.sender.clone();
    state.subjectFeePercent = Uint128::new(5);
    state.protocolFeePercent = Uint128::new(5);
    state.protocolFeeDestination = info.sender.clone();
    state.sharesSupply = HashMap::new();*/

    let mut state = State::default();
    state.owner = info.sender.clone();
    state.protocolFeeDestination = info.sender.clone();
    state.subjectFeePercent = Uint128::new(5);
    state.protocolFeePercent = Uint128::new(5);


    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetFeeDestination { fee_destination } => set_fee_destination(deps, info, fee_destination),
        ExecuteMsg::SetProtocolFeePercent { protocol_fee_percent } => set_protocol_fee_percent(deps, info, protocol_fee_percent),
        ExecuteMsg::SetSubjectFeePercent { subject_fee_percent } => set_subject_fee_percent(deps, info, subject_fee_percent),
        ExecuteMsg::BuyShares { shares_subject, amount } => buy_shares(deps, _env, info, shares_subject, amount),
        ExecuteMsg::SellShares { shares_subject, amount } => sell_shares(deps, _env, info, shares_subject, amount),
    }
}

pub fn set_fee_destination(deps: DepsMut, info: MessageInfo, fee_destination: Addr) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocolFeeDestination = fee_destination;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_fee_destination"))
}

pub fn set_protocol_fee_percent(deps: DepsMut, info: MessageInfo, fee_percent: Uint128) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocolFeePercent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_protocol_fee_percent"))
}

pub fn set_subject_fee_percent(deps: DepsMut, info: MessageInfo, fee_percent: Uint128) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.subjectFeePercent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_subject_fee_percent"))
}

pub fn buy_shares(deps: DepsMut, env: Env, info: MessageInfo, shares_subject: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let supply = state.sharesSupply.get(&shares_subject).cloned().unwrap_or_else(|| Uint128::zero());
    if supply > Uint128::zero() || shares_subject == info.sender {
        let price_response: GetPriceResponse = get_price(deps.as_ref(), supply, amount)?;
        let price: Uint128 = price_response.price;
        let protocol_fee = price * state.protocolFeePercent / Uint128::new(1_000_000_000_000_000_000);
        let subject_fee = price * state.subjectFeePercent / Uint128::new(1_000_000_000_000_000_000);
        assert!(
                info.funds[0].amount >= price + protocol_fee + subject_fee,
                "Insufficient payment"
        );
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                let subject_map = state.shares_balance.entry(shares_subject.clone()).or_insert_with(|| HashMap::new());
                *subject_map.entry(info.sender.clone()).or_insert(Uint128::zero()) += amount;
                let supply = state.sharesSupply.entry(shares_subject.clone()).or_insert(Uint128::zero());
                *supply += amount;
                Ok(state)
        })?;

        let the_protocol_fee = vec![Coin {
                denom: info.funds[0].denom.clone(),
                amount: protocol_fee.into(),
        }];
        let protocol_fee_result = BankMsg::Send {
                to_address: state.protocolFeeDestination.to_string(),
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
        Err(ContractError::Std(StdError::generic_err("Some error")))
    }
}

pub fn sell_shares(deps: DepsMut, env: Env, info: MessageInfo, shares_subject: Addr, amount: Uint128) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let supply = state.sharesSupply.get(&shares_subject).cloned().unwrap_or_else(|| Uint128::zero());
    if supply > amount {
        let price_response: GetPriceResponse = get_price(deps.as_ref(), supply, amount)?;
        let price: Uint128 = price_response.price;
        let protocol_fee = price * state.protocolFeePercent / Uint128::new(1_000_000_000_000_000_000);
        let subject_fee = price * state.subjectFeePercent / Uint128::new(1_000_000_000_000_000_000);
        let balance = state.shares_balance.get(&shares_subject).and_then(|balances| balances.get(&info.sender).copied()).unwrap_or(Uint128::zero());
        if balance >= amount {
            STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
                let subject_map = state.shares_balance.entry(shares_subject.clone()).or_insert_with(|| HashMap::new());
                *subject_map.entry(info.sender.clone()).or_insert(Uint128::zero()) -= amount;
                let supply = state.sharesSupply.entry(shares_subject.clone()).or_insert(Uint128::zero());
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
                to_address: state.protocolFeeDestination.to_string(),
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
            Err(ContractError::Std(StdError::generic_err("Insufficient shares")))
        }
    } else {
        Err(ContractError::Std(StdError::generic_err("Cannot sell the last share")))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { supply, amount } => to_binary(&get_price(deps, supply, amount)?),
        QueryMsg::GetBuyPrice { shares_subject, amount } => to_binary(&get_buy_price(deps, shares_subject, amount)?),
        QueryMsg::GetSellPrice { shares_subject, amount } => to_binary(&get_sell_price(deps, shares_subject, amount)?),
        QueryMsg::GetBuyPriceAfterFee { shares_subject, amount } => to_binary(&get_buy_price_after_fee(deps, shares_subject, amount)?),
        QueryMsg::GetSellPriceAfterFee { shares_subject, amount } => to_binary(&get_sell_price_after_fee(deps, shares_subject, amount)?),
        QueryMsg::GetShareBalance { shares_subject, my_address } => to_binary(&get_share_balance(deps, shares_subject, my_address)?),
        QueryMsg::GetState {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state)
        }
    }
}

pub fn get_price(deps: Deps, supply: Uint128, amount: Uint128) -> StdResult<GetPriceResponse> {
    let sum1 = if supply.is_zero() {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1)) * supply * (Uint128::new(2) * (supply - Uint128::new(1)) + Uint128::new(1)) / Uint128::new(6)
    };

    let sum2 = if supply == Uint128::zero() && amount == Uint128::new(1) {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1) + amount) * (supply + amount) * (Uint128::new(2) * (supply - Uint128::new(1) + amount) + Uint128::new(1)) / Uint128::new(6)
    };

    let summation = sum2 - sum1;

    let theprice = summation * Uint128::new(1_000_000_000_000_000_000) / Uint128::new(16000);
    Ok(GetPriceResponse { price: theprice })
}

pub fn get_buy_price(deps: Deps, shares_subject: Addr, amount: Uint128) -> StdResult<GetBuyPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let theprice_response = get_price(deps, state.sharesSupply.get(&shares_subject).copied().unwrap_or_else(|| Uint128::zero()), amount)?;
    let theprice: Uint128 = theprice_response.price;
    Ok(GetBuyPriceResponse { price: theprice})
}

pub fn get_sell_price(deps: Deps, shares_subject: Addr, amount: Uint128) -> StdResult<GetSellPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let price_response: GetPriceResponse = get_price(deps, state.sharesSupply.get(&shares_subject).copied().unwrap_or_else(|| Uint128::zero()) - amount, amount)?;
    let price2: Uint128 = price_response.price;
    Ok(GetSellPriceResponse { price: price2 })
}

pub fn get_buy_price_after_fee(deps: Deps, shares_subject: Addr, amount: Uint128) -> StdResult<GetBuyPriceAfterFeeResponse> {
    let state = STATE.load(deps.storage)?;
    let price_response: GetBuyPriceResponse = get_buy_price(deps, shares_subject, amount)?;
    let price: Uint128 = price_response.price;
    let protocol_fee = price * state.protocolFeePercent / Uint128::new(1_000_000_000_000_000_000);
    let subject_fee = price * state.subjectFeePercent / Uint128::new(1_000_000_000_000_000_000);
    let returnprice = price + protocol_fee + subject_fee;
    Ok(GetBuyPriceAfterFeeResponse { price: returnprice})
}

pub fn get_sell_price_after_fee(deps: Deps, shares_subject: Addr, amount: Uint128) -> StdResult<GetSellPriceAfterFeeResponse> {
    let state = STATE.load(deps.storage)?;
    let price_response: GetSellPriceResponse = get_sell_price(deps, shares_subject, amount)?;
    let price: Uint128 = price_response.price;
    let protocol_fee = price * state.protocolFeePercent / Uint128::new(1_000_000_000_000_000_000);
    let subject_fee = price * state.subjectFeePercent / Uint128::new(1_000_000_000_000_000_000);
    let returnprice = price - protocol_fee - subject_fee;
    Ok(GetSellPriceAfterFeeResponse { price: returnprice})
}

pub fn get_share_balance(deps: Deps, shares_subject: Addr, my_address: Addr) -> StdResult<GetShareBalance> {
    let state = STATE.load(deps.storage)?;
    Ok(GetShareBalance {
        amount: state
        .shares_balance
        .get(&shares_subject)
        .map(|balance_map| {
            balance_map
                .values()
                .fold(Uint128::zero(), |acc, &balance| acc + balance)
        })
        .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        // Create a mock environment
        let mut deps = mock_dependencies();

        // Set up a mock environment with an admin
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "earth"));

        // Execute the instantiate function
        let msg = InstantiateMsg {};
        let res: Result<Response, ContractError> = instantiate(deps.as_mut(), env.clone(), info, msg);

        // Ensure the contract was instantiated successfully
        assert!(res.is_ok());

        // Query the state to check if it matches the expected initialization
        //let state = STATE.load(deps.storage);
        let query_response = query(deps.as_ref(), env, QueryMsg::GetState {}).unwrap();
        let state: State = from_binary(&query_response).unwrap();

        // Check if the state is properly initialized
        assert_eq!(state.owner, Addr::unchecked("creator"));
        assert_eq!(state.subjectFeePercent, Uint128::new(5));
        assert_eq!(state.protocolFeePercent, Uint128::new(5));
        assert_eq!(state.protocolFeeDestination, Addr::unchecked("creator"));
        assert_eq!(state.shares_balance.len(), 0);
        assert_eq!(state.sharesSupply.len(), 0); // Initialized as an empty HashMap
    }
}
