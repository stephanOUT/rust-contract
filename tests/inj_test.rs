mod inj_tests {
    use cosmwasm_std::{Addr, Coin, Uint128};
    use injective_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
    use injective_test_tube::{Account, Bank, InjectiveTestApp, Module, Wasm};
    use rust_contract::{
        msg::{ExecuteMsg, GetShareBalanceResponse, InstantiateMsg, QueryMsg},
        state::State,
    };

    #[test]
    fn test() {
        let app = InjectiveTestApp::new();
        // init two accounts, one admin, one user
        let accs = app
            .init_accounts(
                &[
                    Coin::new(1_000_000_000_000, "usdt"),
                    Coin::new(1_000_000_000_000_000_000, "inj"),
                ],
                2,
            )
            .unwrap();
        let admin = &accs[0];
        let user = &accs[1];
        // `Wasm` is the module we use to interact with cosmwasm releated logic on the appchain
        // it implements `Module` trait which you will see more later.
        let wasm = Wasm::new(&app);
        let bank = Bank::new(&app);

        // Load compiled wasm bytecode
        let wasm_byte_code = std::fs::read("./artifacts/rust_contract-aarch64.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, admin)
            .unwrap()
            .data
            .code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {},
                None,  // contract admin used for migration, not the same as cw1_whitelist admin
                None,  // contract label
                &[],   // funds
                admin, // signer
            )
            .unwrap()
            .data
            .address;

        // query contract state to check if contract instantiation works properly
        let contract_state = wasm
            .query::<QueryMsg, State>(&contract_addr, &QueryMsg::GetState {})
            .unwrap();

        assert_eq!(
            contract_state.owner,
            contract_state.protocol_fee_destination
        );

        // have user buy a share of user

        let balance_request = QueryBalanceRequest {
            address: user.address(),
            denom: "inj".to_string(),
        };

        let balanceResponse = bank.query_balance(&balance_request.into()).unwrap();
        println!(
            "user balance before transaction: {:?}",
            balanceResponse.balance
        );

        let funds = &[Coin::new(100000000000000000, "inj")];
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(user.address()),
                amount: Uint128::new(8),
            },
            funds,
            user,
        )
        .unwrap();

        let balance_request = QueryBalanceRequest {
            address: user.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!(
            "user balance after transaction: {:?}",
            balance_response.balance
        );

        // query shares
        let user_shares = wasm
            .query::<QueryMsg, GetShareBalanceResponse>(
                &contract_addr,
                &QueryMsg::GetShareBalance {
                    shares_subject: Addr::unchecked(user.address()),
                    my_address: Addr::unchecked(user.address()),
                },
            )
            .unwrap();
        println!("user shares: {:?}", user_shares.amount.u128());


        // query admin balance (where protocol fees go)
        let balance_request = QueryBalanceRequest {
            address: admin.address(),
            denom: "inj".to_string(),
        };     
        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!(
            "admin balance after transaction: {:?}",
            balance_response.balance
        );   

        // set fee destination to different address
        // wasm.execute::<ExecuteMsg>(
        //     &contract_addr,
        //     &ExecuteMsg::SetFeeDestination {
        //         fee_destination: Addr::unchecked(accs[1].address()),
        //     },
        //     &[],
        //     &accs[0],
        // )
        // .unwrap();
    }
}
