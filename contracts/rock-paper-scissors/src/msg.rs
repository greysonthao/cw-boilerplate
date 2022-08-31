use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{GameMove, GameState};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame {
        opponent: String,
        host_move: GameMove,
    },
    OpponentResponse {
        host: String,
        opp_move: GameMove,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGameByHostAndOpponent { host: String, opponent: String },
    GetGamesByHost { host: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetGamesResponse {
    pub games: Vec<GameState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
