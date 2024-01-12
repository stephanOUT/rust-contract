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
    is_buy: bool
) -> StdResult<GetPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    let mut price = get_price(supply, amount);
    if !is_buy {
        price = get_price((supply - amount), amount);
    }

    if with_fees {
        let protocol_fee = calculate_fee(price, state.protocol_fee_percent);
        let subject_fee = calculate_fee(price, state.subject_fee_percent);
        let mut price_with_fees = price + protocol_fee + subject_fee;
        if !is_buy {
            price_with_fees = price - protocol_fee - subject_fee;
        }
        Ok(GetPriceResponse {
            price: price_with_fees,
        })
    } else {
        Ok(GetPriceResponse { price })
    }

    // IT SEEMS THE MUT STATE ERRORS ON FEEDBACK CALLING THE FUNCTION
}
