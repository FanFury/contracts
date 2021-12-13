use cosmwasm_std::{Binary, Uint128};
use cw0::Expiration;
use cw20::{Cw20ReceiveMsg, Logo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing: Option<String>,
    pub logo: Option<Logo>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub admin_address: String,
    pub minting_contract_address: String,
}

use crate::state::{ GameResult, WalletPercentage };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    SetPlatformFeeWallets {
        wallet_percentages: Vec<WalletPercentage>
    },
    SetPoolTypeParams {
        pool_type: String,
        pool_fee: Uint128,
        min_teams_for_pool: u32,
        max_teams_for_pool: u32,
        max_teams_for_gamer: u32,
        wallet_percentages: Vec<WalletPercentage>,
    },
    CreateGame {
        game_id: String
    },
    CancelGame {
        game_id: String,
    },
    LockGame {
        game_id: String
    },
    CreatePool {
        game_id: String,
        pool_type: String
    },
    ClaimReward {
        gamer: String
    },
    GamePoolRewardDistribute {
        game_id: String,
        pool_id: String,
        game_winners: Vec<GameResult>
    },    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PoolTeamDetails {
        pool_id: String,
    },
    PoolDetails {
        pool_id: String,
    },
    PoolTypeDetails {
        pool_type: String,
    },
    AllTeams {},
    QueryReward {
        gamer: String
    },
    QueryGameResult {
        gamer: String,
        game_id: String,
        pool_id: String,
        team_id: String
    },
    GameDetails {
        game_id: String,
    },
    PoolTeamDetailsWithTeamId {
        pool_id: String,
        team_id: String,
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceivedMsg {
    GamePoolBidSubmit(GamePoolBidSubmitCommand),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GamePoolBidSubmitCommand {
    pub gamer: String,
    pub pool_type: String,
    pub pool_id: String,
    pub game_id: String,
    pub team_id: String,
}