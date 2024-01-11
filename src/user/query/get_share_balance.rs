use cosmwasm_std::{Deps, Addr, StdResult};

use crate::{msg::GetShareBalanceResponse, state::SHARES_BALANCE};

pub fn get_share_balance(
    deps: Deps,
    shares_subject: Addr,
    my_address: Addr,
) -> StdResult<GetShareBalanceResponse> {
    let balance = SHARES_BALANCE
        .may_load(deps.storage, (&my_address, &shares_subject))?
        .unwrap_or_default();
    Ok(GetShareBalanceResponse { amount: balance })
}