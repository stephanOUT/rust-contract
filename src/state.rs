use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub subject_fee_percent: Uint128,
    pub protocol_fee_percent: Uint128,
    pub protocol_fee_destination: Addr,
   // pub shares_supply: HashMap<Addr, Uint128>,
   // pub shares_balance: HashMap<Addr, HashMap<Addr, Uint128>>,
}

// impl Default for State {
//     fn default() -> Self {
//         State {
//             owner: Addr::unchecked("default_owner"), // Provide a default value for owner
//             subject_fee_percent: Uint128::zero(),
//             protocol_fee_percent: Uint128::zero(),
//             protocol_fee_destination: Addr::unchecked("default_destination"), // Provide a default value for protocolFeeDestination
//             shares_supply: HashMap::new(),
//             shares_balance: HashMap::new(),
//         }
//     }
// }

pub const STATE: Item<State> = Item::new("state");
pub const SHARES_SUPPLY: Map<&Addr, Uint128> = Map::new("shares_supply");
pub const SHARES_BALANCE: Map<(&Addr, &Addr), Uint128> = Map::new("shares_balance");