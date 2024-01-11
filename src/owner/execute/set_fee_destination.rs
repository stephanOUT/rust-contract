pub fn set_fee_destination(
    deps: DepsMut,
    info: MessageInfo,
    fee_destination: Addr,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.protocol_fee_destination = fee_destination;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set_fee_destination"))
}