#[cfg(not(feature = "library"))]
use crate::{
    msg::{
        ExecuteMsg, GetBuyPriceAfterFeeResponse, GetBuyPriceResponse, GetPriceResponse,
        GetSellPriceAfterFeeResponse, GetSellPriceResponse, GetShareBalanceResponse,
        InstantiateMsg, QueryMsg,
    },
    state::{State, SHARES_BALANCE, SHARES_SUPPLY, STATE},
    ContractError,
};
use cosmwasm_std::{
    coins, entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, StdError, StdResult,
    Uint128,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

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
    let state = State {
        owner: info.sender.clone(),
        subject_fee_percent: Uint128::new(5),
        protocol_fee_percent: Uint128::new(5),
        protocol_fee_destination: info.sender.clone(), // change later
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("subject_fee_percent", Uint128::new(5))
        .add_attribute("protocol_fee_percent", Uint128::new(5)))
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

// TODO simplify this
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

    // user buying own shares for first time
    if shares_subject == info.sender && shares_supply.is_zero() {
        let price_response = get_price(amount, amount)?;
        let price: Uint128 = price_response.price;
        println!("Price: {}", price);
        let protocol_fee = price * state.protocol_fee_percent ;
        let subject_fee = price * state.subject_fee_percent ;
        println!("subject_fee: {}", subject_fee);
        println!("protocol_fee: {}", protocol_fee);
        // assert!(
        //     info.funds[0].amount >= price + protocol_fee + subject_fee,
        //     "Insufficient payment"
        // );
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

        //if info.funds[0].amount > (price + protocol_fee + subject_fee) {
        let amount_back = Uint128::new(1000000);//info.funds[0].amount - price - protocol_fee - subject_fee;
        let amount_back_result = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(amount_back.into(), "inj"),
        };

        //}

        let response = Response::new()
            .add_message(protocol_fee_result)
            .add_message(subject_fee_result)
            .add_message(amount_back_result);
        Ok(response)
    }
    // anyone buying shares
    else if shares_supply > Uint128::zero() {
        let price_response = get_price(shares_supply, amount)?;
        let price: Uint128 = price_response.price;
        println!("Price: {}", price);
        let protocol_fee =
            price * state.protocol_fee_percent / Uint128::new(100);
        let subject_fee =
            price * state.subject_fee_percent / Uint128::new(100);
        assert!(
            info.funds[0].amount >= price + protocol_fee + subject_fee,
            "Insufficient payment"
        );
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

        // let the_protocol_fee = vec![Coin {
        //     denom: "inj".to_string(),
        //     amount: protocol_fee.into(),
        // }];
        deps.api.debug("send protocol fee ");
        let protocol_fee_result = BankMsg::Send {
            to_address: state.protocol_fee_destination.to_string(),
            amount: coins(1, "inj"),
        };

        let the_subject_fee = vec![Coin {
            denom: "inj".to_string(),
            amount: subject_fee.into(),
        }];
        let subject_fee_result = BankMsg::Send {
            to_address: shares_subject.to_string(),
            amount: the_subject_fee,
        };

        //if info.funds[0].amount > (price + protocol_fee + subject_fee) {
        let amount_back = info.funds[0].amount - price - protocol_fee - subject_fee;
        let the_amount_back = vec![Coin {
            denom: "inj".to_string(),
            amount: amount_back.into(),
        }];
        let amount_back_result = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: the_amount_back,
        };
        //}

        let response = Response::new()
            .add_message(protocol_fee_result)
            .add_message(subject_fee_result)
            .add_message(amount_back_result);
        Ok(response)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "buy_shares: supply is zero",
        )))
    }
}

pub fn get_price(supply: Uint128, amount: Uint128) -> StdResult<GetPriceResponse> {
    println!("get_price: supply: {}, amount: {}", supply, amount);
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
    let the_price = (summation * Uint128::new(1000000000000000000)) / Uint128::new(16000);
    Ok(GetPriceResponse { price: the_price })
}
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
        let price_response: GetPriceResponse = get_price(supply, amount)?;
        let price: Uint128 = price_response.price;
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice { supply, amount } => to_json_binary(&get_price(supply, amount)?),
        QueryMsg::GetBuyPrice {
            shares_subject,
            amount,
        } => {
            println!(
                "Query: GetBuyPrice - shares_subject: {}, amount: {}",
                shares_subject, amount
            );
            let result = get_buy_price(deps, shares_subject, amount)?;
            println!("Query Result: {:?}", result);
            to_json_binary::<GetBuyPriceResponse>(&result)
        }
        QueryMsg::GetSellPrice {
            shares_subject,
            amount,
        } => to_json_binary::<GetSellPriceResponse>(&get_sell_price(deps, shares_subject, amount)?),
        QueryMsg::GetBuyPriceAfterFee {
            shares_subject,
            amount,
        } => to_json_binary::<GetBuyPriceAfterFeeResponse>(&get_buy_price_after_fee(
            deps,
            shares_subject,
            amount,
        )?),
        QueryMsg::GetSellPriceAfterFee {
            shares_subject,
            amount,
        } => to_json_binary::<GetSellPriceAfterFeeResponse>(&get_sell_price_after_fee(
            deps,
            shares_subject,
            amount,
        )?),
        QueryMsg::GetShareBalance {
            shares_subject,
            my_address,
        } => to_json_binary::<GetShareBalanceResponse>(&get_share_balance(
            deps,
            shares_subject,
            my_address,
        )?),
        QueryMsg::GetState {} => {
            println!("Query: GetState");
            let state: State = STATE.load(deps.storage)?;
            // let state = get_deserialized_state(&deps)?;
            println!("Query Result: {:?}", state);
            to_json_binary::<State>(&state)
        }
    }
}

pub fn get_buy_price(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetBuyPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();
    let buy_price_response = get_price(supply, amount)?;
    let buy_price: Uint128 = buy_price_response.price;
    Ok(GetBuyPriceResponse { price: buy_price })
}

pub fn get_sell_price(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
) -> StdResult<GetSellPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();
    let sell_price_response: GetPriceResponse = get_price(supply - amount, amount)?;
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

pub fn get_share_balance(
    deps: Deps,
    shares_subject: Addr,
    my_address: Addr,
) -> StdResult<GetShareBalanceResponse> {
    let balance = SHARES_BALANCE
        .may_load(deps.storage, (&my_address, &shares_subject))?
        .unwrap_or_default();
    Ok(GetShareBalanceResponse { amount: balance })
}

