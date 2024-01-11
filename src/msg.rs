use cosmwasm_std::{Addr, Uint128};
use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::State;

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {
    SetFeeDestination { fee_destination: Addr },
    SetProtocolFeePercent { protocol_fee_percent: Uint128 },
    SetSubjectFeePercent { subject_fee_percent: Uint128 },
    BuyShares { shares_subject: Addr, amount: Uint128 },
    SellShares { shares_subject: Addr, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetPriceResponse)]
    GetPrice { shares_subject: Addr, amount: Uint128, with_fees: bool },
    #[returns(GetShareBalanceResponse)]
    GetShareBalance { shares_subject: Addr, my_address: Addr },
    #[returns(State)]
    GetState { },
}

#[cw_serde]
pub struct GetPriceResponse {
    pub price: Uint128,
}
#[cw_serde]
pub struct GetShareBalanceResponse {
    pub amount: Uint128,
}