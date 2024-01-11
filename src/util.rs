use cosmwasm_std::Uint128;

pub fn calculate_fee(price: Uint128, fee_percent: Uint128) -> Uint128 {
    return price * fee_percent / Uint128::new(100);
}

pub fn get_price(supply: Uint128, amount: Uint128) -> Uint128 {
    println!("get_price: supply: {}, amount: {}", supply, amount);
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
}
