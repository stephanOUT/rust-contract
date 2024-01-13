use cosmwasm_std::{DepsMut, Event, MessageInfo, Response, Uint128};

use crate::{state::STATE, ContractError};

pub fn set_buy_sell_quantity_limit(
    deps: DepsMut,
    info: MessageInfo,
    limit: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.buy_sell_quantity_limit = limit;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_buy_sell_quantity_limit").add_attribute("limit", limit)))
}
