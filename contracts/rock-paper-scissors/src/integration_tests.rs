#[cfg(test)]

mod tests {
    use crate::msg::{GetGamesResponse, InstantiateMsg, QueryMsg};
    use crate::state::GameMove;
    use crate::{contract, msg::ExecuteMsg};
    use anyhow::Result;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

    const USER: &str = "user";
    const OPPONENT: &str = "opponent";

    pub fn contract_rps() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(contract::execute, contract::instantiate, contract::query);

        Box::new(contract)
    }

    pub fn mock_app() -> App {
        let init_amount = vec![Coin {
            denom: "TNT".to_string(),
            amount: Uint128::new(100),
        }];

        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER.to_string()),
                    init_amount.clone(),
                )
                .unwrap()
        });

        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OPPONENT.to_string()),
                    init_amount.clone(),
                )
                .unwrap()
        });

        app
    }

    pub struct Suite {
        pub app: App,
        pub contract_id: u64,
        pub owner: String,
    }

    impl Suite {
        pub fn init() -> Result<Suite> {
            let mut app = mock_app();
            let contract_id = app.store_code(contract_rps());
            let owner = USER.to_string();

            Ok(Suite {
                app,
                contract_id,
                owner,
            })
        }

        pub fn instantiate(&mut self, admin: Option<String>) -> Result<Addr> {
            self.app.instantiate_contract(
                self.contract_id,
                Addr::unchecked(self.owner.to_string()),
                &InstantiateMsg { admin: None },
                &[],
                "rps",
                admin,
            )
        }

        pub fn execute(
            &mut self,
            contract_addr: Addr,
            msg: ExecuteMsg,
            send_funds: Vec<Coin>,
        ) -> Result<AppResponse> {
            self.app.execute_contract(
                Addr::unchecked(self.owner.to_string()),
                contract_addr,
                &msg,
                &send_funds,
            )
        }

        pub fn query(&self, contract_addr: Addr, msg: QueryMsg) -> Result<GetGamesResponse> {
            let res: GetGamesResponse = self
                .app
                .wrap()
                .query_wasm_smart(contract_addr, &msg)
                .unwrap();

            Ok(res)
        }
    }

    #[test]
    fn test_instantiate_contract() {
        let mut suite = Suite::init().unwrap();
        let contract_addr = suite.instantiate(None).unwrap();

        //println!("contract addr: {:?}", contract_addr);
        assert_eq!(contract_addr.clone(), Addr::unchecked("contract0"));
    }

    #[test]
    fn test_start_game() {
        let mut suite = Suite::init().unwrap();
        let contract_addr = suite.instantiate(None).unwrap();

        //println!("contract addr: {:?}", contract_addr);
        //assert_eq!(contract_addr.clone(), Addr::unchecked("contract0"));

        let msg = ExecuteMsg::StartGame {
            opponent: OPPONENT.to_string(),
            host_move: GameMove::Rock,
        };

        let host_wager = vec![Coin {
            denom: "TNT".to_string(),
            amount: Uint128::new(10),
        }];

        let res = suite
            .execute(contract_addr.clone(), msg, host_wager.clone())
            .unwrap();

        assert_eq!(res.events[1].attributes[4].value, "10TNT ".to_string());

        let msg = QueryMsg::GetGameByHostAndOpponent {
            host: USER.to_string(),
            opponent: OPPONENT.to_string(),
        };

        let res = suite.query(contract_addr.clone(), msg).unwrap();

        //println!("GET GAME BY HOST AND OPP: {:?}", res);
    }

    #[test]
    fn test_end_game() {
        let mut suite = Suite::init().unwrap();
        let contract_addr = suite.instantiate(None).unwrap();

        let msg = ExecuteMsg::StartGame {
            opponent: OPPONENT.to_string(),
            host_move: GameMove::Rock,
        };

        let host_wager = vec![Coin {
            denom: "TNT".to_string(),
            amount: Uint128::new(10),
        }];

        let _res = suite
            .execute(contract_addr.clone(), msg, host_wager.clone())
            .unwrap();

        let msg = ExecuteMsg::OpponentResponse {
            host: suite.owner.clone().to_string(),
            opp_move: GameMove::Rock,
        };

        let opp_wager = host_wager.clone();

        let res = suite
            .app
            .execute_contract(
                Addr::unchecked(OPPONENT.to_string()),
                contract_addr.clone(),
                &msg,
                &opp_wager,
            )
            .unwrap();

        //println!("END GAME: {:?}", res);

        assert_eq!(res.events[1].attributes[4].value, "tie".to_string());

        let res = suite
            .app
            .wrap()
            .query_balance(
                Addr::unchecked(suite.owner.clone()),
                opp_wager[0].denom.clone(),
            )
            .unwrap();

        assert_eq!(res.amount, Uint128::new(100));

        let res = suite
            .app
            .wrap()
            .query_balance(
                Addr::unchecked(OPPONENT.clone()),
                opp_wager[0].denom.clone(),
            )
            .unwrap();

        assert_eq!(res.amount, Uint128::new(100));
    }
}
