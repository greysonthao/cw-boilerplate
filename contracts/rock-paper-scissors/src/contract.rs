#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetGamesResponse, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{GameMove, GameResult, GameState, Leaderboard, ADMIN, GAMES, LEADERBOARD};

const CONTRACT_NAME: &str = "crates.io:rock-paper-scissors";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new().add_attribute("method", "migrate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg.admin {
        Some(admin) => {
            let valid_addr = deps.api.addr_validate(&admin)?;
            ADMIN.set(deps, Some(valid_addr.clone()))?;
            Ok(Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("admin", valid_addr))
        }
        None => {
            ADMIN.set(deps, None)?;
            Ok(Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("admin", "None"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => try_start_game(deps, info, opponent, host_move),
        ExecuteMsg::OpponentResponse { opp_move, host } => {
            try_opponent_response(deps, info, host, opp_move)
        }
    }
}

pub fn try_start_game(
    deps: DepsMut,
    info: MessageInfo,
    opponent: String,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    if info.sender.clone().to_string() == opponent.clone() {
        return Err(ContractError::HostAndOpponentCannotBeTheSame {
            val: opponent.clone(),
        });
    }
    //validate opp address
    let valid_addr = deps.api.addr_validate(&opponent.clone())?;

    //check if game already exists between the to addresses
    let host = info.sender.clone().to_string();
    match GAMES.load(deps.storage, (&host, &opponent)) {
        Ok(_) => return Err(ContractError::ActiveGameAlreadyExists {}),
        Err(_) => {}
    }

    //validate funds have been sent
    if info.clone().funds.len() == 0 {
        return Err(ContractError::MissingWagerAmount {});
    };

    let game = GameState {
        host: Addr::unchecked(info.sender.clone()),
        opponent: valid_addr.clone(),
        host_wager: info.funds.clone(),
        opp_wager: None,
        host_move: host_move.clone(),
        opp_move: None,
        result: None,
    };

    GAMES.save(
        deps.storage,
        (
            &info.sender.clone().to_string(),
            &valid_addr.clone().to_string(),
        ),
        &game,
    )?;
    GAMES.save(
        deps.storage,
        (
            &valid_addr.clone().to_string(),
            &info.sender.clone().to_string(),
        ),
        &game,
    )?;

    let mut host_wager = String::new();

    for coin in info.clone().funds.iter() {
        let coin_amount = coin.clone().amount.to_string();
        let coin_denom = coin.clone().denom;
        host_wager = host_wager + &coin_amount + &coin_denom + " ";
    }

    Ok(Response::new()
        .add_attribute("execute", "start_game")
        .add_attribute("host", info.sender)
        .add_attribute("opponent", valid_addr)
        .add_attribute("host_wager", host_wager))
}

pub fn try_opponent_response(
    deps: DepsMut,
    info: MessageInfo,
    host: String,
    opp_move: GameMove,
) -> Result<Response, ContractError> {
    let valid_host = deps.api.addr_validate(&host.clone())?;

    match GAMES.load(deps.storage, (&host, &info.sender.to_string())) {
        Ok(game) => {
            if info.funds.clone() != game.host_wager.clone() {
                return Err(ContractError::InsufficientWagerAmount {});
            }

            let result = get_game_result(&game.host_move, &opp_move)?;

            let result_of_game: String;

            if let prev_leadboard = LEADERBOARD.load(
                deps.storage,
                (
                    &game.host.clone().to_string(),
                    &game.opponent.clone().to_string(),
                ),
            )? {
                match result {
                    GameResult::HostWins => {
                        result_of_game = "host_wins".to_string();

                        let new_host_score = match prev_leadboard.host_score {
                            Some(h_score) => h_score.checked_add(Uint128::new(1)).unwrap(),
                            None => Uint128::new(1),
                        };

                        let new_leaderboard = Leaderboard {
                            host: prev_leadboard.host.clone(),
                            opponent: prev_leadboard.opponent.clone(),
                            host_score: Some(new_host_score.clone()),
                            opp_score: prev_leadboard.opp_score.clone(),
                            ties: prev_leadboard.ties.clone(),
                        };

                        LEADERBOARD.save(
                            deps.storage,
                            (
                                &new_leaderboard.host.to_string(),
                                &new_leaderboard.opponent.to_string(),
                            ),
                            &new_leaderboard,
                        )?;
                    }
                    GameResult::OpponentWins => {
                        result_of_game = "opponent_wins".to_string();

                        let new_opp_score = match prev_leadboard.opp_score {
                            Some(o_score) => o_score.checked_add(Uint128::new(1)).unwrap(),
                            None => Uint128::new(1),
                        };

                        let new_leaderboard = Leaderboard {
                            host: prev_leadboard.host.clone(),
                            opponent: prev_leadboard.opponent.clone(),
                            host_score: prev_leadboard.host_score.clone(),
                            opp_score: Some(new_opp_score.clone()),
                            ties: prev_leadboard.ties.clone(),
                        };

                        LEADERBOARD.save(
                            deps.storage,
                            (
                                &new_leaderboard.host.to_string(),
                                &new_leaderboard.opponent.to_string(),
                            ),
                            &new_leaderboard,
                        )?;
                    }
                    GameResult::Tie => {
                        result_of_game = "tie".to_string();

                        let new_ties_score = match prev_leadboard.ties {
                            Some(ties_score) => ties_score.checked_add(Uint128::new(1)).unwrap(),
                            None => Uint128::new(1),
                        };

                        let new_leaderboard = Leaderboard {
                            host: prev_leadboard.host.clone(),
                            opponent: prev_leadboard.opponent.clone(),
                            host_score: prev_leadboard.host_score.clone(),
                            opp_score: prev_leadboard.opp_score.clone(),
                            ties: Some(new_ties_score.clone()),
                        };

                        LEADERBOARD.save(
                            deps.storage,
                            (
                                &new_leaderboard.host.to_string(),
                                &new_leaderboard.opponent.to_string(),
                            ),
                            &new_leaderboard,
                        )?;
                    }
                }
            };

            /* match result {
                GameResult::HostWins => {
                    game_result = "host_wins".to_string();
                    leaderboard = Leaderboard {
                        host: game.host.clone(),
                        opponent: game.opponent.clone(),
                        host_score: Some(1),
                        opp_score: todo!(),
                        ties: todo!(),
                    }
                }
                GameResult::OpponentWins => game_result = "opponent_wins".to_string(),
                GameResult::Tie => game_result = "tie".to_string(),
            } */

            let bank_msg = send_funds_to_winner(
                result.clone(),
                info.funds.clone(),
                game.host_wager.clone(),
                valid_host,
                info.sender.clone(),
            )?;

            let new_game = GameState {
                host: game.host.clone(),
                opponent: game.opponent.clone(),
                host_wager: game.host_wager.clone(),
                opp_wager: Some(info.funds.clone()),
                host_move: game.host_move.clone(),
                opp_move: Some(opp_move.clone()),
                result: Some(result),
            };

            GAMES.save(
                deps.storage,
                (&host.clone(), &info.sender.clone().to_string()),
                &new_game,
            )?;

            GAMES.remove(
                deps.storage,
                (&host.clone(), &info.sender.to_string().clone()),
            );

            Ok(Response::new()
                .add_attribute("execute", "opponent_response")
                .add_attribute("host", host)
                .add_attribute("opponent", info.sender.to_string())
                //.add_attribute("game_result", result_of_game)
                .add_messages(bank_msg))
        }
        Err(_) => return Err(ContractError::GameNotFound {}),
    }
}

pub fn get_game_result(host_move: &GameMove, opp_move: &GameMove) -> StdResult<GameResult> {
    if &host_move == &opp_move {
        Ok(GameResult::Tie)
    } else if host_move == &GameMove::Rock && opp_move == &GameMove::Paper
        || host_move == &GameMove::Paper && opp_move == &GameMove::Scissors
        || host_move == &GameMove::Scissors && opp_move == &GameMove::Rock
    {
        Ok(GameResult::OpponentWins)
    } else {
        Ok(GameResult::HostWins)
    }
}

pub fn send_funds_to_winner(
    result: GameResult,
    opp_wager: Vec<Coin>,
    host_wager: Vec<Coin>,
    host: Addr,
    opponent: Addr,
) -> StdResult<Vec<BankMsg>> {
    let mut bank_msgs: Vec<BankMsg> = vec![];

    let mut msg: BankMsg;

    if result == GameResult::Tie {
        msg = BankMsg::Send {
            to_address: host.clone().to_string(),
            amount: host_wager.clone(),
        };

        bank_msgs.push(msg);

        let msg_1 = BankMsg::Send {
            to_address: opponent.clone().to_string(),
            amount: opp_wager.clone(),
        };
        bank_msgs.push(msg_1);
    };

    let total_wager = host_wager[0]
        .clone()
        .amount
        .checked_add(opp_wager[0].amount.clone())?;

    if result == GameResult::OpponentWins {
        msg = BankMsg::Send {
            to_address: opponent.to_string(),
            amount: vec![Coin {
                amount: total_wager,
                denom: host_wager[0].denom.clone(),
            }],
        };
        bank_msgs.push(msg);
    } else if result == GameResult::HostWins {
        msg = BankMsg::Send {
            to_address: host.to_string(),
            amount: vec![Coin {
                amount: total_wager,
                denom: host_wager[0].denom.clone(),
            }],
        };
        bank_msgs.push(msg);
    }

    Ok(bank_msgs)
}

/* pub fn get_prev_leaderboard(host: Addr, opponent: Addr) -> StdResult<Leaderboard> {
    match LEADERBOARD.load(de, k)
} */

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGameByHostAndOpponent { host, opponent } => {
            to_binary(&try_query_game_by_host_and_opponent(deps, host, opponent)?)
        }
        QueryMsg::GetGamesByHost { host } => to_binary(&try_query_games_by_host(deps, host)?),
    }
}

pub fn try_query_game_by_host_and_opponent(
    deps: Deps,
    host: String,
    opponent: String,
) -> StdResult<GetGamesResponse> {
    let _valid_host = deps.clone().api.addr_validate(&host.clone())?;
    let _valid_opp = deps.clone().api.addr_validate(&opponent.clone())?;
    let mut game: Vec<GameState> = vec![];

    match GAMES.load(deps.storage, (&host.clone(), &opponent.clone())) {
        Ok(g) => {
            game.push(g);

            Ok(GetGamesResponse { games: game })
        }
        Err(_) => Err(StdError::generic_err("No game found")),
    }
}

pub fn try_query_games_by_host(deps: Deps, host: String) -> StdResult<GetGamesResponse> {
    let _valid_host = deps.clone().api.addr_validate(&host.clone())?;

    let res: StdResult<Vec<_>> = GAMES
        .prefix(&host)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let games_found = res?;

    let mut games: Vec<GameState> = vec![];

    for game in games_found {
        games.push(game.1);
    }

    Ok(GetGamesResponse { games })
}
