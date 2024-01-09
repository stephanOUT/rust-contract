#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Addr, Uint128};
    use rust_contract::contract::{execute, instantiate, query};
    use rust_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
        assert_eq!(Uint128::new(5), state.protocol_fee_percent);
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
    fn buy_shares() {
        let mut deps = mock_dependencies();
        let shares_to_buy = Uint128::new(1);
        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // buy shares
        let info = mock_info("anyone", &coins(1000, "earth"));
        let msg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("shares_subject"),
            amount: shares_to_buy,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(3, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(
            shares_to_buy,
            state.shares_supply[&Addr::unchecked("shares_subject")]
        );
        assert_eq!(
            shares_to_buy,
            state.shares_balance[&Addr::unchecked("shares_subject")][&Addr::unchecked("anyone")]
        );
    }

    #[test]
    fn sell_shares() {
        let mut deps = mock_dependencies();
        let shares_to_buy = Uint128::new(1);
        let shares_to_sell = Uint128::new(1);
        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {};
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // buy shares
        let info = mock_info("anyone", &coins(1000, "earth"));
        let msg = ExecuteMsg::BuyShares {
            shares_subject: Addr::unchecked("shares_subject"),
            amount: shares_to_buy,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(3, res.messages.len());

        // sell shares
        let info = mock_info("anyone", &coins(1000, "earth"));
        let msg = ExecuteMsg::SellShares {
            shares_subject: Addr::unchecked("shares_subject"),
            amount: shares_to_sell,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(3, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let state: State = from_json(&res).unwrap();
        assert_eq!(
            Uint128::zero(),
            state.shares_supply[&Addr::unchecked("shares_subject")]
        );
        assert_eq!(
            Uint128::zero(),
            state.shares_balance[&Addr::unchecked("shares_subject")][&Addr::unchecked("anyone")]
        );
    }
}
