#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate, query},
        msg::{ExecuteMsg, GetGamesResponse, InstantiateMsg, QueryMsg},
        state::GameMove,
        ContractError,
    };
    use cosmwasm_std::{
        from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        Coin, DepsMut, Response, StdError, Uint128,
    };

    const USER: &str = "user1";
    const DENOM: &str = "TNT";
    const AMOUNT: Uint128 = Uint128::new(100);
    const OPPONENT: &str = "opp1";

    fn instantiate_contract(deps: DepsMut) -> Response {
        let msg = InstantiateMsg {
            /* admin: Some(USER.to_string()), */
            admin: None,
        };
        let info = mock_info(
            USER,
            &[
                Coin {
                    denom: DENOM.to_string(),
                    amount: AMOUNT,
                },
                Coin {
                    denom: "LUNA".to_string(),
                    amount: Uint128::new(1),
                },
            ],
        );
        instantiate(deps, mock_env(), info, msg).unwrap()
    }

    fn start_game(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = ExecuteMsg::StartGame {
            opponent: OPPONENT.to_string(),
            host_move: GameMove::Rock,
        };
        let info = mock_info(
            &USER,
            &[Coin {
                denom: DENOM.to_string(),
                amount: AMOUNT,
            }],
        );

        execute(deps, mock_env(), info, msg)
    }

    fn start_game_host_and_opp_same(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = ExecuteMsg::StartGame {
            opponent: USER.to_string(),
            host_move: GameMove::Rock,
        };
        let info = mock_info(
            &USER,
            &[Coin {
                denom: DENOM.to_string(),
                amount: AMOUNT,
            }],
        );

        execute(deps, mock_env(), info, msg)
    }

    fn start_game_missing_funds(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = ExecuteMsg::StartGame {
            opponent: OPPONENT.to_string(),
            host_move: GameMove::Rock,
        };
        let info = mock_info(&USER, &[]);

        execute(deps, mock_env(), info, msg)
    }

    #[test]
    fn instantiate_test() {
        let mut deps = mock_dependencies();
        let res = instantiate_contract(deps.as_mut());
        //println!("RES: {:?}", res);
        assert_eq!(res.attributes[1].value, "None");
    }

    #[test]
    fn execute_test() {
        let mut deps = mock_dependencies();
        let _res = instantiate_contract(deps.as_mut());
        let res = start_game_host_and_opp_same(deps.as_mut());
        match res {
            Err(ContractError::HostAndOpponentCannotBeTheSame { val: _ }) => {}
            _ => panic!("Should error here"),
        }

        let res = start_game_missing_funds(deps.as_mut());
        match res {
            Err(ContractError::MissingWagerAmount {}) => {}
            _ => panic!("Should error here"),
        }

        let res = start_game(deps.as_mut()).unwrap();
        assert_eq!(res.attributes[2].value, "opp1");

        let res = start_game(deps.as_mut());
        match res {
            Err(ContractError::ActiveGameAlreadyExists {}) => {}
            _ => panic!("Should error here"),
        }
    }

    #[test]
    fn response_test() {
        let mut deps = mock_dependencies();
        let _res = instantiate_contract(deps.as_mut());
        let res = start_game(deps.as_mut()).unwrap();
        assert_eq!(res.attributes[2].value, "opp1");

        let query_msg = QueryMsg::GetGameByHostAndOpponent {
            host: USER.to_string(),
            opponent: OPPONENT.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let value: GetGamesResponse = from_binary(&res).unwrap();
        //print!("GAMES RESPONSE: {:?}", value);
        assert_eq!(value.games.len(), 1);

        let response_msg = ExecuteMsg::OpponentResponse {
            host: USER.to_string(),
            opp_move: GameMove::Paper,
        };
        let info = mock_info(
            OPPONENT,
            &[Coin {
                amount: AMOUNT,
                denom: DENOM.to_string(),
            }],
        );
        let res = execute(deps.as_mut(), mock_env(), info, response_msg).unwrap();
        println!("RESPONSE RES: {:?}", res);
        //assert_eq!(res.attributes[3].value, "opponent_wins");

        let res = query(deps.as_ref(), mock_env(), query_msg);

        match res {
            Err(StdError::GenericErr { msg: _ }) => {}
            _ => panic!("Should error here"),
        }
    }

    #[test]
    fn query_game_by_host_and_opp_test() {
        let mut deps = mock_dependencies();
        let _res = instantiate_contract(deps.as_mut());

        let query_msg = QueryMsg::GetGameByHostAndOpponent {
            host: USER.to_string(),
            opponent: OPPONENT.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg.clone());

        match res {
            Err(StdError::GenericErr { msg: _ }) => {}
            _ => panic!("Should error here"),
        }

        let _res = start_game(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let value: GetGamesResponse = from_binary(&res).unwrap();

        //print!("VALUE: {:?}", value);

        assert_eq!(value.games[0].host_move, GameMove::Rock);
    }

    #[test]
    fn query_games_by_host() {
        let mut deps = mock_dependencies();
        let _res = instantiate_contract(deps.as_mut());
        let _res = start_game(deps.as_mut());

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(
                USER,
                &[Coin {
                    denom: "LUNA".to_string(),
                    amount: Uint128::new(10),
                }],
            ),
            ExecuteMsg::StartGame {
                opponent: "other_guy".to_string(),
                host_move: GameMove::Paper,
            },
        )
        .unwrap();

        let query_msg = QueryMsg::GetGamesByHost {
            host: USER.to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let value: GetGamesResponse = from_binary(&res).unwrap();

        print!("VALUE: {:?}", value);

        assert_eq!(value.games.len(), 2);
    }
}
