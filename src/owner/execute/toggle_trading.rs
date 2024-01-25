use cosmwasm_std::{DepsMut, Event, MessageInfo, Response};

use crate::{
    state::{State, STATE},
    ContractError,
};

pub fn toggle_trading(
    deps: DepsMut,
    info: MessageInfo,
    is_enabled: bool,
) -> Result<Response, ContractError> {
    let state: State = STATE.load(deps.storage)?;

    if state.trading_is_enabled == is_enabled {
        return Err(ContractError::TradingStateTheSame {});
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.trading_is_enabled = is_enabled;
        Ok(state)
    })?;
    Ok(Response::new().add_event(
        Event::new("toggle_trading").add_attribute("is_enabled", is_enabled.to_string()),
    ))
}
