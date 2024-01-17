use cosmwasm_std::{Addr, Deps, StdResult, Uint128};

use crate::{
    msg::GetPriceResponse,
    state::{SHARES_SUPPLY, STATE},
    util::{calculate_fee, get_price},
};

pub fn get_price_query(
    deps: Deps,
    shares_subject: Addr,
    with_fees: bool,
    is_buy: bool,
) -> StdResult<GetPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let supply = SHARES_SUPPLY
        .may_load(deps.storage, &shares_subject)?
        .unwrap_or_default();

    // Calculate the price without considering fees
    let base_price = get_price(if is_buy {
        supply
    } else {
        supply - Uint128::new(1)
    });

    // Calculate fees if needed
    let (protocol_fee, subject_fee, referral_fee) = if with_fees && is_buy {
        // BUY
        (
            calculate_fee(base_price, state.protocol_buy_fee_percent),
            calculate_fee(base_price, state.subject_buy_fee_percent),
            calculate_fee(base_price, state.referral_buy_fee_percent),
        )
    } else if with_fees && !is_buy {
        // SELL
        (
            calculate_fee(base_price, state.protocol_sell_fee_percent),
            calculate_fee(base_price, state.subject_sell_fee_percent),
            calculate_fee(base_price, state.referral_sell_fee_percent),
        )
    } else {
        // NO FEES
        (Uint128::zero(), Uint128::zero(), Uint128::zero())
    };

    // Adjust the price based on whether it's a buy or sell
    let price_with_fees = if is_buy {
        base_price + protocol_fee + subject_fee + referral_fee
    } else {
        base_price - protocol_fee - subject_fee - referral_fee
    };

    Ok(GetPriceResponse {
        price: price_with_fees,
    })
}