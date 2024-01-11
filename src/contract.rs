use crate::user::query::get_share_balance;
#[cfg(not(feature = "library"))]
use crate::{
    msg::{ExecuteMsg, GetPriceResponse, GetShareBalanceResponse, InstantiateMsg, QueryMsg},
    owner::execute::{set_fee_destination, set_protocol_fee_percent, set_subject_fee_percent},
    state::{State, STATE},
    user::execute::{buy_shares, sell_shares},
    user::query::get_price_query,
    ContractError,
};
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, StdResult, Uint128};
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
        subject_fee_percent: Uint128::new(10),
        protocol_fee_percent: Uint128::new(10), // 10 makes it easy to test
        protocol_fee_destination: info.sender.clone(), // change later
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("subject_fee_percent", Uint128::new(10))
        .add_attribute("protocol_fee_percent", Uint128::new(10)))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {
            shares_subject,
            amount,
            with_fees,
        } => to_json_binary::<GetPriceResponse>(&get_price_query(
            deps,
            shares_subject,
            amount,
            with_fees,
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
