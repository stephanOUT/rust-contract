use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

use crate::{state::STATE, ContractError};

pub fn set_protocol_fee_percent(
    deps: DepsMut,
    info: MessageInfo,
    fee_percent: Uint128,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_percent = fee_percent;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_protocol_fee_percent"))
}