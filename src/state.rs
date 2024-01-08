use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use cosmwasm_std::{Addr, to_binary, from_binary, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub subjectFeePercent: Uint128,
    pub protocolFeePercent: Uint128,
    pub protocolFeeDestination: Addr,
    pub sharesSupply: HashMap<Addr, Uint128>,
    pub shares_balance: HashMap<Addr, HashMap<Addr, Uint128>>,
}

impl Default for State {
    fn default() -> Self {
        State {
            owner: Addr::unchecked("default_owner"), // Provide a default value for owner
            subjectFeePercent: Uint128::zero(),
            protocolFeePercent: Uint128::zero(),
            protocolFeeDestination: Addr::unchecked("default_destination"), // Provide a default value for protocolFeeDestination
            sharesSupply: HashMap::new(),
            shares_balance: HashMap::new(),
        }
    }
}

pub const STATE: Item<State> = Item::new("state");