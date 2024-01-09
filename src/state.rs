use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub subject_fee_percent: Uint128,
    pub protocol_fee_percent: Uint128,
    pub protocol_fee_destination: Addr,
}

pub const STATE: Item<State> = Item::new("state");
pub const SHARES_SUPPLY: Map<&Addr, Uint128> = Map::new("shares_supply");
pub const SHARES_BALANCE: Map<(&Addr, &Addr), Uint128> = Map::new("shares_balance");
