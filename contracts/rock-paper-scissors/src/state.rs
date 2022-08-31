use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Map;

use cw_controllers::Admin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameState {
    pub host: Addr,
    pub opponent: Addr,
    pub host_wager: Vec<Coin>,
    pub opp_wager: Option<Vec<Coin>>,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Leaderboard {
    pub host: Addr,
    pub opponent: Addr,
    pub host_score: Option<Uint128>,
    pub opp_score: Option<Uint128>,
    pub ties: Option<Uint128>,
}

pub const GAMES: Map<(&str, &str), GameState> = Map::new("games");
pub const LEADERBOARD: Map<(&str, &str), Leaderboard> = Map::new("leaderboard");

pub const ADMIN: Admin = Admin::new("admin");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins,
    OpponentWins,
    Tie,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameMove {
    Rock,
    Paper,
    Scissors,
}
