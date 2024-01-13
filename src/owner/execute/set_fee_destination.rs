use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::{state::STATE, ContractError};

pub fn set_fee_destination(
    deps: DepsMut,
    info: MessageInfo,
    fee_destination: Addr,
) -> Result<Response, ContractError> {
    let destination = fee_destination.to_string();
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_destination = fee_destination;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_fee_destination").add_attribute("destination", destination)))
}
