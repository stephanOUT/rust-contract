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
                    Coin::new(1000000000000000000, "inj"), // 1 INJ
                ],
                3,
            )
            .unwrap();
        let admin = &accs[0];
        let user_1 = &accs[1];
        let user_2 = &accs[2];
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
                None, // contract admin used for migration, not the same as cw1_whitelist admin
                Some("label"), // contract label
                &[],  // funds
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
            address: user_1.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!(
            "user balance before transaction: {:?}",
            balance_response.balance
        );

        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(user_1.address()),
                referral: Addr::unchecked(user_2.address()),
            },
            &[], // empty funds when buying first share
            user_1,
        )
        .unwrap();

        let balance_request = QueryBalanceRequest {
            address: user_1.address(),
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
                    shares_subject: Addr::unchecked(user_1.address()),
                    my_address: Addr::unchecked(user_1.address()),
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

    #[test]
    fn buy_share_of_other_user() {
        // init
        let app = InjectiveTestApp::new();
        let accs = app
            .init_accounts(
                &[
                    Coin::new(1000000000000000000, "inj"), // 1 INJ
                ],
                3,
            )
            .unwrap();
        let admin = &accs[0];
        let user_1 = &accs[1];
        let user_2 = &accs[2];
        let wasm = Wasm::new(&app);
        let bank = Bank::new(&app);
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
                None, // contract admin used for migration, not the same as cw1_whitelist admin
                Some("label"), // contract label
                &[],  // funds
                admin, // signer
            )
            .unwrap()
            .data
            .address;
        println!("user 1: buying initial share");
        let balance_request = QueryBalanceRequest {
            address: user_1.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 1: balance before: {:?}", balance_response.balance);

        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(user_1.address()),
                referral: Addr::unchecked(user_2.address()),
            },
            &[], // empty funds when buying first share
            user_1,
        )
        .unwrap();

        let balance_request = QueryBalanceRequest {
            address: user_1.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 1: balance after: {:?}", balance_response.balance);

        println!("user 2: buying initial share");
        let balance_request = QueryBalanceRequest {
            address: user_2.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 2: balance before: {:?}", balance_response.balance);

        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(user_2.address()),
                referral: Addr::unchecked(user_1.address()),
            },
            &[], // empty funds when buying first share
            user_2,
        )
        .unwrap();

        let balance_request = QueryBalanceRequest {
            address: user_2.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 2: balance after: {:?}", balance_response.balance);

        println!("user 2: buying user 1 share");
        let balance_request = QueryBalanceRequest {
            address: user_2.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 2: balance before: {:?}", balance_response.balance);
        let funds = &[Coin::new(100000000000000000, "inj")];
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(user_1.address()),
                referral: Addr::unchecked(user_2.address()),
            },
            funds, // send funds when buying shares
            user_2,
        )
        .unwrap();

        let balance_request = QueryBalanceRequest {
            address: user_2.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!("user 2: balance after: {:?}", balance_response.balance);
    }
}
