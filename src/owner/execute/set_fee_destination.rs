use cosmwasm_std::{Addr, DepsMut, Event, MessageInfo, Response};

use crate::{state::STATE, ContractError};

pub fn set_fee_destination(
    deps: DepsMut,
    info: MessageInfo,
    fee_destination: Addr,
) -> Result<Response, ContractError> {
    let validated_address = deps.api.addr_validate(&fee_destination.to_string())?;
    let destination = validated_address.to_string();
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_destination = validated_address;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_fee_destination").add_attribute("destination", destination)))
}
