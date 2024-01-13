#[cfg(not(feature = "library"))]
use crate::{
    msg::{ExecuteMsg, GetPriceResponse, GetShareBalanceResponse, InstantiateMsg, QueryMsg},
    owner::execute::{set_fee_destination, set_protocol_fee_percent, set_subject_fee_percent},
    state::{State, STATE},
    user::execute::{buy_shares, sell_shares},
    user::query::get_price_query,
    ContractError,
};
use crate::{
    owner::execute::{set_buy_sell_quantity_limit, toggle_trading},
    user::query::{get_share_balance, get_state},
};
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Event, StdResult, Uint128};
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
        subject_fee_percent: Uint128::new(5000),  // 5.000%
        protocol_fee_percent: Uint128::new(5000), // 5.000%
        protocol_fee_destination: info.sender.clone(), // change later
        trading_is_enabled: true,
        buy_sell_quantity_limit: Uint128::new(20),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_event(Event::new("contract_instantiated"))
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("subject_fee_percent", Uint128::new(5000))
        .add_attribute("protocol_fee_percent", Uint128::new(5000)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let state: State = STATE.load(deps.storage)?;
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
        } => {
            if state.trading_is_enabled == false {
                return Err(ContractError::TradingIsDisabled {});
            }
            if amount > state.buy_sell_quantity_limit {
                return Err(ContractError::BuySellQuantityLimitExceeded {});
            }
            buy_shares(deps, info, shares_subject, amount)
        }
        ExecuteMsg::SellShares {
            shares_subject,
            amount,
        } => {
            if state.trading_is_enabled == false {
                return Err(ContractError::TradingIsDisabled {});
            }
            if amount > state.buy_sell_quantity_limit {
                return Err(ContractError::BuySellQuantityLimitExceeded {});
            }
            sell_shares(deps, info, shares_subject, amount)
        }
        ExecuteMsg::ToggleTrading { is_enabled } => toggle_trading(deps, info, is_enabled),
        ExecuteMsg::SetBuySellQuantityLimit { limit } => {
            set_buy_sell_quantity_limit(deps, info, limit)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {
            shares_subject,
            amount,
            with_fees,
            is_buy,
        } => to_json_binary::<GetPriceResponse>(&get_price_query(
            deps,
            shares_subject,
            amount,
            with_fees,
            is_buy,
        )?),
        QueryMsg::GetShareBalance {
            shares_subject,
            my_address,
        } => to_json_binary::<GetShareBalanceResponse>(&get_share_balance(
            deps,
            shares_subject,
            my_address,
        )?),
        QueryMsg::GetState {} => to_json_binary(&get_state(deps)?),
    }
}
