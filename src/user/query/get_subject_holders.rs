use cosmwasm_std::{Deps, Addr, StdResult};

use crate::{msg::GetSubjectHoldersResponse, state::SHARES_HOLDERS};

pub fn get_subject_holders (
    deps: Deps,
    shares_subject: Addr
) -> StdResult<GetSubjectHoldersResponse> {
    let holders = SHARES_HOLDERS
        .may_load(deps.storage, (&shares_subject))?
        .unwrap_or_default();
    Ok(GetSubjectHoldersResponse { amount: holders })
}