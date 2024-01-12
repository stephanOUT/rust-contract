use cosmwasm_std::{DepsMut, MessageInfo, Response};

use crate::{state::STATE, ContractError};

pub fn toggle_trading(
    deps: DepsMut,
    info: MessageInfo,
    is_enabled: bool,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.trading_is_enabled = is_enabled;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "toggle_trading"))
}
