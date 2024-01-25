use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{msg::GetSubjectHoldersResponse, state::SHARES_HOLDERS};

pub fn get_subject_holders(
    deps: Deps,
    shares_subject: Addr,
) -> StdResult<GetSubjectHoldersResponse> {
    let validated_shares_subject_address = deps.api.addr_validate(&shares_subject.to_string())?;
    let holders = SHARES_HOLDERS
        .may_load(deps.storage, &validated_shares_subject_address)?
        .unwrap_or_default();
    Ok(GetSubjectHoldersResponse { amount: holders })
}
