use std::str::FromStr;

use cosmwasm_std::{Decimal, Uint128};

const FEE_SCALE: Uint128 = Uint128::new(100000);

const MULTIPLY_SCALER: f64 = 0.1;
const FRACTION: f64 = 0.06;
const FRACTION_DENOM: f64 = 7.8;
const FORMULA_EXPONENT: f64 = 2.05;

pub fn calculate_fee(price: Uint128, fee_percent: Uint128) -> Uint128 {
    return price * fee_percent / FEE_SCALE;
}

pub fn get_price(supply: Uint128) -> Uint128 {
    if supply.is_zero() {
        return Uint128::zero();
    }
    let price2 = MULTIPLY_SCALER * (FRACTION + supply.u128() as f64 / FRACTION_DENOM).powf(FORMULA_EXPONENT);
    let price_decimal = Decimal::from_str(&price2.to_string()).unwrap();
    let price = price_decimal.atomics();
    return price;
}