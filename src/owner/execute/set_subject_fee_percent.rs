use cosmwasm_std::{DepsMut, Event, MessageInfo, Response, Uint128};

use crate::{state::STATE, ContractError};

pub fn set_subject_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.subject_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new()
        .add_event(Event::new("set_subject_fee_percent").add_attribute("fee_percent", fee_percent)))
}
