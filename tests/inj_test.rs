mod inj_tests {
    use cosmwasm_std::{Addr, Coin, Uint128};
    use injective_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
    use injective_test_tube::{Account, Bank, InjectiveTestApp, Module, SigningAccount, Wasm};
    use rust_contract::{
        msg::{ExecuteMsg, GetPriceResponse, GetShareBalanceResponse, InstantiateMsg, QueryMsg},
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

        // have user 1 buy a share of user 1

        let balance_request = QueryBalanceRequest {
            address: user_1.address(),
            denom: "inj".to_string(),
        };

        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        println!(
            "user 1 balance before transaction: {:?}",
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

    #[test]
    fn end_to_end() {
        let app = InjectiveTestApp::new();
        let admin = &app
            .init_account(&[Coin::new(1000000000000000000, "inj")])
            .unwrap();
        let user_1 = &app
            .init_account(&[Coin::new(1000000000000000000, "inj")])
            .unwrap();
        let user_2 = &app
            .init_account(&[Coin::new(1000000000000000000, "inj")])
            .unwrap();
        let referring_user = &app.init_account(&[]).unwrap();

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

        // query contract state to check if contract instantiation works properly
        let contract_state = wasm
            .query::<QueryMsg, State>(&contract_addr, &QueryMsg::GetState {})
            .unwrap();

        println!("user 1 buys a share of user 1");
        println!(
            "user 1 balance before transaction: {:?}",
            get_balance(user_1.address(), &bank)
        );
        buy_share(
            &wasm,
            &contract_addr,
            &user_1.address(),
            &referring_user.address(),
            user_1,
            &[],
        );
        println!(
            "user 1 balance after transaction: {:?}",
            get_balance(user_1.address(), &bank)
        );

        println!("user 2 buys a share of user 2");
        println!(
            "user 2 balance before transaction: {:?}",
            get_balance(user_2.address(), &bank)
        );
        buy_share(
            &wasm,
            &contract_addr,
            &user_2.address(),
            &referring_user.address(),
            user_2,
            &[],
        );
        println!(
            "user 2 balance after transaction: {:?}",
            get_balance(user_2.address(), &bank)
        );

        println!("user 2 buys a share of user 1");
        let user_1_price = get_price(&wasm, &contract_addr, user_1.address(), true, true);
        println!(
            "user 1 price: {:?}",
            user_1_price.to_string()
        );
        println!(
            "user 2 balance before transaction: {:?}",
            get_balance(user_2.address(), &bank)
        );
        buy_share(
            &wasm,
            &contract_addr,
            &user_1.address(),
            &referring_user.address(),
            user_2,
            &[Coin::new(user_1_price.u128(), "inj")],
        );
        println!(
            "user 2 balance after transaction: {:?}",
            get_balance(user_2.address(), &bank)
        );
        println!(
            "referring user balance: {:?}",
            get_balance(referring_user.address(), &bank)
        );
    }

    fn get_balance(addr: String, bank: &Bank<'_, InjectiveTestApp>) -> String {
        let balance_request = QueryBalanceRequest {
            address: addr,
            denom: "inj".to_string(),
        };
        let balance_response = bank.query_balance(&balance_request.into()).unwrap();
        return balance_response.balance.unwrap().amount;
    }

    fn buy_share(
        wasm: &Wasm<'_, InjectiveTestApp>,
        contract_addr: &String,
        shares_subject: &String,
        referring_user: &String,
        signer: &SigningAccount,
        funds: &[Coin],
    ) {
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(shares_subject),
                referral: Addr::unchecked(referring_user),
            },
            funds, // empty funds when buying first share
            signer,
        )
        .unwrap();
    }

    fn get_price(
        wasm: &Wasm<'_, InjectiveTestApp>,
        contract_addr: &String,
        shares_subject: String,
        with_fees: bool,
        is_buy: bool,
    ) -> Uint128 {
        let price_response = wasm
            .query::<QueryMsg, GetPriceResponse>(
                &contract_addr,
                &QueryMsg::GetPrice {
                    shares_subject: Addr::unchecked(&shares_subject),
                    with_fees,
                    is_buy,
                },
            )
            .unwrap();
        return price_response.price;
    }
}
