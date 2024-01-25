#[cfg(not(feature = "library"))]
use crate::{
    msg::{
        ExecuteMsg, GetPriceResponse, GetShareBalanceResponse, GetSubjectHoldersResponse, InstantiateMsg, QueryMsg,
    },
    owner::execute::{
        set_fee_destination, set_protocol_buy_fee_percent, set_protocol_sell_fee_percent,
        set_referral_buy_fee_percent, set_referral_sell_fee_percent, set_subject_buy_fee_percent,
        set_subject_sell_fee_percent,
    },
    state::{State, STATE},
    user::execute::{buy_shares, sell_shares},
    user::query::get_price_query,
    ContractError,
};
use crate::{
    owner::execute::toggle_trading,
    user::query::{get_share_balance, get_state, get_subject_holders},
};
use cosmwasm_std::{entry_point, to_json_binary, Binary, Deps, Event, StdResult, Uint128};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:my-first-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SUBJECT_BUY_FEE_PERCENT: Uint128 = Uint128::new(3000); // 3.000%
const SUBJECT_SELL_FEE_PERCENT: Uint128 = Uint128::new(3000); // 3.000%
const PROTOCOL_BUY_FEE_PERCENT: Uint128 = Uint128::new(2500); // 3.000%
const PROTOCOL_SELL_FEE_PERCENT: Uint128 = Uint128::new(3000); // 3.000%
const REFERRAL_BUY_FEE_PERCENT: Uint128 = Uint128::new(500); // 0.500%
const REFERRAL_SELL_FEE_PERCENT: Uint128 = Uint128::new(0); // 0.000%

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg,) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        subject_buy_fee_percent: SUBJECT_BUY_FEE_PERCENT,
        subject_sell_fee_percent: SUBJECT_SELL_FEE_PERCENT,
        protocol_buy_fee_percent: PROTOCOL_BUY_FEE_PERCENT,
        protocol_sell_fee_percent: PROTOCOL_SELL_FEE_PERCENT,
        referral_buy_fee_percent: REFERRAL_BUY_FEE_PERCENT, 
        referral_sell_fee_percent: REFERRAL_SELL_FEE_PERCENT,
        protocol_fee_destination: info.sender.clone(), // change later
        trading_is_enabled: true,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_event(Event::new("contract_instantiated"))
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("subject_buy_fee_percent", SUBJECT_BUY_FEE_PERCENT)
        .add_attribute("subject_sell_fee_percent", SUBJECT_SELL_FEE_PERCENT)
        .add_attribute("protocol_buy_fee_percent", PROTOCOL_BUY_FEE_PERCENT)
        .add_attribute("protocol_sell_fee_percent", PROTOCOL_SELL_FEE_PERCENT)
        .add_attribute("referral_buy_fee_percent", REFERRAL_BUY_FEE_PERCENT)
        .add_attribute("referral_sell_fee_percent", REFERRAL_SELL_FEE_PERCENT))
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
        ExecuteMsg::SetProtocolBuyFeePercent {
            protocol_buy_fee_percent,
        } => set_protocol_buy_fee_percent(deps, info, protocol_buy_fee_percent),
        ExecuteMsg::SetProtocolSellFeePercent {
            protocol_sell_fee_percent,
        } => set_protocol_sell_fee_percent(deps, info, protocol_sell_fee_percent),
        ExecuteMsg::SetSubjectBuyFeePercent {
            subject_buy_fee_percent,
        } => set_subject_buy_fee_percent(deps, info, subject_buy_fee_percent),
        ExecuteMsg::SetSubjectSellFeePercent {
            subject_sell_fee_percent,
        } => set_subject_sell_fee_percent(deps, info, subject_sell_fee_percent),
        ExecuteMsg::SetReferralBuyFeePercent {
            referral_buy_fee_percent,
        } => set_referral_buy_fee_percent(deps, info, referral_buy_fee_percent),
        ExecuteMsg::SetReferralSellFeePercent {
            referral_sell_fee_percent,
        } => set_referral_sell_fee_percent(deps, info, referral_sell_fee_percent),
        ExecuteMsg::BuyShares {
            shares_subject,
            referral,
        } => {
            if state.trading_is_enabled == false {
                return Err(ContractError::TradingIsDisabled {});
            }
            buy_shares(deps, info, shares_subject, referral)
        }
        ExecuteMsg::SellShares {
            shares_subject,
            referral,
        } => {
            if state.trading_is_enabled == false {
                return Err(ContractError::TradingIsDisabled {});
            }
            sell_shares(deps, info, shares_subject, referral)
        }
        ExecuteMsg::ToggleTrading { is_enabled } => toggle_trading(deps, info, is_enabled),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPrice {
            shares_subject,
            with_fees,
            is_buy,
        } => to_json_binary::<GetPriceResponse>(&get_price_query(
            deps,
            shares_subject,
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
        QueryMsg::GetSubjectHolders { shares_subject } => {
            to_json_binary::<GetSubjectHoldersResponse>(&get_subject_holders(deps, shares_subject)?)
        }
    }
}
