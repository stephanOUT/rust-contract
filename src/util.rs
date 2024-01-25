use std::str::FromStr;

use cosmwasm_std::{Decimal, Uint128};

const FEE_SCALE: Uint128 = Uint128::new(100000);

pub fn calculate_fee(price: Uint128, fee_percent: Uint128) -> Uint128 {
    return price * fee_percent / FEE_SCALE;
}

pub fn get_price(supply: Uint128) -> Uint128 {
    if supply.is_zero() {
        return Uint128::zero();
    }
    let price2 = 0.1 * (0.06 + supply.u128() as f64 / 7.8).powf(2.05);
    let price_decimal = Decimal::from_str(&price2.to_string()).unwrap();
    let price = price_decimal.atomics();
    return price;
}