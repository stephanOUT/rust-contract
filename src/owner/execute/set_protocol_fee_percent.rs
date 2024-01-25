use cosmwasm_std::{DepsMut, Event, MessageInfo, Response, StdError, Uint128};

use crate::{state::STATE, ContractError};

pub fn set_protocol_buy_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if fee_percent > Uint128::new(5000) {
            return Err(ContractError::Std(StdError::generic_err(
                "Cannot set fees higher than 5%",
            )));
        }
        state.protocol_buy_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_event(
        Event::new("set_protocol_buy_fee_percent").add_attribute("fee_percent", fee_percent),
    ))
}

pub fn set_protocol_sell_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        if fee_percent > Uint128::new(5000) {
            return Err(ContractError::Std(StdError::generic_err(
                "Cannot set fees higher than 5%",
            )));
        }
        state.protocol_sell_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_event(
        Event::new("set_protocol_sell_fee_percent").add_attribute("fee_percent", fee_percent),
    ))
}
