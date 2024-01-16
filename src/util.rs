use std::str::FromStr;

use cosmwasm_std::{Decimal, Uint128};

pub fn calculate_fee(price: Uint128, fee_percent: Uint128) -> Uint128 {
    return price * fee_percent / Uint128::new(100000);
}

/*pub fn get_price(supply: Uint128, amount: Uint128) -> Uint128 {
    let sum1 = if supply.is_zero() {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1))
            * supply
            * (Uint128::new(2) * (supply - Uint128::new(1)) + Uint128::new(1))
            / Uint128::new(6)
    };

    let sum2 = if supply == Uint128::zero() && amount == Uint128::new(1) {
        Uint128::zero()
    } else {
        (supply - Uint128::new(1) + amount)
            * (supply + amount)
            * (Uint128::new(2) * (supply - Uint128::new(1) + amount) + Uint128::new(1))
            / Uint128::new(6)
    };

    let summation = sum2 - sum1;
    let the_price = summation * Uint128::new(1000000000000000000) / Uint128::new(16000);
    return the_price;
}*/

// pub fn get_price(supply: Uint128, amount: Uint128) -> Uint128 {
//     if supply.is_zero() {
//         return Uint128::zero();
//     } else {
//         let price = (10_000_000f64 * 0.1) * ((10_000_000f64 * 0.6) + (supply * Uint128::new(10_000_000) / 5.8)).pow(18) / 10_000_000_000f64;

//         // let base_price: Uint128 = Uint128::new(1000000000000000000);
//         // let scaling_factor: f64 = 0.1;
//         // let exponent = (0.6 + supply.u128() as f64 / 5.8).powf(1.8);
//         // let result_float = scaling_factor * exponent;
//         // let result = Uint128::from(result_float as u128) * base_price;
//         result
//     }
// }

pub fn get_price(supply: Uint128) -> Uint128 {
    if supply.is_zero() {
        return Uint128::zero();
    }
    let price2 = 0.1 * (0.6 + supply.u128() as f64 / 5.8).powf(1.8);
    println!("Price Float: {}", price2);

    let price_decimal = Decimal::from_str(&price2.to_string()).unwrap();
    println!("Price Decimal: {}", price_decimal);

    let price = price_decimal.atomics();
    println!("Price Uint128: {}", price);

    return price;
}