mod inj_tests {
    use cosmwasm_std::{Addr, Coin, Uint128};
    use injective_test_tube::{Account, InjectiveTestApp, Module, Wasm};
    use rust_contract::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::State,
    };

    #[test]
    fn test() {
        let app = InjectiveTestApp::new();
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
        // `Wasm` is the module we use to interact with cosmwasm releated logic on the appchain
        // it implements `Module` trait which you will see more later.
        let wasm = Wasm::new(&app);

        // Load compiled wasm bytecode
        let wasm_byte_code = std::fs::read("./artifacts/rust_contract.wasm").unwrap();
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


        // have account 2 buy a share of account 2
        wasm.execute::<ExecuteMsg>(
            &contract_addr,
            &ExecuteMsg::BuyShares {
                shares_subject: Addr::unchecked(accs[1].address()),
                amount: Uint128::new(1),
            },
            &[Coin::new(100000000000000000, "inj")],
            &accs[1],
        )
        .unwrap();
    }
}
