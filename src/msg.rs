use cosmwasm_std::{Addr, Uint128};
use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::State;

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {
    SetFeeDestination { fee_destination: Addr },
    SetProtocolBuyFeePercent { protocol_buy_fee_percent: Uint128 },
    SetProtocolSellFeePercent { protocol_sell_fee_percent: Uint128 },
    SetSubjectBuyFeePercent { subject_buy_fee_percent: Uint128 },
    SetSubjectSellFeePercent { subject_sell_fee_percent: Uint128 },
    SetReferalBuyFeePercent { referal_buy_fee_percent: Uint128 },
    SetReferalSellFeePercent { referal_sell_fee_percent: Uint128 },
    BuyShares { shares_subject: Addr, referal: Addr },
    SellShares { shares_subject: Addr },
    ToggleTrading { is_enabled: bool },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetPriceResponse)]
    GetPrice { shares_subject: Addr, with_fees: bool, is_buy: bool },
    #[returns(GetShareBalanceResponse)]
    GetShareBalance { shares_subject: Addr, my_address: Addr },
    #[returns(State)]
    GetState { },
    #[returns(GetSubjectHoldersResponse)]
    GetSubjectHolders { shares_subject: Addr },
}

#[cw_serde]
pub struct GetPriceResponse {
    pub price: Uint128,
}
#[cw_serde]
pub struct GetShareBalanceResponse {
    pub amount: Uint128,
}
#[cw_serde]
pub struct GetSubjectHoldersResponse {
    pub amount: Uint128,
}