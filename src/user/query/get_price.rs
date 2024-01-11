use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

use crate::{
    msg::GetPriceResponse,
    state::{SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
};

pub fn get_price_query(
    deps: Deps,
    shares_subject: Addr,
    amount: Uint128,
    with_fees: bool,
) -> StdResult<GetPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    let price = get_price(supply, amount);

    if with_fees {
        let protocol_fee = calculate_fee(price, state.protocol_fee_percent);
        let subject_fee = calculate_fee(price, state.subject_fee_percent);
        let price_with_fees = price + protocol_fee + subject_fee;
        Ok(GetPriceResponse {
            price: price_with_fees,
        })
    } else {
        Ok(GetPriceResponse { price })
    }
}
