use cosmwasm_std::{Addr, Deps, StdResult};

use crate::{msg::GetShareBalanceResponse, state::SHARES_BALANCE};

pub fn get_share_balance(
    deps: Deps,
    shares_subject: Addr,
    my_address: Addr,
) -> StdResult<GetShareBalanceResponse> {
    let validated_shares_subject_address = deps.api.addr_validate(&shares_subject.to_string())?;
    let validated_my_address = deps.api.addr_validate(&my_address.to_string())?;
    let balance = SHARES_BALANCE
        .may_load(
            deps.storage,
            (&validated_my_address, &validated_shares_subject_address),
        )?
        .unwrap_or_default();
    Ok(GetShareBalanceResponse { amount: balance })
}
