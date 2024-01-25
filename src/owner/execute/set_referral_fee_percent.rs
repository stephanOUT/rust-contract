use cosmwasm_std::{DepsMut, Event, MessageInfo, Response, Uint128, StdError };

use crate::{state::STATE, ContractError};

const MAX_FEE_PERCENT: Uint128 = Uint128::new(2500);

pub fn set_referral_buy_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if fee_percent > MAX_FEE_PERCENT {
            return Err(ContractError::Std(StdError::generic_err(
                "Cannot set fees higher than MAX_FEE_PERCENT",
            )));
        }
        state.referral_buy_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_referral_buy_fee_percent").add_attribute("fee_percent", fee_percent)))
}

pub fn set_referral_sell_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if fee_percent > MAX_FEE_PERCENT {
            return Err(ContractError::Std(StdError::generic_err(
                "Cannot set fees higher than MAX_FEE_PERCENT",
            )));
        }
        state.referral_sell_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_referral_sell_fee_percent").add_attribute("fee_percent", fee_percent)))
}
