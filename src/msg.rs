use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetFeeDestination { fee_destination: Addr },
    SetProtocolFeePercent { protocol_fee_percent: Uint128 },
    SetSubjectFeePercent { subject_fee_percent: Uint128 },
    BuyShares { shares_subject: Addr, amount: Uint128 },
    SellShares { shares_subject: Addr, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrice { supply: Uint128, amount: Uint128 },
    GetBuyPrice { shares_subject: Addr, amount: Uint128 },
    GetSellPrice { shares_subject: Addr, amount: Uint128 },
    GetBuyPriceAfterFee { shares_subject: Addr, amount: Uint128 },
    GetSellPriceAfterFee { shares_subject: Addr, amount: Uint128 },
    GetShareBalance { shares_subject: Addr, my_address: Addr },
    GetState { },

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetPriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetBuyPriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSellPriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetBuyPriceAfterFeeResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSellPriceAfterFeeResponse {
    pub price: Uint128,
}