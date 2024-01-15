#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Addr, Uint128};
    use rust_contract::contract::{execute, instantiate, query};
    use rust_contract::msg::{
        ExecuteMsg, GetPriceResponse, GetShareBalanceResponse, InstantiateMsg, QueryMsg,
    };
    use rust_contract::state::State;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(
            State {
                owner: Addr::unchecked("creator"),
                subject_fee_percent: Uint128::new(5000),
                protocol_fee_percent: Uint128::new(5000),
                protocol_fee_destination: Addr::unchecked("creator"),
                trading_is_enabled: true,
                buy_sell_quantity_limit: Uint128::new(20),
            },
            state
        );
    }

    #[test]
    fn set_fee_destination() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::SetFeeDestination {
            fee_destination: Addr::unchecked("fee_destination"),
        };

        // we can just call .unwrap() to assert this was a success
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(
            "fee_destination",
            state.protocol_fee_destination.to_string()
        );
    }

    #[test]
    fn set_protocol_fee_percent() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::SetProtocolFeePercent {
            protocol_fee_percent: Uint128::new(10),
        };

        // we can just call .unwrap() to assert this was a success
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(Uint128::new(10), state.protocol_fee_percent);
    }

    #[test]
    fn set_subject_fee_percent() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::SetSubjectFeePercent {
            subject_fee_percent: Uint128::new(10),
        };

        // we can just call .unwrap() to assert this was a success
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(Uint128::new(10), state.subject_fee_percent);
    }

    #[test]
    fn buy_self_shares() {
        let mut deps = mock_dependencies();
        let shares_to_buy = Uint128::new(1);
        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // buy shares
        let info = mock_info("anyone", &coins(1000000000000000, "earth"));
        let msg: ExecuteMsg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("anyone"),
            amount: shares_to_buy,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        println!("{:?}", res.events);
        assert_eq!(2, res.messages.len());

        // check how much user gets back
       // println!("{:?}", res.messages);

        // it worked, let's query the shares balance
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetShareBalance {
                shares_subject: Addr::unchecked("anyone"),
                my_address: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let shares_balance: GetShareBalanceResponse = from_json(&res).unwrap();
        assert_eq!(shares_to_buy, shares_balance.amount);
    }

    #[test]
    fn buy_shares_of_someone_else() {
        // init
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // user 1 buys 1 share of user 1
        println!("user1");
        let info = mock_info("user_1", &coins(1000000000000000000, "earth"));
        let msg: ExecuteMsg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("user_1"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // user 2 buys 1 share of user 1
        println!("user2");
        let info = mock_info("user_2", &coins(1000000000000000000, "earth"));
        let msg: ExecuteMsg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("user_1"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        println!("{:?}", res.events);
 
        assert_eq!(3, res.messages.len());
        println!("user2 finish");

    }

    #[test]
    fn sell_self_shares() {
        let mut deps = mock_dependencies();
        let shares_to_buy = Uint128::new(2);
        let shares_to_sell = Uint128::new(1);
        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // buy first share (cant sell)
        let info = mock_info("anyone", &coins(1000000000000000, "earth"));
        let msg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("anyone"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // buy another share (can sell)
        let info = mock_info("anyone", &coins(1000000000000000, "earth"));
        let msg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("anyone"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // sell share
        let info = mock_info("anyone", &coins(1000000000000000, "earth"));
        let msg = ExecuteMsg::SellShares {
            shares_subject: Addr::unchecked("anyone"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(3, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetShareBalance {
                shares_subject: Addr::unchecked("anyone"),
                my_address: Addr::unchecked("anyone"),
            },
        )
        .unwrap();
        let shares_balance: GetShareBalanceResponse = from_json(&res).unwrap();
        assert_eq!(shares_to_buy - shares_to_sell, shares_balance.amount);
        // );
    }

    //#[test]
    // fn get_price() {
    //     let mut deps = mock_dependencies();

    //     // init
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg = InstantiateMsg {};
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // get price
    //     let msg = QueryMsg::GetPrice {
    //         supply: Uint128::new(1),
    //         amount: Uint128::new(1),
    //     };
    //     let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    //     let price: GetPriceResponse = from_json(&res).unwrap();
    //     assert_eq!(Uint128::new(62500000000000), price.price);
    // }

    // #[test]
    // fn get_buy_price() {
    //     let mut deps = mock_dependencies();

    //     // init
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg = InstantiateMsg {};
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // get buy price price
    //     let msg = QueryMsg::GetBuyPrice {
    //         shares_subject: Addr::unchecked("creator"),
    //         amount: Uint128::new(1),
    //     };
    //     let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    //     let get_buy_price_response: GetBuyPriceResponse = from_json(&res).unwrap();
    //     assert_eq!(Uint128::new(0), get_buy_price_response.price);
    // }
    #[test]
    fn get_share_balance() {
        let mut deps = mock_dependencies();

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // buy first share
        let info = mock_info("anyone", &coins(1000000000000000, "earth"));
        let msg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("anyone"),
            amount: Uint128::new(1),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(2, res.messages.len());

        // get share balance
        let msg = QueryMsg::GetShareBalance {
            shares_subject: Addr::unchecked("anyone"),
            my_address: Addr::unchecked("anyone"),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let get_share_balance_response: GetShareBalanceResponse = from_json(&res).unwrap();
        assert_eq!(Uint128::new(1), get_share_balance_response.amount);
    }

    // #[test]
    // fn get_buy_price_after_fee() {
    //     let mut deps = mock_dependencies();

    //     // init
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg = InstantiateMsg {};
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // set protocol fee percent
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg = ExecuteMsg::SetProtocolFeePercent {
    //         protocol_fee_percent: Uint128::new(10),
    //     };
    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // set subject fee percent
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg = ExecuteMsg::SetSubjectFeePercent {
    //         subject_fee_percent: Uint128::new(5),
    //     };
    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // get buy price after fee
    //     let msg = QueryMsg::GetBuyPriceAfterFee {
    //         shares_subject: Addr::unchecked("creator"),
    //         amount: Uint128::new(1),
    //     };

    //     let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    //     let get_buy_price_response: GetBuyPriceAfterFeeResponse = from_json(&res).unwrap();
    //     assert_eq!(Uint128::new(0), get_buy_price_response.price);
    // }
}
