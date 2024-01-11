#[cfg(not(feature = "library"))]
use crate::{
    msg::{
        ExecuteMsg, GetPriceResponse, GetShareBalanceResponse,
        InstantiateMsg, QueryMsg,
    },
    state::{State, SHARES_BALANCE, STATE},
    user::execute::{buy_shares, sell_shares},
    user::query::get_price_query,
    owner::execute::{set_fee_destination, set_protocol_fee_percent, set_subject_fee_percent},
    ContractError,
};
use cosmwasm_std::{entry_point, to_json_binary, Addr, Binary, Deps, StdResult, Uint128};
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
        } => to_json_binary::<GetPriceResponse>(&get_price_query(deps, shares_subject, amount, with_fees)?),
        // QueryMsg::GetBuyPrice {
        //     shares_subject,
        //     amount,
        // } => {
        //     println!(
        //         "Query: GetBuyPrice - shares_subject: {}, amount: {}",
        //         shares_subject, amount
        //     );
        //     let result = get_buy_price(deps, shares_subject, amount)?;
        //     println!("Query Result: {:?}", result);
        //     to_json_binary::<GetBuyPriceResponse>(&result)
        // }
        // QueryMsg::GetSellPrice {
        //     shares_subject,
        //     amount,
        // } => to_json_binary::<GetSellPriceResponse>(&get_sell_price(deps, shares_subject, amount)?),
        // QueryMsg::GetBuyPriceAfterFee {
        //     shares_subject,
        //     amount,
        // } => to_json_binary::<GetBuyPriceAfterFeeResponse>(&get_price_with_fees(
        //     deps,
        //     shares_subject,
        //     amount,
        // )?),
        // QueryMsg::GetSellPriceAfterFee {
        //     shares_subject,
        //     amount,
        // } => to_json_binary::<GetSellPriceAfterFeeResponse>(&get_sell_price_after_fee(
        //     deps,
        //     shares_subject,
        //     amount,
        // )?),
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

// pub fn get_buy_price(
//     deps: Deps,
//     shares_subject: Addr,
//     amount: Uint128,
// ) -> StdResult<GetBuyPriceResponse> {
//     let state = STATE.load(deps.storage)?;
//     let supply = SHARES_SUPPLY
//         .may_load(deps.storage, &shares_subject)?
//         .unwrap_or_default();
//     let buy_price_response = get_price(supply, amount)?;
//     let buy_price: Uint128 = buy_price_response.price;
//     Ok(GetBuyPriceResponse { price: buy_price })
// }

// pub fn get_sell_price(
//     deps: Deps,
//     shares_subject: Addr,
//     amount: Uint128,
// ) -> StdResult<GetSellPriceResponse> {
//     let state = STATE.load(deps.storage)?;
//     let supply = SHARES_SUPPLY
//         .may_load(deps.storage, &shares_subject)?
//         .unwrap_or_default();
//     let sell_price_response: GetPriceResponse = get_price(supply - amount, amount)?;
//     let sell_price: Uint128 = sell_price_response.price;
//     Ok(GetSellPriceResponse { price: sell_price })
// }

// pub fn get_sell_price_after_fee(
//     deps: Deps,
//     shares_subject: Addr,
//     amount: Uint128,
// ) -> StdResult<GetSellPriceAfterFeeResponse> {
//     let state = STATE.load(deps.storage)?;
//     let price_response: GetSellPriceResponse = get_sell_price(deps, shares_subject, amount)?;
//     let price: Uint128 = price_response.price;
//     let protocol_fee = price * state.protocol_fee_percent / Uint128::new(1_000_000_000_000_000_000);
//     let subject_fee = price * state.subject_fee_percent / Uint128::new(1_000_000_000_000_000_000);
//     let return_price = price - protocol_fee - subject_fee;
//     Ok(GetSellPriceAfterFeeResponse {
//         price: return_price,
//     })
// }

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
