use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub subject_buy_fee_percent: Uint128,
    pub subject_sell_fee_percent: Uint128,
    pub protocol_buy_fee_percent: Uint128,
    pub protocol_sell_fee_percent: Uint128,
    pub referal_buy_fee_percent: Uint128,
    pub referal_sell_fee_percent: Uint128,
    pub protocol_fee_destination: Addr,
    pub trading_is_enabled: bool,
}

pub const STATE: Item<State> = Item::new("state");
pub const SHARES_SUPPLY: Map<&Addr, Uint128> = Map::new("shares_supply");
pub const SHARES_BALANCE: Map<(&Addr, &Addr), Uint128> = Map::new("shares_balance");
pub const SHARES_HOLDERS: Map<&Addr, Uint128> = Map::new("shares_holders");